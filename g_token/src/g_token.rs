#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use pair::{AddLiquidityResultType, ProxyTrait as _};
use router::factory::PairTokens;

pub mod config;
pub mod pair_interactions;
pub mod router_interaction;
pub mod slippage;

pub const MIN_MINT_DEPOSIT: u64 = 4_000;

#[derive(TopDecode, TopEncode, NestedDecode, NestedEncode)]
pub struct PairInfo<M: ManagedTypeApi> {
    pub tokens: PairTokens<M>,
    pub address: ManagedAddress<M>,
}

#[multiversx_sc::contract]
pub trait GToken:
    pair_interactions::PairInteractions + config::Config + router_interaction::RouterInteraction
{
    #[init]
    fn init(&self, router_addr: ManagedAddress, base_pair_id: TokenIdentifier) {
        self.set_router_addr(router_addr);

        if self.base_pair().is_empty() {
            self.base_pair().set_token_id(base_pair_id);
        }
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
    fn mint(&self) {
        let caller = self.blockchain().get_caller();
        let base_pair_id = self.base_pair().get_token_id();

        // Set payment swap amount
        let mut g_pair_payment = self.call_value().single_esdt();
        require!(
            g_pair_payment.amount >= MIN_MINT_DEPOSIT,
            "Insufficient liquidity deposit"
        );
        g_pair_payment.amount /= 2u64;

        // TODO add scenario for base pair token
        let pair_info = self.get_pair_info(&g_pair_payment.token_identifier);
        let call_pair = || self.pair_proxy_obj(pair_info.address.clone());

        // Compute base pair payment
        let base_pair_payment = {
            let mut amount_out_min: BigUint<Self::Api> = call_pair()
                .get_amount_out_view(&g_pair_payment.token_identifier, &g_pair_payment.amount)
                .execute_on_dest_context();
            slippage::apply(&mut amount_out_min);

            call_pair()
                .swap_tokens_fixed_input(&base_pair_id, amount_out_min)
                .with_esdt_transfer(g_pair_payment.clone())
                .execute_on_dest_context::<EsdtTokenPayment<Self::Api>>()
        };

        let (first_send_payment, second_send_payment) = {
            let mut first_payment = g_pair_payment;
            let mut second_payment = base_pair_payment;
            if pair_info.tokens.first_token_id != first_payment.token_identifier {
                core::mem::swap(&mut first_payment, &mut second_payment);
            }
            (first_payment, second_payment)
        };
        let first_token_amount_min = slippage::from_ref(&first_send_payment.amount);
        let second_token_amount_min = call_pair()
            .get_equivalent(
                &first_send_payment.token_identifier,
                &first_token_amount_min,
            )
            .execute_on_dest_context::<BigUint<Self::Api>>();
        let first_send_amt = first_send_payment.amount.clone();
        let second_send_amt = second_send_payment.amount.clone();

        let (lp_payment, first_payment_dust, second_payment_dust) = call_pair()
            .add_liquidity(first_token_amount_min, second_token_amount_min)
            .with_multi_token_transfer(ManagedVec::from_iter([
                first_send_payment,
                second_send_payment,
            ]))
            .execute_on_dest_context::<AddLiquidityResultType<Self::Api>>()
            .into_tuple();

        let (first_token_for_position, second_token_for_position) = call_pair()
            .get_tokens_for_given_position(&lp_payment.amount)
            .execute_on_dest_context::<MultiValue2<EsdtTokenPayment, EsdtTokenPayment>>()
            .into_tuple();

        // g_token_amount to mint is based on the lp position amount of the base pair for the amount of lp tokens received
        let g_token_amount = if first_token_for_position.token_identifier == base_pair_id {
            first_token_for_position
        } else if second_token_for_position.token_identifier == base_pair_id {
            second_token_for_position
        } else {
            sc_panic!("invalid position tokens received")
        }
        .amount;
        // Protocol keeps 5% of g_token_amount for stability
        let user_amt = &g_token_amount * 95u64 / 100u64;
        let mint_fee = &g_token_amount - &user_amt;

        let mut g_token_payment = self.g_token().mint(g_token_amount);
        self.g_token_supply()
            .update(|old| *old += &g_token_payment.amount);
        g_token_payment.amount = user_amt;

        // Update dust for all
        self.add_dust(
            &first_payment_dust.token_identifier,
            first_send_amt - first_payment_dust.amount,
        );
        self.add_dust(
            &second_payment_dust.token_identifier,
            second_send_amt - second_payment_dust.amount,
        );
        self.add_dust(&g_token_payment.token_identifier, mint_fee);

        self.send().direct_esdt(
            &caller,
            &g_token_payment.token_identifier,
            0,
            &g_token_payment.amount,
        );
    }

    #[storage_mapper("g_token")]
    fn g_token(&self) -> FungibleTokenMapper;

    #[storage_mapper("g_token_supply")]
    fn g_token_supply(&self) -> SingleValueMapper<BigUint>;
}
