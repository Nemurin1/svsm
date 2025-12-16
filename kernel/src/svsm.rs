// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 SUSE LLC
//
// Author: Joerg Roedel <jroedel@suse.de>

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

use bootlib::kernel_launch::{KernelLaunchInfo, Stage2LaunchInfo, STAGE2_HEAP_END, STAGE2_HEAP_START};
use bootlib::platform::SvsmPlatformType;
use core::arch::global_asm;
use core::arch::asm;
use core::panic::PanicInfo;
use core::slice;
use cpuarch::snp_cpuid::SnpCpuidTable;
use svsm::address::{Address, PhysAddr, VirtAddr};
#[cfg(feature = "attest")]
use svsm::attest::AttestationDriver;
use svsm::config::SvsmConfig;
use svsm::console::install_console_logger;
use svsm::cpu::control_regs::{cr0_init, cr4_init};
use svsm::cpu::cpuid::{dump_cpuid_table, register_cpuid_table};
use svsm::cpu::gdt::GLOBAL_GDT;
use svsm::cpu::idt::svsm::{early_idt_init, idt_init};
use svsm::cpu::idt::{IdtEntry, EARLY_IDT_ENTRIES, IDT};
use svsm::cpu::percpu::{cpu_idle_loop, this_cpu, try_this_cpu, PerCpu, PERCPU_AREAS};
use svsm::cpu::shadow_stack::{
    determine_cet_support, is_cet_ss_supported, shadow_stack_info, SCetFlags, MODE_64BIT, S_CET,
};
use svsm::cpu::smp::start_secondary_cpus;
use svsm::cpu::sse::sse_init;
use svsm::debug::gdbstub::svsm_gdbstub::{debug_break, gdbstub_start};
// use svsm::debug::stacktrace::print_stack;
use svsm::enable_shadow_stacks;
use svsm::fs::{initialize_fs, opendir, populate_ram_fs};
use svsm::hyperv::hyperv_setup;
use svsm::igvm_params::IgvmParams;
use svsm::kernel_region::new_kernel_region;
use svsm::mm::alloc::{memory_info, print_memory_info, root_mem_init};
use svsm::mm::memory::init_memory_map;
use svsm::mm::pagetable::paging_init;
// use svsm::mm::ro_after_init::make_ro_after_init;
use svsm::mm::virtualrange::virt_log_usage;
use svsm::mm::{init_kernel_mapping_info, FixedAddressMappingRange};
use svsm::mm::validate::{
    init_valid_bitmap_alloc, valid_bitmap_addr, valid_bitmap_addr_virt,
};
use svsm::platform;
use svsm::platform::{init_capabilities, init_platform_type, SvsmPlatformCell, SVSM_PLATFORM};
use svsm::requests::request_loop_main;
use svsm::sev::secrets_page_mut;
use svsm::svsm_paging::{init_page_table, init_page_table_arm, invalidate_early_boot_memory};
use svsm::task::schedule_init;
use svsm::task::{exec_user, start_kernel_task};
use svsm::types::PAGE_SIZE;
use svsm::utils::{immut_after_init::ImmutAfterInitCell, zero_mem_region, MemoryRegion};
#[cfg(all(feature = "vtpm", not(test)))]
use svsm::vtpm::vtpm_init;

use svsm::mm::validate::{init_valid_bitmap_ptr, migrate_valid_bitmap};

use alloc::string::String;
use release::COCONUT_VERSION;

use svsm::svsm_arm64::cpu::gicv3::{gicv3_init, gicv3_enable_irq};

// use svsm::stage2::stage2_main;

#[cfg(feature = "attest")]
use kbs_types::Tee;

#[cfg(feature = "cca")]
use svsm::realm::rsi::rsi_cmd::{init_realm_config};
#[cfg(feature = "cca")]
use svsm::svsm_arm64::cpu::gicv3::{init_mmio_gic};
#[cfg(feature = "cca")]
use svsm::console::{init_mmio_uart};

extern "C" {
    static bsp_stack: u8;
    static bsp_stack_end: u8;
}

extern "C" {
    static kernel_region_phys_start: u64;
    static kernel_region_phys_end: u64;
}

/*
 * Launch protocol:
 *
 * The stage2 loader will map and load the svsm binary image and jump to
 * startup_64.
 *
 * %rdi  Pointer to the KernelLaunchInfo structure
 * %rsi  Pointer to the valid-bitmap from stage2
 */
global_asm!(include_str!("start.s"));

static CPUID_PAGE: ImmutAfterInitCell<SnpCpuidTable> = ImmutAfterInitCell::uninit();
static LAUNCH_INFO: ImmutAfterInitCell<KernelLaunchInfo> = ImmutAfterInitCell::uninit();


pub fn memory_init(launch_info: &KernelLaunchInfo) {
    root_mem_init(
        PhysAddr::from(launch_info.heap_area_phys_start),
        VirtAddr::from(launch_info.heap_area_virt_start),
        launch_info.heap_area_size as usize / PAGE_SIZE,
    );
}


fn boot_stack_info() {
    let bs = this_cpu().get_current_stack();
    log::info!("Boot stack @ {bs:#018x}");
}

fn mapping_info_init(launch_info: &KernelLaunchInfo) {
    let kernel_mapping = FixedAddressMappingRange::new(
        VirtAddr::from(launch_info.heap_area_virt_start),
        VirtAddr::from(launch_info.heap_area_virt_end()),
        PhysAddr::from(launch_info.heap_area_phys_start),
    );
    init_kernel_mapping_info(kernel_mapping, None);
}

/// # Panics
///
/// Panics if the provided address is not aligned to a [`SnpCpuidTable`].

fn init_cpuid_table(addr: VirtAddr) {
    // SAFETY: this is called from the main function for the SVSM and no other
    // CPUs have been brought up, so the pointer cannot be aliased.
    // `aligned_mut()` will check alignment for us.
    let table = unsafe {
        addr.aligned_mut::<SnpCpuidTable>()
            .expect("Misaligned SNP CPUID table address")
    };

    for func in table.func.iter_mut().take(table.count as usize) {
        if func.eax_in == 0x8000001f {
            func.eax_out |= 1 << 28;
        }
    }

    CPUID_PAGE
        .init_from_ref(table)
        .expect("Already initialized CPUID page");
    register_cpuid_table(&CPUID_PAGE);
}

fn setup_svsm_early_allocator(heap_start: u64, heap_end: u64) {
    let vstart = VirtAddr::from(heap_start);
    let vend = VirtAddr::from(heap_end);
    let pstart = PhysAddr::from(vstart.bits()); // Identity mapping
    let nr_pages = (vend - vstart) / PAGE_SIZE;

    root_mem_init(pstart, vstart, nr_pages);
}

#[no_mangle]
extern "C" fn svsm_start(/* li: &KernelLaunchInfo, vb_addr: usize */fdt_addr: u64) -> ! {
    let kernel_start = unsafe { &kernel_region_phys_start as *const u64 as u64 };
    let kernel_end = unsafe { &kernel_region_phys_end as *const u64 as u64 };
    let kernel_size: u64 = 256 * 1024;

    let linfo = KernelLaunchInfo {
        kernel_region_phys_start: kernel_start,
        kernel_region_phys_end: kernel_end,
        heap_area_phys_start: kernel_end,
        heap_area_virt_start: kernel_end,
        heap_area_size: kernel_size,
        kernel_elf_stage2_virt_start: kernel_start,
        kernel_elf_stage2_virt_end: kernel_end,
        kernel_fs_start: 0,
        kernel_fs_end: 0,
        stage2_start: 0,
        stage2_end: 0,
        cpuid_page: 0,
        secrets_page: 0,
        stage2_igvm_params_phys_addr: 0,
        vtom: 0,
        debug_serial_port: 0x0900_0000,
        use_alternate_injection: false,
        suppress_svsm_interrupts: true,
        platform_type: SvsmPlatformType::Native,
        kernel_region_virt_start: kernel_start,
        stage2_igvm_params_size: 0,
        igvm_params_phys_addr: 0,
        igvm_params_virt_addr: 0,
        };

    let li = &linfo;
    let launch_info: KernelLaunchInfo = *li;

    init_platform_type(launch_info.platform_type);
    let mut platform_cell = SvsmPlatformCell::new(li.suppress_svsm_interrupts);
    let platform = platform_cell.platform_mut();

    let debug_serial_port = li.debug_serial_port;
    install_console_logger("SVSM").expect("Console logger already initialized");
    platform
        .env_setup(debug_serial_port, launch_info.vtom.try_into().unwrap())
        .expect("Early environment setup failed");

    // 初始化堆
    setup_svsm_early_allocator(STAGE2_HEAP_START.into(), STAGE2_HEAP_END.into());

    let kernel_base: PhysAddr = kernel_start.into();
    let kernel_size: usize = 4 * 1024 * 1024;

    let kernel_region = MemoryRegion::<PhysAddr>::new(
            kernel_base,
            kernel_size.try_into().unwrap(),
        );

    init_valid_bitmap_alloc(kernel_region).expect("Failed to allocate valid-bitmap");

    // let valid_bitmap = valid_bitmap_addr(); // move from stage2
    let valid_bitmap = valid_bitmap_addr_virt();
    let vb_addr: usize = valid_bitmap.bits();// move from stage2
    let vb_ptr = core::ptr::NonNull::new(VirtAddr::new(vb_addr).as_mut_ptr::<u64>()).unwrap();

    mapping_info_init(&launch_info);
    // log::info!("!!!");
    // log::info!("!!!");

    // SAFETY: we trust the previous stage to pass a valid pointer
    unsafe { init_valid_bitmap_ptr(new_kernel_region(&launch_info), vb_ptr) };

    // GLOBAL_GDT.load_selectors();

    let mut early_idt = [IdtEntry::default(); EARLY_IDT_ENTRIES];
    let mut idt = IDT::new(&mut early_idt);
    // SAFETY: the IDT here will remain in scope until the full IDT is
    // initialized later, and thus can safely be used as the early IDT.
    unsafe {
        early_idt_init(&mut idt);
    }
    log::info!("{} {} {}", kernel_start, kernel_end, kernel_size);

    // Capture the debug serial port before the launch info disappears from
    // the address space.
    // let debug_serial_port = li.debug_serial_port;

    LAUNCH_INFO
        .init_from_ref(li)
        .expect("Already initialized launch info");



    // init_cpuid_table(VirtAddr::from(launch_info.cpuid_page));

    // let secrets_page_virt = VirtAddr::from(launch_info.secrets_page);

    // SAFETY: the secrets page address directly comes from IGVM.
    // We trust stage 2 to give the value provided by IGVM.
    /*
    unsafe {
        secrets_page_mut().copy_from(secrets_page_virt);
        zero_mem_region(secrets_page_virt, secrets_page_virt + PAGE_SIZE);
    }
    */

    // cr0_init();
    // determine_cet_support(platform);
    // cr4_init(platform);

    // data abort
    // memory_init(&launch_info);
    migrate_valid_bitmap().expect("Failed to migrate valid-bitmap");

    /*

    /* 获取stage2 elf文件的长度和指向其起始位置的指针 */
    let kernel_elf_len = (launch_info.kernel_elf_stage2_virt_end
        - launch_info.kernel_elf_stage2_virt_start) as usize;
    log::info!("...");
    let kernel_elf_buf_ptr = launch_info.kernel_elf_stage2_virt_start as *const u8;
    log::info!("...");
    // SAFETY: we trust stage 2 to pass on a correct pointer and length. This
    // cannot be aliased because we are on CPU 0 and other CPUs have not been
    // brought up. The resulting slice is &[u8], so there are no alignment
    // requirements.
    let kernel_elf_buf = unsafe { slice::from_raw_parts(kernel_elf_buf_ptr, kernel_elf_len) };
    log::info!("...");
    let kernel_elf = match elf::Elf64File::read(kernel_elf_buf) {
        Ok(kernel_elf) => kernel_elf,
        Err(e) => panic!("error reading kernel ELF: {}", e),
    };
    log::info!("...");
    /* 到这里创造了一个elf文件的实体，存储了elf文件的各种信息 */

    */

    // 这一步初始化了加密掩码和页表标志位掩码
    paging_init(platform, false).expect("Failed to initialize paging");
    // let init_pgtable =
    //     init_page_table(&launch_info, &kernel_elf).expect("Could not initialize the page table");
    // let init_pgtable =
    //     init_page_table_arm(&launch_info).expect("Could not initialize the page table");
    // SAFETY: we are initializing the state, including stack and registers
    /*
    unsafe {
        init_pgtable.load();
    }
    */

    // SAFETY: this is the first CPU, so there can be no other dependencies
    // on multi-threaded access to the per-cpu areas.
    /*
    let percpu_shared = unsafe { PERCPU_AREAS.create_new(0) };
    let bsp_percpu = PerCpu::alloc(percpu_shared).expect("Failed to allocate BSP per-cpu data");

    bsp_percpu
        .setup(platform, init_pgtable)
        .expect("Failed to setup BSP per-cpu area");
    bsp_percpu
        .setup_on_cpu(platform)
        .expect("Failed to run percpu.setup_on_cpu()");
    bsp_percpu.load();
    // Now the stack unwinder can be used
    bsp_percpu.set_current_stack(MemoryRegion::from_addresses(
        VirtAddr::from(&raw const bsp_stack),
        VirtAddr::from(&raw const bsp_stack_end),
    ));
    */
    // idt_init().expect("Failed to allocate IDT");

    /* 
    if is_cet_ss_supported() {
        enable_shadow_stacks!(bsp_percpu);
    }
    */

    initialize_fs();

    // Idle task must be allocated after PerCPU data is mapped
    /*
    bsp_percpu
        .setup_idle_task(svsm_main)
        .expect("Failed to allocate idle task for BSP");
    */

    platform
        .env_setup_late(debug_serial_port)
        .expect("Late environment setup failed");

    // dump_cpuid_table();

    let mem_info = memory_info();
    print_memory_info(&mem_info);
    // boot_stack_info();
    // shadow_stack_info();

    platform
        .configure_alternate_injection(launch_info.use_alternate_injection)
        .expect("Alternate injection required but not available");

    platform_cell.global_init();

    // sse_init();

    // SAFETY: there is no current task running on this processor yet, so
    // initializing the scheduler is safe.
    // unsafe {
    //     schedule_init();
    // }

    svsm_main(0, fdt_addr);

    unreachable!("SVSM entry point terminated unexpectedly");
}


#[no_mangle]
pub extern "C" fn svsm_main(cpu_index: usize, fdt_addr: u64) {
    debug_assert_eq!(cpu_index, 0);

    // let launch_info: Stage2LaunchInfo = Default::default();

    // stage2_main(&launch_info);

    // If required, the GDB stub can be started earlier, just after the console
    // is initialised in svsm_start() above.
    // gdbstub_start(&**SVSM_PLATFORM).expect("Could not start GDB stub");
    // Uncomment the line below if you want to wait for
    // a remote GDB connection
    // debug_break();

    SVSM_PLATFORM
        .env_setup_svsm()
        .expect("SVSM platform environment setup failed");

    hyperv_setup().expect("failed to complete Hyper-V setup");

    let launch_info = &*LAUNCH_INFO;
    log::info!("test info");
    // let igvm_params = IgvmParams::new(VirtAddr::from(launch_info.igvm_params_virt_addr))
    //     .expect("Invalid IGVM parameters");
    let igvm_params: IgvmParams<'_> = IgvmParams::new_empty().expect("Invalid IGVM parameters");
    if (launch_info.vtom != 0) && (launch_info.vtom != igvm_params.get_vtom()) {
        panic!("Launch VTOM does not match VTOM from IGVM parameters");
    }

    let config = SvsmConfig::new(*SVSM_PLATFORM, igvm_params);

    init_memory_map(&config, &LAUNCH_INFO).expect("Failed to init guest memory map");

    populate_ram_fs(LAUNCH_INFO.kernel_fs_start, LAUNCH_INFO.kernel_fs_end)
        .expect("Failed to unpack FS archive");

    init_capabilities();

    // let cpus = config.load_cpu_info().expect("Failed to load ACPI tables");

    // start_secondary_cpus(&**SVSM_PLATFORM, &cpus);

    // Make ro_after_init section read-only
    // make_ro_after_init().expect("Failed to make ro_after_init region read-only");

    invalidate_early_boot_memory(&**SVSM_PLATFORM, &config, launch_info)
        .expect("Failed to invalidate early boot memory");

    if let Err(e) = SVSM_PLATFORM.prepare_fw(&config, new_kernel_region(&LAUNCH_INFO)) {
        panic!("Failed to prepare guest FW: {e:#?}");
    }

    #[cfg(feature = "attest")]
    {
        let mut proxy = AttestationDriver::try_from(Tee::Snp).unwrap();
        let _data = proxy.attest().unwrap();

        // Nothing to do with data at the moment, simply print a success message.
        log::info!("attestation successful");
    }

    #[cfg(all(feature = "vtpm", not(test)))]
    vtpm_init().expect("vTPM failed to initialize");

    // virt_log_usage();

    if let Err(e) = SVSM_PLATFORM.launch_fw(&config) {
        panic!("Failed to launch FW: {e:?}");
    }

    #[cfg(test)]
    {
        if config.has_qemu_testdev() {
            crate::testutils::set_has_qemu_testdev();
        }
        if config.has_test_iorequests() {
            crate::testutils::set_has_test_iorequests();
        }
        crate::test_main();
    }

    match exec_user("/init", opendir("/").expect("Failed to find FS root")) {
        Ok(_) => (),
        Err(e) => log::info!("Failed to launch /init: {e:?}"),
    }

    // Start request processing on this CPU if required.
    // request_loop_main(0);
    /*
    if SVSM_PLATFORM.start_svsm_request_loop() {
        start_kernel_task(request_loop_main, 0, String::from("request-loop on CPU 0"))
            .expect("Failed to launch request loop task");
    }
    */

    log::info!("Enter request loop and pass device tree address");
    log::info!("Device Tree address: {:#018x}", fdt_addr);
    request_loop_main(cpu_index);

    log::info!("SVSM native launch completed and enter idle loop");
    platform::halt();
}

#[no_mangle]
pub extern "C" fn not_main(fdt_addr: u64) {
    #[cfg(feature = "cca")]
    {
        // 第一步调用rsi_realm_config获取配置信息
        init_realm_config().expect("REALM_CONFIG already initialized");
        // 然后根据配置信息中的ipa_width来修改页表项
        // init_mmio_gic().expect("Cannot map gic in unprotected IPA");
        init_mmio_uart().expect("Cannot map uart in unprotected IPA");;
    }

    // ...

    /*
    unsafe {
        // Initialize GICv3
        gicv3_init();
        
        // Example: enable a plausible interrupt ID for testing.
        // Choose an interrupt ID that QEMU generates (for virt machine,
        // Example UART0 is typically at ID 33 in some setups; you may adjust).
        // We enable ID 33 here as a demo; if no source asserts it, you won't see IRQ.
        gicv3_enable_irq(33);
    }

    // Initialize MMU


    // Enter svsm_main and load guest kernel
    // svsm_main();

    // Enable interrupts in PSTATE (clear I bit)
    unsafe {
        asm!("msr DAIFCLR, #2"); // enable IRQ only
    }
    */

    svsm_start(fdt_addr);
    unsafe{
      loop {
        // low-power wait; interrupts will preempt into vector table
        asm!("wfi");
      }
    }

  // ...
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    /*
    secrets_page_mut().clear_vmpck(0);
    secrets_page_mut().clear_vmpck(1);
    secrets_page_mut().clear_vmpck(2);
    secrets_page_mut().clear_vmpck(3);
    */

    if let Some(cpu) = try_this_cpu() {
        log::error!(
            "Panic on CPU[{}]! COCONUT-SVSM Version: {}",
            cpu.get_cpu_index(),
            COCONUT_VERSION
        );
    } else {
        log::error!("Panic on CPU[?]! COCONUT-SVSM Version: {}", COCONUT_VERSION);
    }
    log::error!("Info: {}", info);

    // print_stack(3);

    loop {
        debug_break();
        platform::halt();
    }
}
