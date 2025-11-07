.section .text.exceptions_vector_table
// Export a symbol for the Rust code to use.
.globl exception_vector_table
exception_vector_table:
    /* Current EL with SP_EL0 */
    b vector_default      /* sync */
    b vector_irq          /* irq */
    b vector_default      /* fiq */
    b vector_default      /* serror */

    /* Current EL with SP_ELx */
    .org exception_vector_table + 0x80
    b vector_default
    b vector_irq
    b vector_default
    b vector_default

    /* Lower EL using AArch64 */
    .org exception_vector_table + 0x100
    b vector_default
    b vector_irq
    b vector_default
    b vector_default

    /* Lower EL using AArch32 (not used) */
    .org exception_vector_table + 0x180
    b vector_default
    b vector_irq
    b vector_default
    b vector_default

/* handlers */
vector_irq:
    /* IRQ handler entry: we must preserve regs if we call C/Rust */
    stp x0, x1, [sp, #-16]!   /* make room */
    stp x2, x3, [sp, #-16]!

    /* read interrupt ID from ICC_IAR1_EL1 */
    mrs x0, ICC_IAR1_EL1
    /* x0 now holds the interrupt ID (may include de-duplicated info) */
    bl  irq_entry_from_asm

    /* write EOI */
    msr ICC_EOIR1_EL1, x0
    isb

    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    eret

vector_default:
    /* for other exceptions we just loop */
    b .

/* symbol visibility to Rust */
    .global irq_entry_from_asm