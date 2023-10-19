use std::ops::Div;

use liquid_staking::AddLiquidityResultType;
use multiversx_sc::{
    codec::TopEncode,
    types::{self, EsdtTokenPayment, ManagedBuffer},
};
use multiversx_sc_scenario::{
    multiversx_chain_vm::world_mock::AccountData,
    scenario_model::{CheckAccount, CheckStateStep, TxExpect, U64Value},
    DebugApi,
};
use test_utils::{
    helpers::{big_num_pow_18, call_step, check_account_allow_other_storages, check_step},
    test_setup::TestSetupTrait,
};

use super::{TestSetup, LS_ADDR, LS_TOKEN, OWNER};

#[test]
fn add_liquidity() {
    DebugApi::dummy();

    let mut setup = TestSetup::new();

    setup.trace(
        "Add liquidity scenario",
        "addLiquidity/basic.scen.json",
        |setup| {
            setup.init_contracts();

            let liq_amount = big_num_pow_18(1).div(10u32);
            let liq_amount_str = &liq_amount.to_str_radix(10)[..];

            let AccountData {
                nonce: owner_nonce,
                mut egld_balance,
                ..
            } = setup.get_account_data(OWNER);

            egld_balance -= &liq_amount;
            let current_egld_balance = &egld_balance.to_str_radix(10)[..];

            setup
                .world
                .sc_call(
                    call_step("addLiquidity-with-lt-1eGLD", OWNER, LS_ADDR)
                        .function("addLiquidity")
                        .argument(liq_amount_str)
                        .egld_value(liq_amount_str),
                )
                .check_state_step(
                    CheckStateStep {
                        comment: Some("users-can-add-sub-1eGLD-as-liquidity".into()),
                        ..check_step()
                    }
                    .put_account(
                        OWNER,
                        CheckAccount::new()
                            .balance(current_egld_balance)
                            .nonce(U64Value::from(owner_nonce + 1))
                            .esdt_balance(LS_TOKEN, liq_amount_str),
                    )
                    .put_account(
                        LS_ADDR,
                        check_account_allow_other_storages(OWNER)
                            .balance(liq_amount_str)
                            .check_storage("str:ls_token_supply", liq_amount_str)
                            .check_storage("str:delegated_egld", liq_amount_str)
                            .check_storage("str:pending_delegation", liq_amount_str),
                    ),
                );

            let liq_amount_2 = big_num_pow_18(1);
            let liq_amount_2_str = &liq_amount_2.to_str_radix(10)[..];

            egld_balance -= &liq_amount_2;
            let current_egld_balance = &egld_balance.to_str_radix(10)[..];

            let final_liq_amount = liq_amount + &liq_amount_2;
            let final_liq_amount_str = &final_liq_amount.to_str_radix(10)[..];

            let add_liq_result = {
                let add_liq_result = AddLiquidityResultType::<DebugApi>::from((
                    types::BigUint::from(0u32),
                    EsdtTokenPayment::new(LS_TOKEN[4..].into(), 0, liq_amount_2.into()),
                ))
                .into_tuple();

                let mut rps = ManagedBuffer::<DebugApi>::new();
                add_liq_result.0.top_encode(&mut rps).unwrap();

                let mut ls_token = ManagedBuffer::<DebugApi>::new();
                add_liq_result.1.top_encode(&mut ls_token).unwrap();

                TxExpect::ok()
                    .result(&format!("0x{}", hex::encode(rps.to_vec())))
                    .result(&format!("0x{}", hex::encode(ls_token.to_vec())))
            };

            setup
                .world
                .sc_call(
                    call_step("addLiquidity-trigger-delegate-to-dsc", OWNER, LS_ADDR)
                        .function("addLiquidity")
                        .argument(liq_amount_2_str)
                        .egld_value(liq_amount_2_str)
                        .gas_limit("45,000,000")
                        .expect(add_liq_result),
                )
                .check_state_step(
                    CheckStateStep {
                        comment: Some("all-pending-delegations-should-be-cleared".into()),
                        ..check_step()
                    }
                    .put_account(
                        OWNER,
                        CheckAccount::new()
                            .balance(current_egld_balance)
                            .nonce(U64Value::from(owner_nonce + 2))
                            .esdt_balance(LS_TOKEN, final_liq_amount_str),
                    )
                    .put_account(
                        LS_ADDR,
                        check_account_allow_other_storages(OWNER)
                            .balance("1100000000000000000")
                            .check_storage("str:ls_token_supply", final_liq_amount_str)
                            .check_storage("str:delegated_egld", final_liq_amount_str)
                            .check_storage("str:pending_delegation", "0x0f43fc2c04ee0000"),
                    ),
                );
        },
    )
}
