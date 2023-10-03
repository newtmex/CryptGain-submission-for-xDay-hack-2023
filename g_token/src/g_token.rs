#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use pair::{
    AddLiquidityResultType, ProxyTrait as _, SwapTokensFixedInputResultType,
    SwapTokensFixedOutputResultType,
};

pub mod config;
pub mod pair_interactions;
pub mod router_interaction;
pub mod slippage;

pub const MIN_MINT_DEPOSIT: u64 = 4_000;

#[multiversx_sc::contract]
pub trait GToken:
    pair_interactions::PairInteractions + config::Config + router_interaction::RouterInteraction
{
    #[init]
    fn init(&self, router_addr: ManagedAddress, base_pair_id: TokenIdentifier) {
        self.set_router_addr(router_addr);
        self.set_base_pair(base_pair_id);
    }

    #[only_owner]
    #[endpoint(registerGToken)]
    #[payable("EGLD")]
    fn register_g_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value();
        self.g_token().issue_and_set_all_roles(
            payment_amount.clone_value(),
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[endpoint]
    #[payable("*")]
    fn mint(&self, opt_g_pair: OptionalValue<TokenIdentifier>) {
        let caller = self.blockchain().get_caller();
        let base_pair_id = self.base_pair().get_token_id();

        // Set payment swap amount
        let mut sent_payment = self.call_value().single_esdt();
        require!(
            sent_payment.amount >= MIN_MINT_DEPOSIT,
            "Insufficient liquidity deposit"
        );
        sent_payment.amount /= 2u64;

        let g_pair_id = opt_g_pair
            .into_option()
            .unwrap_or_else(|| sent_payment.token_identifier.clone());
        require!(
            g_pair_id != base_pair_id,
            "Specify GPair ID when minting with base pair"
        );

        let pair_info = self.get_pair_info(&g_pair_id);
        let call_pair = || self.pair_proxy_obj(pair_info.address.clone());

        // Compute other pair payment
        let mut amount_out: BigUint<Self::Api> = call_pair()
            .get_amount_out_view(&sent_payment.token_identifier, &sent_payment.amount)
            .execute_on_dest_context();
        slippage::apply(&mut amount_out);

        let (first_payment, second_payment) = if sent_payment.token_identifier == base_pair_id {
            let g_pair_payment = call_pair()
                .swap_tokens_fixed_input(&g_pair_id, amount_out)
                .with_esdt_transfer(sent_payment.clone())
                .execute_on_dest_context::<SwapTokensFixedInputResultType<Self::Api>>();

            (g_pair_payment, sent_payment)
        } else {
            let (base_pair_payment, g_pair_payment_dust) = call_pair()
                .swap_tokens_fixed_output(&base_pair_id, amount_out)
                .with_esdt_transfer(sent_payment.clone())
                .execute_on_dest_context::<SwapTokensFixedOutputResultType<Self::Api>>()
                .into_tuple();

            self.add_dust(
                &g_pair_payment_dust.token_identifier,
                g_pair_payment_dust.amount,
            );

            (sent_payment, base_pair_payment)
        };

        let first_token_sent_amt = first_payment.amount.clone();
        let second_token_sent_amt = second_payment.amount.clone();

        let first_token_amount_min = slippage::from_ref(&first_payment.amount);
        let second_token_amount_min = call_pair()
            .get_equivalent(&first_payment.token_identifier, &first_token_amount_min)
            .execute_on_dest_context::<BigUint<Self::Api>>();
        let (
            //
            lp_payment,
            first_payment_optimal,
            second_payment_optimal,
        ) = call_pair()
            .add_liquidity(first_token_amount_min, second_token_amount_min)
            .with_multi_token_transfer(ManagedVec::from_iter([first_payment, second_payment]))
            .execute_on_dest_context::<AddLiquidityResultType<Self::Api>>()
            .into_tuple();

        let (first_token_for_position, second_token_for_position) = call_pair()
            .get_tokens_for_given_position(&lp_payment.amount)
            .execute_on_dest_context::<MultiValue2<EsdtTokenPayment, EsdtTokenPayment>>()
            .into_tuple();

        let (g_payment, mint_fee) = {
            // g_token_amount to mint is based on the lp position amount of the base pair for the amount of lp tokens received
            let g_token_amount = if first_token_for_position.token_identifier == base_pair_id {
                first_token_for_position
            } else if second_token_for_position.token_identifier == base_pair_id {
                second_token_for_position
            } else {
                sc_panic!("invalid position tokens received")
            }
            .amount;
            let mut g_token_payment = self.g_token().mint(g_token_amount);

            // Update GToken supplies and distribution values
            let user_amt = self.add_g_supply(g_pair_id, &g_token_payment.amount, lp_payment.amount);
            drop(pair_info); // to use after this point, request from storage again;

            let mint_fee = &g_token_payment.amount - &user_amt;
            g_token_payment.amount = user_amt;

            (g_token_payment, mint_fee)
        };

        // Update dust for all
        self.add_dust(
            &first_payment_optimal.token_identifier,
            first_token_sent_amt - first_payment_optimal.amount,
        );
        self.add_dust(
            &second_payment_optimal.token_identifier,
            second_token_sent_amt - second_payment_optimal.amount,
        );
        self.add_dust(&g_payment.token_identifier, mint_fee);

        self.send()
            .direct_esdt(&caller, &g_payment.token_identifier, 0, &g_payment.amount);
    }
}
