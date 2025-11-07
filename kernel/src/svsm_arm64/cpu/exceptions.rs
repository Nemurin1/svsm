use crate::print;
use crate::svsm_arm64::uart_console;
use core::arch::global_asm;

global_asm!(include_str!("exceptions.s"));

#[no_mangle]
pub extern "C" fn irq_entry_from_asm(irq_id: u64) {
    // This function is called from assembly with irq_id in x0.
    // Note: IRQ id may include some special values; real code must check spurious ID.
    print!(">>> IRQ received: ");
    // print decimal (simple)
    let mut buf = [0u8; 20];
    let n = u64_to_dec(irq_id, &mut buf);
    let s = core::str::from_utf8(&buf[..n]).unwrap_or("?");
    print!("{}", s);
    print!("\n");
}

fn u64_to_dec(mut v: u64, out: &mut [u8]) -> usize {
    if v == 0 {
        out[0] = b'0';
        return 1;
    }
    let mut i = out.len();
    while v > 0 && i > 0 {
        i -= 1;
        out[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    let start = i;
    let len = out.len() - start;
    out.copy_within(start..out.len(), 0);
    len
}
