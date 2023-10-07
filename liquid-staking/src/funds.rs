multiversx_sc::imports!();

pub const DIVISION_SAFTETY_CONST: u64 = 1_000_000_000;

#[multiversx_sc::module]
pub trait FundsModule {
    #[payable("EGLD")]
    #[endpoint(topUpRewards)]
    fn top_up_rewards(&self) {
        let reward_increase = self.call_value().egld_value().clone_value();
        let total_delegated = self.ls_token_supply().get();

        if reward_increase > 0 && total_delegated > 0 {
            let rps_increase = (&reward_increase * DIVISION_SAFTETY_CONST) / total_delegated;

            self.reward_per_share().update(|rps| *rps += rps_increase);
            self.rewards_reserve()
                .update(|reserve| *reserve += reward_increase);
        }
    }

    #[storage_mapper("pending_delegation")]
    fn pending_delegation(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("rewards_reserve")]
    fn rewards_reserve(&self) -> SingleValueMapper<BigUint>;

    #[view(getLsSupply)]
    #[storage_mapper("ls_token_supply")]
    fn ls_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[view(getRewardPerShare)]
    #[storage_mapper("reward_per_share")]
    fn reward_per_share(&self) -> SingleValueMapper<BigUint>;
}
