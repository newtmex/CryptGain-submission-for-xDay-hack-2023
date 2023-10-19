use pair::{
    AddLiquidityResultType, ProxyTrait as _, SwapTokensFixedInputResultType,
    SwapTokensFixedOutputResultType,
};

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

        self.add_g_supply(g_pair, &g_payment.amount, lp_payment);
        self.add_dust(&g_payment.token_identifier, g_payment.amount);

        // TODO re enable when this is enabled on router contract
        // let _: IgnoreValue = self
        //     .pair_proxy_obj(pair_info.address)
        //     .resume()
        //     .execute_on_dest_context();
    }

    fn pair_swap_fixed_input(
        &self,
        token_out: &TokenIdentifier,
        amount_out_min: BigUint,
        payment: EsdtTokenPayment,
        call_pair: impl Fn() -> pair::Proxy<Self::Api>,
    ) -> SwapTokensFixedInputResultType<Self::Api> {
        call_pair()
            .swap_tokens_fixed_input(token_out, amount_out_min)
            .with_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    fn pair_swap_fixed_output(
        &self,
        token_out: &TokenIdentifier,
        amount_out: BigUint,
        payment: EsdtTokenPayment,
        call_pair: impl Fn() -> pair::Proxy<Self::Api>,
    ) -> SwapTokensFixedOutputResultType<Self::Api> {
        call_pair()
            .swap_tokens_fixed_output(token_out, amount_out)
            .with_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    fn pair_get_tokens_for_given_position(
        &self,
        amount: &BigUint,
        call_pair: impl Fn() -> pair::Proxy<Self::Api>,
    ) -> MultiValue2<EsdtTokenPayment, EsdtTokenPayment> {
        call_pair()
            .get_tokens_for_given_position(amount)
            .execute_on_dest_context::<MultiValue2<EsdtTokenPayment, EsdtTokenPayment>>()
    }

    #[proxy]
    fn pair_proxy_obj(&self, pair_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
