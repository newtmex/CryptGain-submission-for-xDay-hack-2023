#![no_std]
#![feature(trait_alias)]

multiversx_sc::imports!();

use dsc_module::proxy::ProxyTrait as _;
use funds::DIVISION_SAFTETY_CONST;

pub mod akf_interaction;
pub mod delegation_interaction;
pub mod funds;
pub mod liquidity_pool;
pub mod storage_cache;

use crate::{
    akf_interaction::akf::ProxyTrait as _, delegation_interaction::delegation::ProxyTrait as _,
    storage_cache::StorageCache,
};

// 0.1 eGLD
pub const MIN_EGLD_TO_DELEGATE: u64 = 100_000_000_000_000_000;
pub const DELEGATE_ACTION_GAS: u64 = 12_000_000;

pub type AddLiquidityResultType<M> = MultiValue2<BigUint<M>, EsdtTokenPayment<M>>;

#[multiversx_sc::contract]
pub trait LiquidStaking:
    dsc_module::DelegationModule
    + liquidity_pool::LiquidityPoolModule
    + funds::FundsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + akf_interaction::AkfInteraction
    + delegation_interaction::DelegationInteraction
{
    #[init]
    fn init(
        &self,
        dsc_addr: &ManagedAddress,
        akf_addr: &ManagedAddress,
        delegation_proxy_addr: &ManagedAddress,
    ) {
        self.set_dsc_address(dsc_addr);
        self.set_akf_address(akf_addr);
        self.set_delegation_address(delegation_proxy_addr);
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(addLiquidity)]
    fn add_liquidity(&self, staked_egld: BigUint) -> AddLiquidityResultType<Self::Api> {
        let owner = self.blockchain().get_owner_address();

        let egld_sent = self.call_value().egld_value().clone_value();
        require!(
            staked_egld >= MIN_EGLD_TO_DELEGATE,
            "Insufficient staked eGLD"
        );
        require!(staked_egld <= egld_sent, "Invalid egld amount sent");

        let mut storage_cache = StorageCache::new(self);
        require!(
            storage_cache.ls_token_id.is_valid_esdt_identifier(),
            "ls token invalid"
        );

        let ls_token = self.pool_add_liquidity(staked_egld, egld_sent, &mut storage_cache);
        self.send().direct_esdt(
            &owner,
            &ls_token.token_identifier,
            ls_token.token_nonce,
            &ls_token.amount,
        );

        let result = (storage_cache.reward_per_share.clone(), ls_token);

        result.into()
    }

    #[only_owner]
    #[endpoint]
    fn claim_reward(
        &self,
        user_addr: &ManagedAddress,
        aku_id: u64,
        rps: &BigUint,
        delegated_egld: &BigUint,
        referrer: OptionalValue<ManagedAddress>,
    ) -> BigUint {
        let gas_limit = 50_000_000;
        require!(
            self.blockchain().get_gas_left() >= gas_limit,
            "not enough gas to initiate claim"
        );

        self.try_claim_from_delegation_proxy();

        let current_rps = self.reward_per_share().get();

        if &current_rps > rps {
            let mut reward = (&current_rps - rps) * delegated_egld / DIVISION_SAFTETY_CONST;
            self.rewards_reserve().update(|reserve| {
                if *reserve >= reward {
                    // 7.7% of reward
                    let bonus_value = &reward * 77u32 / 1_000u32;
                    let user_value = &reward - &bonus_value;

                    {
                        // 40% of bonus
                        let referrer_value = &bonus_value * 4u32 / 10u32;
                        let uru_value = bonus_value - &referrer_value;

                        if uru_value > 0 {
                            let () = self
                                .call_akf()
                                .add_uru_egld(aku_id)
                                .with_egld_transfer(uru_value)
                                .execute_on_dest_context();
                        }

                        if let Some(referrer) = referrer.into_option() {
                            if referrer_value > 0 {
                                self.send().direct_egld(&referrer, &referrer_value);
                            }
                        } else {
                            // Add value to delegation
                            self.pending_delegation()
                                .update(|bal| *bal += referrer_value);
                        }
                    }

                    self.send().direct_egld(user_addr, &user_value);

                    *reserve -= core::mem::take(&mut reward);
                }
            });

            if reward > 0 {
                return rps.clone();
            }
        }

        current_rps
    }

    #[endpoint]
    #[payable("*")]
    #[only_owner]
    fn remove_liquidity(&self, _caller: &ManagedAddress, egld_for_plv: &BigUint) {
        let storage_cache = StorageCache::new(self);
        let payment = self.call_value().single_esdt();

        require!(
            storage_cache.ls_token_id.is_valid_esdt_identifier(),
            "ERROR_LS_TOKEN_NOT_ISSUED"
        );
        require!(
            payment.token_identifier == storage_cache.ls_token_id,
            "ERROR_BAD_PAYMENT_TOKEN"
        );
        require!(payment.amount > 0, "ERROR_BAD_PAYMENT_AMOUNT");
        require!(
            egld_for_plv <= &payment.amount,
            "PLV amount is larger than unDelegate amount"
        );

        // let egld_to_unstake = self.pool_remove_liquidity(&payment.amount, &mut storage_cache);
        // // TODO
        // // require!(
        // //     egld_to_unstake >= MIN_EGLD_TO_DELEGATE,
        // //     ERROR_INSUFFICIENT_UNSTAKE_AMOUNT
        // // );
        // self.burn_ls_token(&payment.amount);

        //    TODO Send unstake tokkens
    }

    #[endpoint]
    fn delegate(&self) {
        let mut storage_cache = StorageCache::new(self);

        if is_enough_to_delegate(&storage_cache.pending_delegation) {
            let delegate_amount = core::mem::take(&mut storage_cache.pending_delegation);
            storage_cache.commit();

            let gas_limit = self.blockchain().get_gas_left();
            let extra_gas = 2_000_000;
            require!(
                gas_limit >= (DELEGATE_ACTION_GAS + extra_gas),
                "not enough gas for delegate"
            );

            self.call_dsc()
                .delegate()
                .with_gas_limit(DELEGATE_ACTION_GAS)
                .with_egld_transfer(delegate_amount.clone())
                .async_call()
                .with_callback(LiquidStaking::callbacks(self).delegate_callback(delegate_amount))
                .call_and_exit();
        }
    }

    #[callback]
    fn delegate_callback(
        &self,
        delegate_amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {}
            ManagedAsyncCallResult::Err(_) => {
                self.pending_delegation()
                    .update(|old| *old += delegate_amount);
            }
        }
    }

    fn try_claim_from_delegation_proxy(&self) {
        self.last_claim_epoch().update(|last_epoch| {
            let current_epoch = self.blockchain().get_block_epoch();

            if *last_epoch < current_epoch {
                *last_epoch = current_epoch;

                let gas_limit = 30_000_000;
                require!(
                    self.blockchain().get_gas_left() > gas_limit,
                    "not enough gas to initiate delegation call"
                );

                self.call_delegation()
                    .claim_rewards()
                    .with_gas_limit(gas_limit)
                    .transfer_execute();
            }
        });
    }

    #[view]
    fn is_enough_to_delegate(&self) -> bool {
        is_enough_to_delegate(&self.pending_delegation().get())
    }

    #[storage_mapper("last_claim_epoch")]
    fn last_claim_epoch(&self) -> SingleValueMapper<u64>;
}

fn is_enough_to_delegate<M: ManagedTypeApi>(amt: &BigUint<M>) -> bool {
    let one_egld = BigUint::from(10u32).pow(18);

    *amt >= one_egld
}
