#![allow(clippy::inconsistent_digit_grouping)]

use multiversx_sc::{
    api::{ErrorApi, ErrorApiImpl, ManagedTypeApi},
    types::BigUint,
};

const MAX_PERCENTAGE: u64 = 100_00;
fn check_slippage<Err: ErrorApi>(slippage: u64) -> u64 {
    if !(1..MAX_PERCENTAGE).contains(&slippage) {
        Err::error_api_impl().signal_error(b"invalid slippage value")
    }

    MAX_PERCENTAGE - slippage
}

pub fn apply<M: ManagedTypeApi>(amount: &mut BigUint<M>, slippage: u64) {
    *amount *= check_slippage::<M>(slippage);
    *amount /= MAX_PERCENTAGE;
}

pub fn from_ref<M: ManagedTypeApi>(amount: &BigUint<M>, slippage: u64) -> BigUint<M> {
    let value = amount * check_slippage::<M>(slippage);
    value / MAX_PERCENTAGE
}
