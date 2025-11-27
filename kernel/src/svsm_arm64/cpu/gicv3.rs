
pub use core::arch::asm;
// use core::arch::global_asm;

/// These base addresses match QEMU 'virt' defaults for GICv3.
/// If your platform differs, replace with values from its DTB.
const GICD_BASE: usize = 0x0800_0000;
const GICR_BASE: usize = 0x080A_0000; // base of redistributor for CPU0

// GICD offsets
const GICD_CTLR: usize = 0x0000;
const GICD_ISENABLER: usize = 0x0100;

// GICR offsets (within redistributor)
const GICR_WAKER: usize = 0x0014;

fn mmio_write32(addr: usize, v: u32) {
    unsafe{core::ptr::write_volatile(addr as *mut u32, v);}
}

fn mmio_read32(addr: usize) -> u32 {
    unsafe{core::ptr::read_volatile(addr as *const u32)}
}

/// Initialize minimal GICv3 for CPU0
pub fn gicv3_init() {
    // 1) Enable System Register Access (ICC_SRE_EL1.SRE = 1)
    let mut sre: u64;
    unsafe{
        asm!("mrs {0}, ICC_SRE_EL1", out(reg) sre);
        sre |= 1;
        asm!("msr ICC_SRE_EL1, {0}", in(reg) sre);
        asm!("isb");

        // 2) Wake up Redistributor (clear ProcessorSleep)
        let waker = GICR_BASE + GICR_WAKER;
        let mut w = mmio_read32(waker);
        w &= !(1 << 1); // clear ProcessorSleep
        mmio_write32(waker, w);
        // wait for ChildrenAsleep == 0
        while (mmio_read32(waker) & (1 << 2)) != 0 {}

        // 3) Enable Distributor (EnableGrp1, bit0)
        mmio_write32(GICD_BASE + GICD_CTLR, 1);

        // 4) Enable Group1 interrupts at CPU interface
        asm!("msr ICC_IGRPEN1_EL1, {0}", in(reg) 1u64);
        asm!("isb");

        // 5) Set priority mask to allow all priorities (0xff)
        asm!("msr ICC_PMR_EL1, {0}", in(reg) 0xffu64);
        asm!("isb");

        // Clear Binary Point register (no preemption priority)
        asm!("msr ICC_BPR1_EL1, {0}", in(reg) 0u64);
    }

    // Done
}

/// Enable a global interrupt ID (SPI/PPI) in distributor
pub fn gicv3_enable_irq(irq: u32) {
    let reg = GICD_BASE + GICD_ISENABLER + ((irq as usize / 32) * 4);
    let bit = 1u32 << (irq % 32);
    mmio_write32(reg, bit);
}
