#![cfg(test)]

pub mod fuzz_data;
pub mod helpers;

use fuzz_data::{
    pair_addr_from_token_id, FuzzerData, TokenTransfer, G_TOKEN, G_TOKEN_ADDR, LS_TOKEN,
    NATIVE_TOKEN, OWNER, TOKEN_IDS,
};
use rand::Rng;

#[test]
fn continuous_convertion_of_gtoken_to_base_pair_should_not_be_profitable() {
    let mut fuzzer = FuzzerData::new();

    let [_native_token, _ls_token, base_token, g_token] = TOKEN_IDS;

    let base_init_bal = fuzzer.get_esdt_balance(OWNER, &base_token[4..]);
    let mut prev_g_token_bal = num_bigint::BigUint::from(0u32);
    while let Some(g_token_bal) = fuzzer.get_esdt_balance_not_zero(OWNER, &g_token[4..]) {
        if prev_g_token_bal > 0u32.into() {
            assert!(
                (&g_token_bal * 10_000u32 / &prev_g_token_bal) < 2_500u32.into(),
                "GToken balance is meant to reduce by at least 8.5%"
            );
        }
        prev_g_token_bal = g_token_bal.clone();

        fuzzer.swap_fixed_input(
            OWNER,
            "sc:GTK_pair",
            TokenTransfer {
                token_id: g_token.into(),
                amount: g_token_bal,
            },
        );

        let added_base_amt = fuzzer.get_esdt_balance(OWNER, &base_token[4..]) - &base_init_bal;
        if added_base_amt >= g_token::MIN_MINT_DEPOSIT.into() {
            fuzzer.mint(OWNER, base_token, added_base_amt, Some(g_token));
        }
    }
}

fn run_arbitrage<const N: usize>(token_ids: [&str; N], max_users: u32) {
    let mut fuzzer = FuzzerData::new().seed_accounts(max_users);

    let mut addresses = vec![OWNER.to_string(), G_TOKEN_ADDR.to_string()];
    addresses.extend_from_slice(&fuzzer.get_users());
    for id in token_ids {
        if !id.contains("BASE") {
            addresses.push(pair_addr_from_token_id(id))
        }
    }

    for address in addresses.iter() {
        fuzzer.dump_account(address);
    }

    let mut last_price_feed = None;
    let mut price_feed_same_count = 0;
    let mut rng = rand::thread_rng();

    'outer_loop: for count in 1..=2000 {
        for v in 0..=rng.gen_range((count / 5)..=(count / 4) + 1) {
            let price_feed = fuzzer.arbitrage(token_ids);

            if let Some(last_feed) = last_price_feed {
                // Consecutive times
                if last_feed == price_feed {
                    price_feed_same_count += 1;
                } else {
                    price_feed_same_count = 0;
                }
            }

            last_price_feed = Some(price_feed);
            if v % 3 == 0 {
                fuzzer.move_block_round();
            }

            if price_feed_same_count >= 30 {
                break 'outer_loop;
            }
        }

        fuzzer.move_block_round();
    }

    for address in addresses {
        fuzzer.dump_account(&address);
    }
}

#[test]
fn arbitrage_with_only_g_token() {
    run_arbitrage([G_TOKEN], 1);
}

#[test]
fn arbitrage_and_best_gtoken_minting_opportunity() {
    run_arbitrage([NATIVE_TOKEN, LS_TOKEN, G_TOKEN], 3);
}
