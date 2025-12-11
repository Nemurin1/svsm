.globl _start
.extern LD_STACK_PTR
.section ".text.boot"

_start:
    mov     x27, x0
    // stack
    ldr     x30, =LD_STACK_PTR
    mov     sp, x30

    // Initialize exceptions
    ldr     x0, =exception_vector_table
    msr     vbar_el1, x0
    isb

    // enabled NEON/FP registers
    mrs     x0, CPACR_EL1
    mov     x1, #3
    lsl     x1, x1, #20
    orr     x0, x0, x1
    msr     CPACR_EL1, x0
    isb 

    bl      _setup_mmu

// MMU
_setup_mmu:
    // Initialize translation table control registers
    ldr     x0, =TCR_EL1_VALUE
    msr     tcr_el1, x0
    ldr     x0, =MAIR_EL1_VALUE
    msr     mair_el1, x0
_setup_pagetable:
    // Set TTBRs
    ldr     x1, =LD_TTBR0_BASE
    msr     ttbr0_el1, x1 // L0 base address TTBR0
    ldr     x2, =LD_TTBR1_BASE
    msr     ttbr1_el1, x2 // L0 base address TTBR1

    //
    //  4KB granule，VA 48-bit：
    //  L0 @ LD_TTBR0_BASE
    //  L1 @ LD_TTBR0_BASE + 0x1000
    //  L2_0 @ LD_TTBR0_BASE + 0x2000   (map 0..1GB  2MB block entry)
    //  L2_1 @ LD_TTBR0_BASE + 0x3000   (map 1GB..2GB 2MB block entry)
    //
    //  each table 4KB，512 8-byte entries.
    //

    mov     x10, x1              // x10 = base (L0 addr)
    add     x11, x10, #0x1000    // x11 = L1 addr
    add     x12, x10, #0x2000    // x12 = L2_0 addr
    add     x13, x10, #0x3000    // x13 = L2_1 addr

    mov     x14, #0
    str     x14, [x10]           // L0[0] = 0
    str     x14, [x11]           // L1[0] = 0
    str     x14, [x12]           // L2_0[0] = 0
    str     x14, [x13]           // L2_1[0] = 0

    // L0[0] -> pointer to L1 (table descriptor: low bits = 0b11)
    // entry = (L1_base & ~0xfff) | 0b11
    orr     x14, x11, #3
    str     x14, [x10]           // L0[0] = &L1 | table-bit(3)

    // L1[0] -> pointer to L2_0; L1[1] -> pointer to L2_1 (table descriptors)
    mov     x15, #0x23
    orr     x14, x12, x15
    str     x14, [x11]           // L1[0] = &L2_0 | table-bit
    orr     x14, x13, #3
    str     x14, [x11, #8]       // L1[1] = &L2_1 | table-bit

    //
    // Now we should fill two L2 pagetables, each L2 pagetable hs 512 escriptors for 
    // 2MB blocks:
    // L2_0 map phys 0x0000_0000 .. 0x3FFF_FFFF (0 .. 1GB - 1)
    // L2_1 map phys 0x4000_0000 .. 0x7FFF_FFFF (1GB .. 2GB - 1)
    //

    // Load constant
    ldr     x20, =TWO_MB         // x20 = 2MB
    ldr     x21, =ONE_GB         // x21 = 1GB
    ldr     x22, =IDENTITY_MAP_ATTR
    ldr     x23, =PERIPHERALS_ATTR

    // === fill L2_0（512 entries）===
    mov     x24, #0              // counter i = 0 .. 511
1:  // if i == 0 --> phys = 0 -> use PERIPHERALS_ATTR, else use IDENTITY_MAP_ATTR
    // cbz     x24, .L2_0_entry_is_zero
    // phys = i * 2MB
    mul     x25, x24, x20       // x25 = phys
    orr     x26, x25, x23       // use PERIPHERALS_ATTR
    // entry = phys | IDENTITY_MAP_ATTR
    // orr     x26, x25, x22
    // b       .L2_0_store
// .L2_0_entry_is_zero:
    // phys = 0
    // mov     x25, #0
    // orr     x26, x25, x23      // use PERIPHERALS_ATTR for phys 0
// .L2_0_store:
    // store into L2_0 (sequentially). We'll write and advance pointer by 8 each time.
    str     x26, [x12], #8
    add     x24, x24, #1
    cmp     x24, #512
    blt     1b

    // === fill L2_1（512 entries），phys base = 1GB + i*2MB ===
    mov     x24, #0
2:  mul     x25, x24, x20      // x25 = i * 2MB
    add     x25, x25, x21      // phys = 1GB + (i*2MB)
    orr     x26, x25, x22      // use IDENTITY_MAP_ATTR for these
    str     x26, [x13], #8
    add     x24, x24, #1
    cmp     x24, #512
    blt     2b


    // ======= Insert a self-map entry =======
    mov     x17, #PGTABLE_LVL3_IDX_PTE_SELFMAP    // x17 = 493
    lsl     x17, x17, #3                          // x17 *= 8 -> 3944
    add     x15, x10, x17                         // x15 = x10 + offset
    orr     x16, x10, #3                          // table descriptor
    str     x16, [x15]                            // L0[493] = L0 | table-bit

    // ======= Modify UART0 L2 PTE: set IPA MSB (ipa_width= 41 -> bit40) =======
    // Compute L2_0 base again (we advanced x12 during fill, so recompute)
    // x10: L0 base (still holds earlier)
    add     x18, x10, #0x2000     // x18 = L2_0_base (byte address)

    // compute index for UART0 (phys 0x0900_0000)
    ldr     x19, =0x09000000      // UART0 physical base
    lsr     x19, x19, #21         // x19 = index = phys >> 21 (2MB granule)
    lsl     x19, x19, #3          // x19 *= 8 -> byte offset for entry

    add     x18, x18, x19         // x18 = &L2_0[index]

    // load-modify-store: set bit40
    ldr     x20, [x18]            // x20 = original PTE
    ldr     x21, =UNPROT_MASK     // x21 = 1 << 40
    orr     x20, x20, x21         // set IPA MSB (bit40)
    str     x20, [x18]            // write back PTE

_enable_mmu:
    // Enable the MMU.
    mrs     x0, sctlr_el1
    orr     x0, x0, #0x1
    msr     sctlr_el1, x0
    dsb     sy              //Programmer’s Guide for ARMv8-A chapter13.2 Barriers
    isb

_start_main:
    mov     x0, x27
    bl      not_main

.equ PSCI_SYSTEM_OFF, 0x84000002
.globl system_off
system_off:
    ldr     x0, =PSCI_SYSTEM_OFF
    hvc     #0

.equ TCR_EL1_VALUE, 0x5B5503510 // ---------------------------------------------
// IPS   | b101    << 32 | 36bits address space - 64GB
// TG1   | b10     << 30 | 4KB granule size for TTBR1_EL1
// SH1   | b11     << 28 | pagetable memory: Inner shareable
// ORGN1 | b01     << 26 | pagetable memory: Normal, Outer Wr.Back Rd.alloc Wr.alloc Cacheble
// IRGN1 | b01     << 24 | pagetable memory: Normal, Inner Wr.Back Rd.alloc Wr.alloc Cacheble
// EPD   | b0      << 23 | Perform translation table walk using TTBR1_EL1
// A1    | b1      << 22 | TTBR1_EL1.ASID defined the ASID
// T1SZ  | b010000 << 16 | Memory region 2^(64-16) -> 0xfffexxxxxxxxxxxx
// TG0   | b00     << 14 | 4KB granule size
// SH0   | b11     << 12 | pagetable memory: Inner Sharebale
// ORGN0 | b01     << 10 | pagetable memory: Normal, Outer Wr.Back Rd.alloc Wr.alloc Cacheble
// IRGN0 | b01     << 8  | pagetable memory: Normal, Inner Wr.Back Rd.alloc Wr.alloc Cacheble
// EPD0  | b0      << 7  | Perform translation table walk using TTBR0_EL1
// 0     | b0      << 6  | Zero field (reserve)
// T0SZ  | b010000 << 0  | Memory region 2^(64-16)

.equ MAIR_EL1_VALUE, 0xFF440C0400// ---------------------------------------------
//                   INDX         MAIR
// DEVICE_nGnRnE    b000(0)     b00000000
// DEVICE_nGnRE     b001(1)     b00000100
// DEVICE_GRE       b010(2)     b00001100
// NORMAL_NC        b011(3)     b01000100
// NORMAL           b100(4)     b11111111

.equ PERIPHERALS_ATTR, 0x60000000000601 // -------------------------------------
// UXN   | b1      << 54 | Unprivileged execute Never
// PXN   | b1      << 53 | Privileged execute Never
// AF    | b1      << 10 | Access Flag
// SH    | b10     << 8  | Outer shareable
// AP    | b00     << 6  | R/W, EL0 access denied
// NS    | b0      << 5  | Security bit (EL3 and Secure EL1 only)
// INDX  | b000    << 2  | Attribute index in MAIR_ELn，see MAIR_EL1_VALUE
// ENTRY | b01     << 0  | Block entry

.equ IDENTITY_MAP_ATTR, 0x40000000000711 // ------------------------------------
// UXN   | b1      << 54 | Unprivileged eXecute Never
// PXN   | b0      << 53 | Privileged eXecute Never
// AF    | b1      << 10 | Access Flag
// SH    | b11     << 8  | Inner shareable
// AP    | b00     << 6  | R/W, EL0 access denied
// NS    | b0      << 5  | Security bit (EL3 and Secure EL1 only)
// INDX  | b100    << 2  | Attribute index in MAIR_ELn，see MAIR_EL1_VALUE
// ENTRY | b01     << 0  | Block entry

.equ TWO_MB, 0x00200000
.equ ONE_GB, 0x40000000
.equ PGTABLE_LVL3_IDX_PTE_SELFMAP, 493
// self-map in L0 493 entry
.equ UNPROT_MASK, 0x10000000000  
// 1 << 40, for ipa_width = 41
.equ ONE_GB, 0x40000000