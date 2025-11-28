use core::arch::asm;

#[repr(C)]
#[derive(Debug)]
pub struct ArmSmcccRes {
    pub a0: u64, // offset 0
    pub a1: u64, // offset 8
    pub a2: u64, // offset 16
    pub a3: u64, // offset 24
}

#[repr(C)]
#[derive(Debug)]
pub struct ArmSmcccQuirk {
    pub id: u64,    // offset 0
    pub state: u64, // offset 8
}

pub const ARM_SMCCC_QUIRK_QCOM_A6: u64 = 1;

#[inline(always)]
pub unsafe fn arm_smccc_smc(
    a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
    a4: u64,
    a5: u64,
    a6: u64,
    a7: u64,
    res: *mut ArmSmcccRes,
    quirk: *mut ArmSmcccQuirk,
) {
    unsafe{
        asm!(
            // ---- Call ----
            "smc #0",

            // ---- Save returned registers ----
            // stp x0, x1, [res, 0]
            "stp x0, x1, [{res_ptr}]",
            // stp x2, x3, [res, 16]
            "stp x2, x3, [{res_ptr}, #16]",

            // ---- Handle Qualcomm A6 quirk ----
            // if quirk == NULL → skip
            "cbz {quirk_ptr}, 2f",

            // load quirk.id
            "ldr x9, [{quirk_ptr}]",

            // compare quirk.id == ARM_SMCCC_QUIRK_QCOM_A6
            "cmp x9, {QUIRK_A6}",
            "b.ne 2f",

            // quirk.state = x6
            "str x6, [{quirk_ptr}, #8]",

            // ---- Return ----
            "2:",

            // Inputs mapped to registers
            inlateout("x0") a0 => _,
            inlateout("x1") a1 => _,
            inlateout("x2") a2 => _,
            inlateout("x3") a3 => _,
            in("x4") a4,
            in("x5") a5,
            in("x6") a6,
            in("x7") a7,

            // Pointers
            res_ptr = in(reg) res,
            quirk_ptr = in(reg) quirk,

            QUIRK_A6 = const ARM_SMCCC_QUIRK_QCOM_A6,

            // Clobbers
            out("x9") _,
            options(nostack)
        );
    }
}
