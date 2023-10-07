use std::ops::Mul;

use multiversx_sc_scenario::scenario_model::{
    Account, BytesKey, BytesValue, CheckAccount, CheckAccounts, CheckEsdtMap, CheckStateStep,
    CheckStorage, CheckStorageDetails, CheckValue,
};

pub fn big_num_pow_18(num: u32) -> num_bigint::BigUint {
    num_bigint::BigUint::from(10u32).pow(18).mul(num)
}

pub fn update_sc_acc<K, V>(code: &BytesValue, storage_values: Vec<(K, V)>) -> Account
where
    K: Into<BytesKey>,
    V: Into<BytesValue>,
{
    let mut account_state = Account::new().update(true).code(code);

    for (key, value) in storage_values {
        account_state.storage.insert(key.into(), value.into());
    }

    account_state
}

pub fn check_account_allow_other_storages(owner: &str) -> CheckAccount {
    CheckAccount {
        owner: CheckValue::Equal(owner.into()),
        esdt: CheckEsdtMap::Star,
        storage: CheckStorage::Equal(CheckStorageDetails {
            other_storages_allowed: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub fn check_step() -> CheckStateStep {
    CheckStateStep {
        accounts: CheckAccounts {
            other_accounts_allowed: true,
            ..Default::default()
        },
        ..Default::default()
    }
}
