use pair::{AddLiquidityResultType, ProxyTrait as _};

use crate::config;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PairInteractions: config::Config {
    #[only_owner]
    #[endpoint]
    #[payable("*")]
    fn pair_add_initial_liquidity(&self, g_pair: TokenIdentifier) {
        let pair_address = self.get_pair_info(&g_pair).address;
        let payments = self.call_value().multi_esdt::<2>();

        // TODO Should the initial lp token be included in the pair info??
        let (_lp_payment, _first_payment_dust, _second_payment_dust) = self
            .pair_proxy_obj(pair_address)
            .add_initial_liquidity()
            .with_multi_token_transfer(ManagedVec::from_iter(payments))
            .execute_on_dest_context::<AddLiquidityResultType<Self::Api>>()
            .into_tuple();

        // TODO Should these be added to dust??
        // self.add_dust(
        //     &first_payment_dust.token_identifier,
        //     &first_payment_dust.amount,
        // );
        // self.add_dust(
        //     &second_payment_dust.token_identifier,
        //     &second_payment_dust.amount,
        // );
    }

    #[proxy]
    fn pair_proxy_obj(&self, pair_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
