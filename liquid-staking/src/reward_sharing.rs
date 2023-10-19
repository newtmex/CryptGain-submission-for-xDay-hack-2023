use multiversx_sc::{api::ManagedTypeApi, types::BigUint};

use crate::funds::DIVISION_SAFTETY_CONST;

pub struct RewardShares<M: ManagedTypeApi> {
    pub user_value: BigUint<M>,
    pub referrer_value: BigUint<M>,
    pub uru_value: BigUint<M>,
}

pub fn split_reward<M: ManagedTypeApi>(reward: &BigUint<M>) -> RewardShares<M> {
    let bonus_value = reward * 77u32 / 1_000u32; // 7.7% of reward
    let user_value = reward - &bonus_value;

    let referrer_value = &bonus_value * 4u32 / 10u32; // 40% of bonus
    let uru_value = bonus_value - &referrer_value;

    RewardShares {
        user_value,
        referrer_value,
        uru_value,
    }
}

pub fn compute_reward<M: ManagedTypeApi>(
    user_rps: &BigUint<M>,
    contract_rps: &BigUint<M>,
    delegated_egld: &BigUint<M>,
) -> BigUint<M> {
    if contract_rps <= user_rps {
        return 0u32.into();
    }

    (contract_rps - user_rps) * delegated_egld / DIVISION_SAFTETY_CONST
}
