use crate::realm::rsi::{RealmConfig};
use crate::realm::rsi::smccc::{ArmSmcccRes, arm_smccc_smc};
use crate::realm::rsi::fid::*;
use crate::realm::rsi::retcodes::*;
use crate::error::SvsmError;
use crate::utils::immut_after_init::ImmutAfterInitCell;

pub static REALM_CONFIG: ImmutAfterInitCell<RealmConfig> = ImmutAfterInitCell::uninit();

pub fn rsi_realm_config() -> RealmConfig {
    let mut res = ArmSmcccRes {
        a0: 0,
        a1: 0,
        a2: 0,
        a3: 0,
    };

    let mut config = RealmConfig::default();

    unsafe {
        // call SMC
        arm_smccc_smc(
            SMC_RSI_REALM_CONFIG,                   // FID
            &mut config as *mut RealmConfig as u64, // arg1 = RealmConfig 地址
            0, 0, 0, 0, 0, 0,
            &mut res as *mut ArmSmcccRes,
            core::ptr::null_mut(),
        );
    }

    // 可以检查 res.a0 判断是否成功
    if res.a0 != RSI_SUCCESS {
        panic!("Failed to read RealmConfig via SMC: {}", res.a0);
    }

    config
}

// This function can only call once
pub fn init_realm_config() -> Result<(), SvsmError> {
    let cfg = rsi_realm_config();
    REALM_CONFIG.init(cfg);
    Ok(())
}