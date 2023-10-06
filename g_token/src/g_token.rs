#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use pair::{AddLiquidityResultType, ProxyTrait as _, RemoveLiquidityResultType};

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
    fn mint(&self, slippage: u64, opt_g_pair: OptionalValue<TokenIdentifier>) {
        let caller = self.blockchain().get_caller();

        // Set payment swap amount
        let mut sent_payment = self.call_value().single_esdt();
        require!(
            sent_payment.amount >= MIN_MINT_DEPOSIT,
            "Insufficient liquidity deposit"
        );
        sent_payment.amount /= 2u64;

        let (base_pair_id, g_pair_id) =
            self.get_base_and_g_pair_id(opt_g_pair, &sent_payment.token_identifier);

        let pair_info = self.get_pair_info(&g_pair_id);
        let call_pair = || self.pair_proxy_obj(pair_info.address.clone());

        // Compute other pair payment
        let mut amount_out: BigUint<Self::Api> = call_pair()
            .get_amount_out_view(&sent_payment.token_identifier, &sent_payment.amount)
            .execute_on_dest_context();
        slippage::apply(&mut amount_out, slippage);

        let compute_amounts_min = |token_payment: &EsdtTokenPayment| {
            let token_amount_min = slippage::from_ref(&token_payment.amount, slippage);
            let other_amount_min = call_pair()
                .get_equivalent(&token_payment.token_identifier, &token_amount_min)
                .execute_on_dest_context::<BigUint>();
            require!(
                token_amount_min > 0 && other_amount_min > 0,
                "Invalid slippage, sent amount combination"
            );

            (token_amount_min, other_amount_min)
        };
        
        let (
            //
            (first_payment, first_token_amount_min),
            (second_payment, second_token_amount_min),
        ) = if sent_payment.token_identifier == base_pair_id {
            let g_pair_payment =
                self.pair_swap_fixed_input(&g_pair_id, amount_out, sent_payment.clone(), call_pair);

            let (g_pair_amount_min, sent_pair_amount_min) = compute_amounts_min(&g_pair_payment);

            (
                (g_pair_payment, g_pair_amount_min),
                (sent_payment, sent_pair_amount_min),
            )
        } else {
            let (base_pair_payment, g_pair_payment_dust) = self
                .pair_swap_fixed_output(&base_pair_id, amount_out, sent_payment.clone(), call_pair)
                .into_tuple();

            let (base_pair_amount_min, sent_pair_amount_min) =
                compute_amounts_min(&base_pair_payment);

            self.add_dust(
                &g_pair_payment_dust.token_identifier,
                g_pair_payment_dust.amount,
            );

            (
                (sent_payment, sent_pair_amount_min),
                (base_pair_payment, base_pair_amount_min),
            )
        };

        let first_token_sent_amt = first_payment.amount.clone();
        let second_token_sent_amt = second_payment.amount.clone();
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

        let (first_token_for_position, second_token_for_position) = self
            .pair_get_tokens_for_given_position(&lp_payment.amount, call_pair)
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
            let user_amt = self.add_g_supply(g_pair_id, &g_token_payment.amount, lp_payment);
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

    #[endpoint]
    #[payable("*")]
    fn burn(
        &self,
        receipt_token_id: TokenIdentifier,
        slippage: u64,
        opt_g_pair: OptionalValue<TokenIdentifier>,
    ) {
        let caller = self.blockchain().get_caller();
        let (_base_pair_id, g_pair_id) = self.get_base_and_g_pair_id(opt_g_pair, &receipt_token_id);

        let pair_info = self.get_pair_info(&g_pair_id);
        let call_pair = || self.pair_proxy_obj(pair_info.address.clone());

        let g_token_payment = self.call_value().single_esdt();
        self.g_token()
            .require_same_token(&g_token_payment.token_identifier);
        // TODO add other risk management checks like the max % that can be withdrawn per period (blocks, timestamp range, epochs ??)
        require!(
            g_token_payment.amount <= pair_info.g_token_supply,
            "GToken burn amount exceeded for pair",
        );

        let lp_payment = self.remove_g_supply(g_pair_id, g_token_payment);
        let (first_token_for_position, second_token_for_position) = call_pair()
            .get_tokens_for_given_position(&lp_payment.amount)
            .execute_on_dest_context::<MultiValue2<EsdtTokenPayment, EsdtTokenPayment>>()
            .into_tuple();

        let apply_slippage = |amt: &BigUint<Self::Api>| slippage::from_ref(amt, slippage);

        let first_token_amount_min = apply_slippage(&first_token_for_position.amount);
        let second_token_amount_min = apply_slippage(&second_token_for_position.amount);
        let (first_token_received, second_token_received) = call_pair()
            .remove_liquidity(first_token_amount_min, second_token_amount_min)
            .with_esdt_transfer(lp_payment)
            .execute_on_dest_context::<RemoveLiquidityResultType<Self::Api>>()
            .into_tuple();

        // Swap other_token_payment to requested token
        let receipt_token_payment = {
            let (mut receipt_token_payment, other_token_payment) =
                if first_token_received.token_identifier == receipt_token_id {
                    (first_token_received, second_token_received)
                } else if second_token_received.token_identifier == receipt_token_id {
                    (second_token_received, first_token_received)
                } else {
                    sc_panic!("Unexpected liquidity tokens received")
                };

            let amount_out_min = apply_slippage(&receipt_token_payment.amount);
            let (swapped_id, _, swapped_amount) = self
                .pair_swap_fixed_input(
                    &receipt_token_payment.token_identifier,
                    amount_out_min,
                    other_token_payment,
                    call_pair,
                )
                .into_tuple();
            require!(
                swapped_id == receipt_token_payment.token_identifier,
                "Swapped token receipt token missmatch"
            );
            receipt_token_payment.amount += swapped_amount;

            receipt_token_payment
        };

        let (receipt_identifier, receipt_nonce, receipt_amount) =
            receipt_token_payment.into_tuple();
        self.send()
            .direct_esdt(&caller, &receipt_identifier, receipt_nonce, &receipt_amount);
    }

    fn get_base_and_g_pair_id(
        &self,
        opt_g_pair: OptionalValue<TokenIdentifier>,
        known_pair_id: &TokenIdentifier,
    ) -> (TokenIdentifier, TokenIdentifier) {
        let g_token_id = self.g_token().get_token_id();
        require!(known_pair_id != &g_token_id, "Forbidden use of GToken");

        let base_pair_id = self.base_pair().get_token_id();

        let g_pair_id = opt_g_pair
            .into_option()
            .unwrap_or_else(|| known_pair_id.clone());
        require!(g_pair_id != base_pair_id, "Specify GPair ID");

        (base_pair_id, g_pair_id)
    }
}
