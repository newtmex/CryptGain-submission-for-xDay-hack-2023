use liquid_staking::funds::FundsModule;
use multiversx_sc_scenario::{
    scenario_model::{Account, CheckAccount, CheckValue, TxExpect},
    DebugApi, WhiteboxContract,
};
use test_utils::{
    helpers::{big_num_pow_18, call_step, check_account_allow_other_storages, check_step},
    test_setup::TestSetupTrait,
};

use super::{TestSetup, AKF_ADDR, LS_ADDR, OWNER};

#[test]
fn claim_rewards() {
    DebugApi::dummy();

    let mut setup = TestSetup::new();

    setup.trace(
        "Claim rewards by simulating the right conditions",
        "claimRewards/basic.scen.json",
        |setup| {
            setup.init_contracts();

            let ls_whitebox = WhiteboxContract::new(LS_ADDR, liquid_staking::contract_obj);

            let referrer_address = "address:ref";
            let user_address = "address:user";

            let liq_amount = big_num_pow_18(1);
            let liq_amount_str = &liq_amount.to_str_radix(10)[..];

            let reward_top_up = &(big_num_pow_18(1) / 1_000_000u32).to_str_radix(10)[..];
            let mut rps = 0u64;

            let add_liq_result = TxExpect {
                refund: CheckValue::Equal(37_000.into()),
                ..TxExpect::ok()
            }
            .result("0")
            .result("0x0000000a4c53542d3132333435360000000000000000000000080de0b6b3a7640000");

            let mut claim_reward_result = TxExpect {
                refund: CheckValue::Equal(5_000.into()),
                ..TxExpect::ok()
            }
            .result("1,000");

            setup
                .world
                .set_state_step(
                    setup
                        .block_state
                        .move_block_epoch(2, None)
                        .put_account(user_address, Account::new())
                        .put_account(referrer_address, Account::new()),
                )
                .sc_call(
                    call_step("addLiquidity-before-claim", OWNER, LS_ADDR)
                        .function("addLiquidity")
                        .argument(liq_amount_str)
                        .egld_value(liq_amount_str)
                        .gas_limit("50,000,000")
                        .expect(add_liq_result),
                )
                .sc_call(
                    call_step("simulate-rewards-topup-1", OWNER, LS_ADDR)
                        .function("topUpRewards")
                        .egld_value(reward_top_up)
                        .gas_limit("50,000,000"),
                )
                .check_state_step(
                    check_step()
                        .put_account(
                            LS_ADDR,
                            check_account_allow_other_storages(OWNER)
                                .balance(reward_top_up)
                                .check_storage("str:pending_delegation", "0")
                                .check_storage("str:last_claim_epoch", "0"),
                        )
                        .put_account(
                            AKF_ADDR,
                            CheckAccount {
                                owner: CheckValue::Equal(OWNER.into()),
                                ..Default::default()
                            }
                            .balance("0"),
                        )
                        .put_account(user_address, CheckAccount::new().balance("0"))
                        .put_account(referrer_address, CheckAccount::new().balance("0")),
                )
                .sc_call(
                    call_step("claimRewards-no-referrer", OWNER, LS_ADDR)
                        .function("claim_reward")
                        .argument(user_address)
                        .argument("1")
                        .argument(rps.to_string())
                        .argument(liq_amount_str)
                        .gas_limit("100,000,000")
                        .expect(claim_reward_result.clone()),
                )
                .check_state_step(
                    check_step()
                        .put_account(
                            LS_ADDR,
                            check_account_allow_other_storages(OWNER)
                                .balance("30,800,000,000")
                                .check_storage("str:pending_delegation", "30,800,000,000")
                                .check_storage("str:last_claim_epoch", "33"),
                        )
                        .put_account(
                            AKF_ADDR,
                            CheckAccount {
                                owner: CheckValue::Equal(OWNER.into()),
                                ..Default::default()
                            }
                            .balance("46,200,000,000"),
                        )
                        .put_account(user_address, CheckAccount::new().balance("923,000,000,000"))
                        .put_account(referrer_address, CheckAccount::new().balance("0")),
                )
                .whitebox_query(&ls_whitebox, |sc| {
                    rps = sc.reward_per_share().get().to_u64().unwrap();
                    claim_reward_result.out =
                        CheckValue::Equal(vec![CheckValue::Equal("2,000".into())]);
                })
                .set_state_step(setup.block_state.move_block_epoch(20, None))
                .sc_call(
                    call_step("simulate-rewards-topup-2", OWNER, LS_ADDR)
                        .function("topUpRewards")
                        .egld_value(reward_top_up)
                        .gas_limit("50,000,000"),
                )
                .sc_call(
                    call_step("claimRewards-with-referrer", OWNER, LS_ADDR)
                        .function("claim_reward")
                        .argument(user_address)
                        .argument("1")
                        .argument(rps.to_string())
                        .argument(liq_amount_str)
                        .argument(referrer_address)
                        .gas_limit("100,000,000")
                        .expect(claim_reward_result),
                )
                .check_state_step(
                    check_step()
                        .put_account(
                            LS_ADDR,
                            check_account_allow_other_storages(OWNER)
                                .balance("30,800,000,000")
                                .check_storage("str:pending_delegation", "30,800,000,000")
                                .check_storage("str:last_claim_epoch", "53"),
                        )
                        .put_account(
                            AKF_ADDR,
                            CheckAccount {
                                owner: CheckValue::Equal(OWNER.into()),
                                ..Default::default()
                            }
                            .balance("92,400,000,000"),
                        )
                        .put_account(
                            user_address,
                            CheckAccount::new().balance("1,846,000,000,000"),
                        )
                        .put_account(
                            referrer_address,
                            CheckAccount::new().balance("30,800,000,000"),
                        ),
                );
        },
    )
}
