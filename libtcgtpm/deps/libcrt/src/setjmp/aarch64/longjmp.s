/* SPDX-License-Identifier: MIT */

.global _longjmp
.global longjmp
.type _longjmp, %function
.type longjmp, %function

/*
 * void longjmp(jmp_buf env, int val)
 *   x0 = env pointer
 *   x1 = val
 * return value = (val == 0 ? 1 : val)
 */

longjmp:
    // if val == 0, return 1
    cmp     x1, #0
    csel    x0, x1, xzr, ne    // x0 = val if val != 0
    csel    x0, x0, #1, eq     // x0 = 1 if val == 0

    // Restore registers from env (x0 = env)
    ldr     x19, [x0, #0]
    ldr     x20, [x0, #8]
    ldr     x21, [x0, #16]
    ldr     x22, [x0, #24]
    ldr     x23, [x0, #32]
    ldr     x24, [x0, #40]
    ldr     x25, [x0, #48]
    ldr     x26, [x0, #56]
    ldr     x27, [x0, #64]
    ldr     x28, [x0, #72]

    ldr     x29, [x0, #80]     // FP
    ldr     x2,  [x0, #88]     // SP
    ldr     x30, [x0, #96]     // LR

    mov     sp, x2             // update real SP

    ret
