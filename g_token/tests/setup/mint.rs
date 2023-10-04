use multiversx_sc_scenario::scenario_model::{CheckAccount, CheckStateStep};

use super::*;

pub(crate) fn run_mint() -> TestSetup {
    let mut setup = TestSetup::new();
    let user = &setup.add_user_address(0u32.into())[..];

    setup.init_contracts();

    setup
        .world
        .set_state_step(
            SetStateStep::new().put_account(user, Account::new().esdt_balance(LS_TOKEN, "200,000")),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(
                    G_TOKEN_ADDR,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "0")
                        .esdt_balance(LSLP_TOKEN, "399,000")
                        .esdt_balance(G_TOKEN, "4,000,000")
                        .esdt_balance(BASE_PAIR, "0"),
                )
                .put_account(
                    user,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "200,000")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "0")
                        .esdt_balance(G_TOKEN, "0"),
                ),
        )
        // First
        .sc_call(
            ScCallStep::new()
                .from(user)
                .to(G_TOKEN_ADDR)
                .function("mint")
                .esdt_transfer(LS_TOKEN, 0, "20,000"),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(
                    G_TOKEN_ADDR,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "153")
                        .esdt_balance(LSLP_TOKEN, "408,759")
                        .esdt_balance(BASE_PAIR, "556")
                        .esdt_balance(G_TOKEN, "4,004,763"),
                )
                .put_account(
                    user,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "180,000")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "0")
                        .esdt_balance(G_TOKEN, "90,489"),
                ),
        )
        // Second
        .sc_call(
            ScCallStep::new()
                .from(user)
                .to(G_TOKEN_ADDR)
                .function("mint")
                .esdt_transfer(LS_TOKEN, 0, "20,000"),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(
                    G_TOKEN_ADDR,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "306")
                        .esdt_balance(LSLP_TOKEN, "418,294")
                        .esdt_balance(BASE_PAIR, "985")
                        .esdt_balance(G_TOKEN, "4,009,310"),
                )
                .put_account(
                    user,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "160,000")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "0")
                        .esdt_balance(G_TOKEN, "176,882"),
                ),
        );

    setup
}

#[test]
fn mint() {
    run_mint();
}

#[test]
#[should_panic(expected = "Forbidden use of GToken")]
fn mint_with_g_token() {
    let mut setup = TestSetup::new();
    let user = &setup.add_user_address(0u32.into())[..];

    setup.init_contracts();

    setup
        .world
        .set_state_step(
            SetStateStep::new().put_account(user, Account::new().esdt_balance(G_TOKEN, "20,000")),
        )
        .sc_call(g_token_call_step("mint", user).esdt_transfer(G_TOKEN, 0, "10,000"));
}
