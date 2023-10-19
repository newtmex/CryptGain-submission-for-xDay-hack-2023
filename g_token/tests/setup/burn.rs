use multiversx_sc_scenario::scenario_model::{CheckAccount, TxExpect};
use test_utils::helpers::check_step;

use super::*;

#[test]
fn burn() {
    mint::run_mint("Burn basic", "burn/basic.scen.json", |setup| {
        let user = &setup.users.get(0).unwrap()[..];

        setup
            .world
            .sc_call(
                g_token_call_step("burn", user)
                    .argument(LS_TOKEN)
                    .argument("1,50")
                    .gas_limit("35,000,000")
                    .esdt_transfer(G_TOKEN, 0, "6,000"),
            )
            .check_state_step(
                check_step()
                    .put_account(
                        G_TOKEN_ADDR,
                        check_g_addr_g_token("4,009,310")
                            .esdt_balance(LS_TOKEN, "306")
                            .esdt_balance(LSLP_TOKEN, "417,695")
                            .esdt_balance(BASE_PAIR, "985"),
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
                    .gas_limit("35,000,000")
                    .esdt_transfer(G_TOKEN, 0, "170,882"),
            )
            .check_state_step(
                check_step()
                    .put_account(
                        G_TOKEN_ADDR,
                        check_g_addr_g_token("4,009,310")
                            .esdt_balance(LS_TOKEN, "306")
                            .esdt_balance(LSLP_TOKEN, "400,621")
                            .esdt_balance(BASE_PAIR, "985"),
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
    });
}

#[test]
fn burn_requesting_g_token() {
    mint::run_mint(
        "Burn error by requesting GToken as collateral to return",
        "burn/request_gtoken_as_collateral_out.scen.json",
        |setup| {
            let user = &setup.users.get(0).unwrap()[..];

            setup.world.sc_call(
                g_token_call_step("burn", user)
                    .argument(G_TOKEN)
                    .argument("2,50")
                    .esdt_transfer(G_TOKEN, 0, "10,000")
                    .expect(TxExpect::err(4, "str:Forbidden use of GToken")),
            );
        },
    );
}
