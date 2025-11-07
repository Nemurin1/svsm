// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 SUSE LLC
//
// Author: Joerg Roedel <jroedel@suse.de>

use core::arch::asm;

pub const EFER: u32 = 0xC000_0080;
pub const SEV_STATUS: u32 = 0xC001_0131;
pub const SEV_GHCB: u32 = 0xC001_0130;
pub const MSR_GS_BASE: u32 = 0xC000_0101;

pub fn read_msr(msr: u32) -> u64 {
    let eax: u32;
    let edx: u32;

    // SAFETY: Inline assembly to read the specified MSR. It does not change
    // any state.
    unsafe {
        /*
        asm!("rdmsr",
             in("x2") msr,
             out("x0") eax,
             out("x3") edx,
             //options(att_syntax)
            );
        */
        asm!("nop");
    }
    // (eax as u64) | (edx as u64) << 32
    0
}

/// # Safety
///
/// The caller should ensure that the new value in the target MSR doesn't break
/// memory safety.
pub unsafe fn write_msr(msr: u32, val: u64) {
    /*
    let eax = val as u32;
    let edx = (val >> 32) as u32;

    // SAFETY: requirements have to be checked by the caller.
    unsafe {
        asm!("wrmsr",
             in("x2") msr,
             in("x0") eax,
             in("x3") edx,
             //options(att_syntax)
            );
    }
    */
}

pub fn rdtsc() -> u64 {
    /*
    let eax: u32;
    let edx: u32;

    // SAFETY: Inline assembly to read the TSC. It does not change any state.
    unsafe {
        asm!("rdtsc",
             out("x0") eax,
             out("x3") edx,
             /* options(att_syntax, nomem, nostack) */);
    }
    (eax as u64) | (edx as u64) << 32
    */
    let cnt: u64;
    unsafe {
        // CNTVCT_EL0 holds a 64-bit virtual count (ARM Generic Timer).
        asm!("mrs {cnt}, CNTVCT_EL0", cnt = out(reg) cnt, options(nomem, nostack, preserves_flags));
    }
    cnt
}

#[derive(Debug, Clone, Copy)]
pub struct RdtscpOut {
    pub timestamp: u64,
    pub pid: u32,
}

pub fn rdtscp() -> RdtscpOut {
    /*
    let eax: u32;
    let edx: u32;
    let ecx: u32;

    // SAFETY: Inline assembly to read the TSC and PID. It does not change
    // any state.
    unsafe {
        asm!("rdtscp",
             out("x0") eax,
             out("x2") ecx,
             out("x3") edx,
             /* options(att_syntax, nomem, nostack) */);
    }
    RdtscpOut {
        timestamp: (eax as u64) | (edx as u64) << 32,
        pid: ecx,
    }
    */
    let timestamp = rdtsc();
    // No direct equivalent of x86's IA32_TSC_AUX/rdtscp pid; return 0.
    RdtscpOut {
        timestamp,
        pid: 0,
    }
}

pub fn read_flags() -> u64 {
    /*
    let rax: u64;
    // SAFETY: Inline assembly to read the EFLAGS register. It does not change
    // any state.
    unsafe {
        asm!(
            r#"
                pushfq
                pop     %rax
            "#,
             out("x0") rax,
             //options(att_syntax)
            );
    }
    rax
    */
    let nzcv: u64;
    unsafe {
        // Read the NZCV condition flags into nzcv (lower bits contain flags).
        asm!("mrs {nzcv}, NZCV", nzcv = out(reg) nzcv, options(nomem, nostack, preserves_flags));
    }
    nzcv
}
