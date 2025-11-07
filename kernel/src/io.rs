// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 SUSE LLC
//
// Author: Joerg Roedel <jroedel@suse.de>

use crate::error::SvsmError;
// use core::arch::asm;
use core::fmt::Debug;

pub trait IOPort: Sync + Debug {
    fn outb(&self, port: u16, value: u8) {
        // SAFETY: Inline assembly to write an ioport, which does not change
        // any state related to memory safety.
        // unsafe { asm!("outb %al, %dx", in("x0") value, in("x3") port, /*options(att_syntax)*/) }
    }

    fn inb(&self, port: u16) -> u8 {
        // SAFETY: Inline assembly to read an ioport, which does not change
        // any state related to memory safety.
        unsafe {
            let ret: u8 = 0;
            // asm!("inb %dx, %al", in("x3") port, out("x0") ret, /*options(att_syntax)*/);
            ret
        }
    }

    fn outw(&self, port: u16, value: u16) {
        // SAFETY: Inline assembly to write an ioport, which does not change
        // any state related to memory safety.
        // unsafe { asm!("outw %ax, %dx", in("x0") value, in("x3") port, /*options(att_syntax)*/) }
    }

    fn inw(&self, port: u16) -> u16 {
        // SAFETY: Inline assembly to read an ioport, which does not change
        // any state related to memory safety.
        unsafe {
            let ret: u16 = 0;
            // asm!("inw %dx, %ax", in("x3") port, out("x0") ret, /*options(att_syntax)*/);
            ret
        }
    }

    fn outl(&self, port: u16, value: u32) {
        // SAFETY: Inline assembly to write an ioport, which does not change
        // any state related to memory safety.
        // unsafe { asm!("outl %eax, %dx", in("x0") value, in("x3") port, /*options(att_syntax)*/) }
    }

    fn inl(&self, port: u16) -> u32 {
        // SAFETY: Inline assembly to read an ioport, which does not change
        // any state related to memory safety.
        unsafe {
            let ret: u32 = 0;
            // asm!("inl %dx, %eax", in("x3") port, out("x0") ret, /*options(att_syntax)*/);
            ret
        }
    }
}

/*
arm通过MMIO实现对外设的访问
pub trait IOPort: Sync + Debug {
    /// Write one byte to the given MMIO address
    fn outb(&self, addr: u64, value: u8) {
        unsafe {
            asm!(
                "strb {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = in(reg) value,
                options(nostack, preserves_flags),
            );
        }
    }

    /// Read one byte from the given MMIO address
    fn inb(&self, addr: u64) -> u8 {
        let value: u8;
        unsafe {
            asm!(
                "ldrb {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = out(reg) value,
                options(nostack, preserves_flags),
            );
        }
        value
    }

    /// Write a 16-bit value
    fn outw(&self, addr: u64, value: u16) {
        unsafe {
            asm!(
                "strh {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = in(reg) value,
                options(nostack, preserves_flags),
            );
        }
    }

    /// Read a 16-bit value
    fn inw(&self, addr: u64) -> u16 {
        let value: u16;
        unsafe {
            asm!(
                "ldrh {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = out(reg) value,
                options(nostack, preserves_flags),
            );
        }
        value
    }

    /// Write a 32-bit value
    fn outl(&self, addr: u64, value: u32) {
        unsafe {
            asm!(
                "str {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = in(reg) value,
                options(nostack, preserves_flags),
            );
        }
    }

    /// Read a 32-bit value
    fn inl(&self, addr: u64) -> u32 {
        let value: u32;
        unsafe {
            asm!(
                "ldr {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = out(reg) value,
                options(nostack, preserves_flags),
            );
        }
        value
    }
}
*/

#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultIOPort {}

impl IOPort for DefaultIOPort {}

pub static DEFAULT_IO_DRIVER: DefaultIOPort = DefaultIOPort {};

/// Generic Read trait to be implemented over any transport channel when reading multiple bytes.
pub trait Read {
    type Err: Into<SvsmError>;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Err>;
}

/// Generic Write trait to be implemented over any transport channel when writing multiple bytes.
pub trait Write {
    type Err: Into<SvsmError>;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Err>;
}

