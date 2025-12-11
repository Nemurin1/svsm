use core::arch::global_asm;

const EL1_SP0_SYNC: &'static str = "EL1_SP0_SYNC";

global_asm!(include_str!("exceptions.s"));

#[repr(C)]
#[derive(Debug)]
pub struct ExceptionCtx {
    regs: [u64; 30],
    elr_el1: u64,
    spsr_el1: u64,
    lr: u64,
}

// print exception data
fn catch(ctx: &mut ExceptionCtx, name: &str) {
    
    log::info!("Exception: {}   ELR_EL1: 0x{:016x}", name, ctx.elr_el1);

    // enter endless loop
    loop {

    }
}

#[no_mangle]
unsafe extern "C" fn el1_sp0_sync(ctx: &mut ExceptionCtx) {
    catch(ctx, EL1_SP0_SYNC);
}