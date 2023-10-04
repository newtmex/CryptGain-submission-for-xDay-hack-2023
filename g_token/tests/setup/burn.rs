use multiversx_sc_scenario::scenario_model::{CheckAccount, CheckStateStep};

use super::*;

#[test]
fn burn() {
    let mut setup = mint::run_mint();
    let user = &setup.users.get(0).unwrap()[..];

    setup
        .world
        .sc_call(
            g_token_call_step("burn", user)
                .argument(LS_TOKEN)
                .argument("1,50")
                .esdt_transfer(G_TOKEN, 0, "6,000"),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(
                    G_TOKEN_ADDR,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "306")
                        .esdt_balance(LSLP_TOKEN, "417,695")
                        .esdt_balance(BASE_PAIR, "985")
                        .esdt_balance(G_TOKEN, "4,009,310"),
                )
                .put_account(
                    user,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "161,253")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "0")
                        .esdt_balance(G_TOKEN, "170,882"),
                ),
        )
        .sc_call(
            g_token_call_step("burn", user)
                .argument(LS_TOKEN)
                .argument("4,50")
                .esdt_transfer(G_TOKEN, 0, "170,882"),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(
                    G_TOKEN_ADDR,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "306")
                        .esdt_balance(LSLP_TOKEN, "400,621")
                        .esdt_balance(BASE_PAIR, "985")
                        .esdt_balance(G_TOKEN, "4,009,310"),
                )
                .put_account(
                    user,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "196,232")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "0")
                        .esdt_balance(G_TOKEN, "0"),
                ),
        );
}

#[test]
#[should_panic(expected = "Forbidden use of GToken")]
fn burn_requesting_g_token() {
    let mut setup = mint::run_mint();
    let user = &setup.users.get(0).unwrap()[..];

    setup.world.sc_call(
        g_token_call_step("burn", user)
            .argument(G_TOKEN)
            .argument("2,50")
            .esdt_transfer(G_TOKEN, 0, "10,000"),
    );
}
