use multiversx_sc_scenario::scenario_model::CheckAccount;
use test_utils::helpers::check_step;

use super::*;

#[test]
fn mint_with_base_pair() {
    TestSetup::new().trace(
        "Minting with Base Pair",
        "mint/min_with_base_pair.scen.json",
        |setup| {
            let user = &setup.add_user_address(0u32.into())[..];

            setup.init_contracts();

            setup
                .world
                .set_state_step(
                    SetStateStep::new()
                        .put_account(user, Account::new().esdt_balance(BASE_PAIR, "200,000")),
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
                                .esdt_balance(LS_TOKEN, "0")
                                .esdt_balance(LSLP_TOKEN, "0")
                                .esdt_balance(BASE_PAIR, "200,000")
                                .esdt_balance(G_TOKEN, "0"),
                        ),
                )
                // First
                .sc_call(
                    call_step("first-mint", user, G_TOKEN_ADDR)
                        .function("mint")
                        .argument("1,50")
                        .argument(LS_TOKEN)
                        .gas_limit("35,000,000")
                        .esdt_transfer(BASE_PAIR, 0, "20,000"),
                )
                .check_state_step(
                    check_step()
                        .put_account(
                            G_TOKEN_ADDR,
                            check_g_addr_g_token("4,000,500")
                                .esdt_balance(LS_TOKEN, "0")
                                .esdt_balance(LSLP_TOKEN, "399,996")
                                .esdt_balance(BASE_PAIR, "11"),
                        )
                        .put_account(
                            user,
                            CheckAccount::new()
                                .esdt_balance(LS_TOKEN, "0")
                                .esdt_balance(LSLP_TOKEN, "0")
                                .esdt_balance(BASE_PAIR, "180,000")
                                .esdt_balance(G_TOKEN, "9,484"),
                        ),
                )
                // Second
                .sc_call(
                    call_step("sencond-mint", user, G_TOKEN_ADDR)
                        .function("mint")
                        .argument("1,50")
                        .gas_limit("35,000,000")
                        .argument(LS_TOKEN)
                        .esdt_transfer(BASE_PAIR, 0, "20,000"),
                )
                .check_state_step(
                    check_step()
                        .put_account(
                            G_TOKEN_ADDR,
                            check_g_addr_g_token("4,000,999")
                                .esdt_balance(LS_TOKEN, "0")
                                .esdt_balance(LSLP_TOKEN, "400,989")
                                .esdt_balance(BASE_PAIR, "23"),
                        )
                        .put_account(
                            user,
                            CheckAccount::new()
                                .esdt_balance(LS_TOKEN, "0")
                                .esdt_balance(LSLP_TOKEN, "0")
                                .esdt_balance(BASE_PAIR, "160,000")
                                .esdt_balance(G_TOKEN, "18,964"),
                        ),
                );
        },
    );
}
