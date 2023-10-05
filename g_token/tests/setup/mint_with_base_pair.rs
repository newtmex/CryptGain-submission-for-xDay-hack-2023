use multiversx_sc_scenario::scenario_model::{CheckStateStep, CheckAccount};

use super::*;

#[test]
fn mint_with_base_pair() {
    let mut setup = TestSetup::new();
    let user = &setup.add_user_address(0u32.into())[..];

    setup.init_contracts();

    setup
        .world
        .set_state_step(
            SetStateStep::new()
                .put_account(user, Account::new().esdt_balance(BASE_PAIR, "200,000")),
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
                        .esdt_balance(LS_TOKEN, "0")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "200,000")
                        .esdt_balance(G_TOKEN, "0"),
                ),
        )
        // First
        .sc_call(
            ScCallStep::new()
                .from(user)
                .to(G_TOKEN_ADDR)
                .function("mint")
                .argument("1,50")
                .argument(LS_TOKEN)
                .esdt_transfer(BASE_PAIR, 0, "20,000"),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(
                    G_TOKEN_ADDR,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "0")
                        .esdt_balance(LSLP_TOKEN, "399,996")
                        .esdt_balance(BASE_PAIR, "11")
                        .esdt_balance(G_TOKEN, "4,000,500"),
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
            ScCallStep::new()
                .from(user)
                .to(G_TOKEN_ADDR)
                .function("mint")
                .argument("1,50")
                .argument(LS_TOKEN)
                .esdt_transfer(BASE_PAIR, 0, "20,000"),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(
                    G_TOKEN_ADDR,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "0")
                        .esdt_balance(LSLP_TOKEN, "400,989")
                        .esdt_balance(BASE_PAIR, "23")
                        .esdt_balance(G_TOKEN, "4,000,999"),
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
}
