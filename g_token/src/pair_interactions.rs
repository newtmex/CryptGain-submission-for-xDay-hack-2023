use pair::{AddLiquidityResultType, ProxyTrait as _};

use crate::config;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PairInteractions: config::Config {
    #[only_owner]
    #[endpoint]
    #[payable("*")]
    fn pair_add_initial_liquidity(&self, g_pair: TokenIdentifier) {
        let pair_info = self.get_pair_info(&g_pair);
        let payments = self.call_value().multi_esdt::<2>();

        require!(
            payments[0].token_identifier == pair_info.tokens.first_token_id,
            "Invalid first token payment"
        );
        require!(
            payments[1].token_identifier == pair_info.tokens.second_token_id,
            "Invalid second token payment"
        );

        let (lp_payment, _first_payment_optimal, second_payment_optimal) = self
            .pair_proxy_obj(pair_info.address)
            .add_initial_liquidity()
            .with_multi_token_transfer(ManagedVec::from_iter(payments))
            .execute_on_dest_context::<AddLiquidityResultType<Self::Api>>()
            .into_tuple();

        // Update GToken supplies and distribution values
        let g_payment = self.g_token().mint(second_payment_optimal.amount);

        self.add_g_supply(g_pair, &g_payment.amount, lp_payment.amount);
        self.add_dust(&g_payment.token_identifier, g_payment.amount);
    }

    #[proxy]
    fn pair_proxy_obj(&self, pair_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
