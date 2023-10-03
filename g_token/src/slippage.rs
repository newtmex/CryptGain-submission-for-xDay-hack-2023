// Slippage is set at 1.5%
#![allow(clippy::inconsistent_digit_grouping)]

use multiversx_sc::{api::ManagedTypeApi, types::BigUint};

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
