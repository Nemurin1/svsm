#![allow(non_camel_case_types)]
#![allow(dead_code)]
pub mod rsi_cmd;
pub mod smccc;

use core::mem::size_of;

pub mod retcodes {
    /// RSI return codes
    pub const RSI_SUCCESS: u64 = 0;
    pub const RSI_ERROR_INPUT: u64 = 1;
    pub const RSI_ERROR_STATE: u64 = 2;
    pub const RSI_INCOMPLETE: u64 = 3;
    pub const RSI_ERROR_UNKNOWN: u64 = 4;
}

/// Helper to build SMC call FIDs similar to ARM_SMCCC_CALL_VAL macro.
/// The exact shift values depend on the SMCCC implementation in the target
/// environment. The values below follow the typical Linux kernel layout:
pub mod arm_smccc {
    /// These shifts are the usual ones used in ARM SMCCC macros (as in Linux).
    /// If you're using kernel headers, replace these with the kernel's values.
    pub const TYPE_SHIFT: u64 = 31;
    pub const CC_SHIFT: u64 = 30;
    pub const OWNER_SHIFT: u64 = 24;

    /// Typical values used in Linux headers:
    /// (replace with kernel-provided constants if you have them)
    pub const ARM_SMCCC_FAST_CALL: u64 = 1;
    pub const ARM_SMCCC_STD_CALL: u64 = 0;
    pub const ARM_SMCCC_SMC_64: u64 = 1; // indicates 64-bit SMC convention
    pub const ARM_SMCCC_OWNER_STANDARD: u64 = 0;

    /// Build call value (similar to ARM_SMCCC_CALL_VAL macro)
    pub const fn call_val(typ: u64, cc: u64, owner: u64, func_num: u64) -> u64 {
        ((typ) << TYPE_SHIFT)
            | ((cc) << CC_SHIFT)
            | ((owner) << OWNER_SHIFT)
            | (func_num)
    }

    /// Convenience wrapper for the common case used in the header:
    pub const fn fid(n: u64) -> u64 {
        call_val(
            ARM_SMCCC_FAST_CALL,
            ARM_SMCCC_SMC_64,
            ARM_SMCCC_OWNER_STANDARD,
            n,
        )
    }
}

/// RSI ABI version constants
pub mod abi {
    pub const RSI_ABI_VERSION_MAJOR: u64 = 1;
    pub const RSI_ABI_VERSION_MINOR: u64 = 1;
    pub const RSI_ABI_VERSION: u64 =
        (RSI_ABI_VERSION_MAJOR << 16) | (RSI_ABI_VERSION_MINOR & 0xFFFF);

    pub const fn get_major(version: u64) -> u64 {
        version >> 16
    }
    pub const fn get_minor(version: u64) -> u64 {
        version & 0xFFFF
    }
}

/// SMC FIDs defined in the header
pub mod fid {
    use super::arm_smccc;

    pub const SMC_RSI_ABI_VERSION: u64 = arm_smccc::fid(0x190);
    pub const SMC_RSI_FEATURES: u64 = arm_smccc::fid(0x191);
    pub const SMC_RSI_MEASUREMENT_READ: u64 = arm_smccc::fid(0x192);
    pub const SMC_RSI_MEASUREMENT_EXTEND: u64 = arm_smccc::fid(0x193);
    pub const SMC_RSI_ATTESTATION_TOKEN_INIT: u64 = arm_smccc::fid(0x194);
    pub const SMC_RSI_ATTESTATION_TOKEN_CONTINUE: u64 = arm_smccc::fid(0x195);
    pub const SMC_RSI_REALM_CONFIG: u64 = arm_smccc::fid(0x196);
    pub const SMC_RSI_IPA_STATE_SET: u64 = arm_smccc::fid(0x197);
    pub const SMC_RSI_IPA_STATE_GET: u64 = arm_smccc::fid(0x198);
    pub const SMC_RSI_HOST_CALL: u64 = arm_smccc::fid(0x199);
    pub const SMC_RSI_PLANE_ENTER: u64 = arm_smccc::fid(0x1A3);
    pub const SMC_RSI_PLANE_SYSREG_READ: u64 = arm_smccc::fid(0x1AE);
    pub const SMC_RSI_PLANE_SYSREG_WRITE: u64 = arm_smccc::fid(0x1AF);
}

/// RSI change/accept constants
pub mod ipa_consts {
    pub const RSI_NO_CHANGE_DESTROYED: u64 = 0;
    pub const RSI_CHANGE_DESTROYED: u64 = 1;

    pub const RSI_ACCEPT: u64 = 0;
    pub const RSI_REJECT: u64 = 1;
}

/// Plane / GIC constants
pub const PLANE_RUN_GPRS: usize = 31;
pub const PLANE_GIC_NUM_LRS: usize = 16;

/// RSI trap constants
pub mod trap {
    pub const RSI_NO_TRAP: u64 = 0;
    pub const RSI_TRAP: u64 = 1;
}

/// RSI GIC owner constants
pub mod gic_owner {
    pub const RSI_GIC_OWNER_0: u64 = 0;
    pub const RSI_GIC_OWNER_N: u64 = 1;
}

/// PLANE ENTER flags + helpers
pub mod plane_enter_flags {
    pub const PLANE_ENTER_FLAG_TRAP_WFI: u64 = 1 << 0;
    pub const PLANE_ENTER_FLAG_TRAP_WFE: u64 = 1 << 1;
    pub const PLANE_ENTER_FLAG_TRAP_HC: u64 = 1 << 2;
    pub const PLANE_ENTER_FLAG_GIC_OWNER: u64 = 1 << 3;
    pub const PLANE_ENTER_FLAG_GIC_SIMD: u64 = 1 << 4;

    pub const PLANE_ENTER_FLAG_SIZE: u32 = 4;
    pub const PLANE_ENTER_FLAG_MASK: u64 = (1 << PLANE_ENTER_FLAG_SIZE) - 1;

    #[inline]
    pub fn set(x: &mut u64, flag: u64) {
        *x |= flag;
    }
    #[inline]
    pub fn clear(x: &mut u64, flag: u64) {
        *x &= !flag;
    }
    #[inline]
    pub fn is_set(x: u64, flag: u64) -> bool {
        (x & flag) != 0
    }
}

/// The original C header uses a `struct realm_config` whose size is 0x1000 and
/// contains a first union with either named fields (ipa_bits, hash_algo, ...)
/// or a 0x200-size pad, and a second union with rpv[64] or pad2[0xe00].
/// We'll reproduce an equivalent layout in Rust using explicit padding fields
/// and alignment to 0x1000.
///
/// Layout:
/// - offset 0x00: ipa_bits (u64)
/// - offset 0x08: hash_algo (u64)
/// - offset 0x10: num_aux_planes (u64)
/// - offset 0x18: gicv3_vtr (u64)
/// - offset 0x20: ats_plane (u64)
/// - offset 0x28..0x200: padding
/// - offset 0x200..0x240: rpv (64 bytes)
/// - offset 0x240..0x1000: padding
#[repr(C)]
#[repr(align(0x1000))]
#[derive(Debug)]
pub struct RealmConfig {
    // first union: 5 * unsigned long (assuming 8 bytes each on aarch64)
    pub ipa_bits: u64,
    pub hash_algo: u64,
    pub num_aux_planes: u64,
    pub gicv3_vtr: u64,
    pub ats_plane: u64,
    // pad to 0x200 bytes for the first union
    _pad1: [u8; 0x200 - 5 * 8],
    // second union: rpv[64] or pad2[0xe00]
    pub rpv: [u8; 64],
    _pad2: [u8; 0xe00 - 64],
}
// Sanity: size_of::<RealmConfig>() should be 0x1000
const _: () = {
    let _ = [0u8; 0]; // silent const context
    #[allow(unknown_lints)]
    #[allow(clippy::assertions_on_constants)]
    const fn _assert() {
        // We can't `assert_eq!` at compile time in stable easily, but include a no-op.
        let _ = size_of::<RealmConfig>();
    }
};

impl Default for RealmConfig {
    fn default() -> Self {
        RealmConfig {
            ipa_bits: 0,
            hash_algo: 0,
            num_aux_planes: 0,
            gicv3_vtr: 0,
            ats_plane: 0,
            _pad1: [0; 0x200 - 5 * 8],
            rpv: [0; 64],
            _pad2: [0; 0xe00 - 64],
        }
    }
}

/// plane_enter, plane_exit, plane_run in C have explicit alignments and
/// fields placed at specific offsets (with __aligned(x) between certain fields).
/// Reproducing exact offsets can be brittle; we therefore represent them
/// as repr(C) structs with explicit padding so offsets match those in the header.
///
/// The C header shows:
/// struct plane_enter {
///   0x000: u64 flags;
///   0x008: u64 pc;
///   0x100: u64 gprs[31] __aligned(0x100);
///   0x200: u64 gicv3_hcr __aligned(0x100);
///   0x208: u64 gicv3_lrs[16];
///   0x300: u64 spsr_el2 __aligned(0x100);
/// } __aligned(0x800);
///
/// We'll build paddings to place fields at the intended offsets.
#[repr(C)]
#[repr(align(0x800))]
#[derive(Debug)]
pub struct PlaneEnter {
    // 0x000
    pub flags: u64,
    pub pc: u64,
    // pad to 0x100
    _pad_to_0x100: [u8; 0x100 - 16],
    // 0x100
    pub gprs: [u64; PLANE_RUN_GPRS], // 31 * 8 = 248 bytes
    // pad to 0x200 (compute current offset: 0x100 + 248 = 0x1F8 -> need 8 bytes)
    _pad_to_0x200: [u8; 0x200 - (0x100 + PLANE_RUN_GPRS * 8)],
    // 0x200
    pub gicv3_hcr: u64,
    pub gicv3_lrs: [u64; PLANE_GIC_NUM_LRS],
    // pad to 0x300 (compute current offset)
    // offsets: 0x200 + 8 + (16*8)=0x200 + 8 + 128 = 0x280 -> need 0x300 - 0x280 = 0x80
    _pad_to_0x300: [u8; 0x300 - (0x200 + 8 + PLANE_GIC_NUM_LRS * 8)],
    // 0x300
    pub spsr_el2: u64,
    // pad to size of struct (C aligns struct to 0x800; the next fields are none).
    _pad_tail: [u8; 0x800 - (0x300 + 8)],
}

#[repr(C)]
#[repr(align(0x800))]
#[derive(Debug)]
pub struct PlaneExit {
    // 0x000
    pub reason: u8,
    // pad so that next aligned block at 0x100
    _pad0: [u8; 0x100 - 1],
    // 0x100
    pub elr_el2: u64,
    pub esr_el2: u64,
    pub far_el2: u64,
    pub hpfar_el2: u64,
    pub spsr_el2: u64,
    // pad to 0x200
    // current offset after the above (from 0x100): 5 * 8 = 40 -> end at 0x128,
    // so pad to 0x200:
    _pad_to_0x200: [u8; 0x200 - (0x100 + 5 * 8)],
    // 0x200
    pub gprs: [u64; PLANE_RUN_GPRS],
    // pad to 0x300
    _pad_to_0x300: [u8; 0x300 - (0x200 + PLANE_RUN_GPRS * 8)],
    // 0x300
    pub gicv3_hcr: u64,
    pub gicv3_lrs: [u64; PLANE_GIC_NUM_LRS],
    pub gicv3_misr: u64,
    pub gicv3_vmcr: u64,
    // pad to 0x400
    _pad_to_0x400: [u8; 0x400 - (0x300 + 8 + PLANE_GIC_NUM_LRS * 8 + 8 + 8)],
    // 0x400
    pub cntp_ctl: u64,
    pub cntp_cval: u64,
    pub cntv_ctl: u64,
    pub cntv_cval: u64,
    // tail pad to align to 0x800
    _pad_tail: [u8; 0x800 - (0x400 + 4 * 8)],
}

#[repr(C)]
#[repr(align(0x1000))]
#[derive(Debug)]
pub struct PlaneRun {
    pub enter: PlaneEnter,
    pub exit: PlaneExit,
    // pad to 0x1000 if necessary - but PlaneEnter (0x800) + PlaneExit (0x800) = 0x1000
}

// RSI exit reasons
pub mod exit_reasons {
    pub const RSI_EXIT_SYNC: u64 = 0;
    pub const RSI_EXIT_IRQ: u64 = 1;
    pub const RSI_EXIT_HOST: u64 = 2;
}

// Simple unit-like tests (compile-time shape checks)
// These are not full guarantees but give quick sanity checks at compile-time when possible.
#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn sizes() {
        // RealmConfig should be 0x1000
        assert_eq!(size_of::<RealmConfig>(), 0x1000);

        // PlaneEnter / PlaneExit should be aligned to the expected sizes (0x800 each)
        assert_eq!(size_of::<PlaneEnter>(), 0x800);
        assert_eq!(size_of::<PlaneExit>(), 0x800);
        assert_eq!(size_of::<PlaneRun>(), 0x1000);
    }

    #[test]
    fn fid_values() {
        // sanity: different SMC FIDs should not be equal
        assert_ne!(fid::SMC_RSI_ABI_VERSION, fid::SMC_RSI_FEATURES);
    }
}
