use multiversx_sc_scenario::scenario_model::{CheckAccount, TxExpect};
use test_utils::helpers::check_step;

use super::*;

pub(crate) fn run_mint(name: &str, path: &str, f: impl Fn(&mut TestSetup)) {
    TestSetup::new().trace(name, path, |setup| {
        let user = &setup.add_user_address(0u32.into())[..];

        setup.init_contracts();

        setup
            .world
            .set_state_step(
                SetStateStep::new()
                    .put_account(user, Account::new().esdt_balance(LS_TOKEN, "200,000")),
            )
            .check_state_step(
                check_step()
                    .put_account(
                        G_TOKEN_ADDR,
                        check_g_addr_g_token("4,000,000")
                            .esdt_balance(LS_TOKEN, "0")
                            .esdt_balance(LSLP_TOKEN, "399,000")
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
                call_step("first-mint", user, G_TOKEN_ADDR)
                    .function("mint")
                    .argument("1,50")
                    .gas_limit("35,000,000")
                    .esdt_transfer(LS_TOKEN, 0, "20,000"),
            )
            .check_state_step(
                check_step()
                    .put_account(
                        G_TOKEN_ADDR,
                        check_g_addr_g_token("4,004,763")
                            .esdt_balance(LS_TOKEN, "153")
                            .esdt_balance(LSLP_TOKEN, "408,759")
                            .esdt_balance(BASE_PAIR, "556"),
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
                call_step("second-mint", user, G_TOKEN_ADDR)
                    .function("mint")
                    .argument("1,50")
                    .gas_limit("35,000,000")
                    .esdt_transfer(LS_TOKEN, 0, "20,000"),
            )
            .check_state_step(
                check_step()
                    .put_account(
                        G_TOKEN_ADDR,
                        check_g_addr_g_token("4,009,310")
                            .esdt_balance(LS_TOKEN, "306")
                            .esdt_balance(LSLP_TOKEN, "418,294")
                            .esdt_balance(BASE_PAIR, "985"),
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

        f(setup);
    });
}

#[test]
fn mint() {
    run_mint("Mint GToken", "mint/basic.scen.json", |_| {});
}

#[test]
fn mint_with_g_token() {
    let mut setup = TestSetup::new();
    let user = &setup.add_user_address(0u32.into())[..];

    setup.init_contracts();

    setup
        .world
        .set_state_step(
            SetStateStep::new().put_account(user, Account::new().esdt_balance(G_TOKEN, "20,000")),
        )
        .sc_call(
            g_token_call_step("mint", user)
                .argument("1,50")
                .esdt_transfer(G_TOKEN, 0, "10,000")
                .expect(TxExpect::err(4, "str:Forbidden use of GToken")),
        );
}
