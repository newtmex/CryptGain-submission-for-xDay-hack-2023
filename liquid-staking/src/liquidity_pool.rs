use crate::{funds, storage_cache::StorageCache};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait LiquidityPoolModule:
    funds::FundsModule + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint]
    fn register_ls_token(&self, token_display_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        let payment_amount = self.call_value().egld_value().clone_value();
        self.ls_token().issue_and_set_all_roles(
            payment_amount,
            token_display_name,
            token_ticker,
            18,
            None,
        );
    }

    fn pool_add_liquidity(
        &self,
        egld_staked: BigUint,
        egld_sent: BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> EsdtTokenPayment {
        let ls_token = self.ls_token().mint(egld_staked);

        storage_cache.ls_token_supply += &ls_token.amount;
        storage_cache.delegated_egld += &egld_sent;
        storage_cache.pending_delegation += egld_sent;

        ls_token
    }

    #[view(getLsTokenId)]
    #[storage_mapper("ls_token")]
    fn ls_token(&self) -> FungibleTokenMapper<Self::Api>;

    #[storage_mapper("delegated_egld")]
    fn delegated_egld(&self) -> SingleValueMapper<BigUint>;
}
