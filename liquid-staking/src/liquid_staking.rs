#![no_std]
#![feature(trait_alias)]

multiversx_sc::imports!();
use dsc_module::proxy::ProxyTrait as _;

pub mod funds;
pub mod liquidity_pool;
pub mod storage_cache;

mod self_proxy {
    #[multiversx_sc::proxy]
    pub trait SelfProxy {
        #[endpoint]
        fn delegate(&self);
    }
}

use crate::storage_cache::StorageCache;

// 0.001 eGLD
pub const MIN_EGLD_TO_DELEGATE: u64 = 1_000_000_000_000_000;

pub type AddLiquidityResultType<M> = MultiValue2<BigUint<M>, EsdtTokenPayment<M>>;

#[multiversx_sc::contract]
pub trait LiquidStaking:
    dsc_module::DelegationModule
    + liquidity_pool::LiquidityPoolModule
    + funds::FundsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self, dsc_addr: &ManagedAddress) {
        self.set_dsc_address(dsc_addr);
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

        if is_enough_to_delegate(&storage_cache.pending_delegation) {
            storage_cache.commit();

            let sc_own_addr = self.blockchain().get_sc_address();
            self.self_proxy()
                .contract(sc_own_addr)
                .delegate()
                .with_gas_limit(30_000_000)
                .transfer_execute();
        }

        result.into()
    }

    #[endpoint]
    fn delegate(&self) {
        let mut storage_cache = StorageCache::new(self);
        
        if is_enough_to_delegate(&storage_cache.pending_delegation) {
            let delegate_amount = core::mem::take(&mut storage_cache.pending_delegation);
            storage_cache.commit();

            let gas_limit = self.blockchain().get_gas_left();
            require!(gas_limit >= 25_000_000, "not enough gas for delegate");

            self.call_dsc()
                .delegate()
                .with_gas_limit(gas_limit)
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
            ManagedAsyncCallResult::Ok(()) => {},
            ManagedAsyncCallResult::Err(_) => {
                self.pending_delegation()
                    .update(|old| *old += delegate_amount);
            },
        }
    }

    #[proxy]
    fn self_proxy(&self) -> self_proxy::Proxy<Self::Api>;
}

fn is_enough_to_delegate<M: ManagedTypeApi>(amt: &BigUint<M>) -> bool {
    let one_egld = BigUint::from(10u32).pow(18);

    *amt >= one_egld
}
