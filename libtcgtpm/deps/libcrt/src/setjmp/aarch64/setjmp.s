/* SPDX-License-Identifier: MIT */

.global setjmp
.global _setjmp
.global __setjmp
.type setjmp, %function
.type _setjmp, %function
.type __setjmp, %function

/*
 * int setjmp(jmp_buf env)
 *   x0 = env pointer
 *
 * Save registers required by ABI (x19–x30)
 * Return 0
 */
setjmp:
_setjmp:
__setjmp:
    // Save callee-saved registers into env (x0)
    str     x19, [x0, #0]
    str     x20, [x0, #8]
    str     x21, [x0, #16]
    str     x22, [x0, #24]
    str     x23, [x0, #32]
    str     x24, [x0, #40]
    str     x25, [x0, #48]
    str     x26, [x0, #56]
    str     x27, [x0, #64]
    str     x28, [x0, #72]

    // Save FP (x29)
    str     x29, [x0, #80]

    // Save SP (same semantic as x86: SP AFTER return)
    mov     x1, sp
    add     x1, x1, #0      // (no need: just being explicit)
    str     x1, [x0, #88]

    // Save LR (return address)
    str     x30, [x0, #96]

    // return 0
    mov     x0, #0
    ret
