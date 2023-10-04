#![allow(clippy::inconsistent_digit_grouping)]

use multiversx_sc::{
    api::{ErrorApiImpl, ManagedTypeApi},
    types::BigUint,
};

/// Slippage is set at 1.5%
const SLIPPAGE_NUMERATOR: u64 = 98_50;
const SLIPPAGE_DENOMINATOR: u64 = 100_00;

pub fn apply<M: ManagedTypeApi>(amount: &mut BigUint<M>) {
    *amount *= SLIPPAGE_NUMERATOR;
    *amount /= SLIPPAGE_DENOMINATOR;
}

pub fn from_ref<M: ManagedTypeApi>(amount: &BigUint<M>) -> BigUint<M> {
    let value = amount * SLIPPAGE_NUMERATOR;
    value / SLIPPAGE_DENOMINATOR
}

pub fn from_ref_user_defined<M: ManagedTypeApi>(amount: &BigUint<M>, slippage: u64) -> BigUint<M> {
    if !(1..SLIPPAGE_DENOMINATOR).contains(&slippage) {
        M::error_api_impl().signal_error(b"invalid slippage value")
    }

    let slippage_numerator = SLIPPAGE_DENOMINATOR - slippage;

    let value = amount * slippage_numerator;
    value / SLIPPAGE_DENOMINATOR
}
