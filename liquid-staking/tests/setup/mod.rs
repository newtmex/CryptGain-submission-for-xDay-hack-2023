pub mod add_liquidity;
pub mod claim_rewards;

use multiversx_sc_scenario::{
    scenario_model::{Account, ScCallStep, ScDeployStep, SetStateStep, TxExpect},
    ScenarioWorld,
};
use test_utils::{
    block_state::BlockState,
    helpers::{big_num_pow_18, update_sc_acc},
    test_setup::TestSetupTrait,
};

// Wasm Files
const LS_WASM: &str = "file:../../output/liquid-staking.wasm";
const DSC_WASM: &str = "file:../../delegation-outputs/delegation.wasm";
const AKF_WASM: &str = "file:../../../akf-mock/output/akf-mock.wasm";
const DELEGATION_PROX_WASM: &str =
    "file:../../../delegation-proxy-mock/output/delegation-proxy-mock.wasm";

// TOKENS
pub const LS_TOKEN: &str = "str:LST-123456";

// Addresses
pub const OWNER: &str = "address:owner";
pub const LS_ADDR: &str = "sc:ls";
pub const DSC_ADDR: &str = "sc:dsc";
pub const AKF_ADDR: &str = "sc:akf";
pub const DELEGATION_PROXY_ADDR: &str = "sc:delegation_proxy";

pub struct TestSetup {
    block_state: BlockState,
    world: ScenarioWorld,
}

impl TestSetup {
    fn new() -> Self {
        let mut world = ScenarioWorld::new();
        world.set_current_dir_from_workspace("liquid-staking");

        world.register_contract(LS_WASM, liquid_staking::ContractBuilder);
        world.register_contract(DSC_WASM, delegation_latest_full::ContractBuilder);
        world.register_contract(AKF_WASM, akf_mock::ContractBuilder);
        world.register_contract(DELEGATION_PROX_WASM, delegation_proxy_mock::ContractBuilder);

        let block_state = BlockState::new(456_484, 14_400);

        Self { block_state, world }
    }

    fn init_contracts(&mut self) {
        let world = &mut self.world;

        let ls_code = world.code_expression(LS_WASM);
        let dsc_code = world.code_expression(DSC_WASM);
        let akf_code = world.code_expression(AKF_WASM);
        let deelgation_mock_code = world.code_expression(DELEGATION_PROX_WASM);

        world
            .set_state_step(
                SetStateStep::new()
                    // Owner Account
                    .put_account(OWNER, Account::new().balance(big_num_pow_18(100_000)))
                    .new_address(OWNER, 0, DSC_ADDR)
                    .new_address(OWNER, 1, LS_ADDR)
                    .new_address(OWNER, 2, AKF_ADDR)
                    .new_address(OWNER, 3, DELEGATION_PROXY_ADDR),
            )
            // Deploy DSC
            .sc_deploy(
                ScDeployStep {
                    id: "deploy-dsc".to_string(),
                    ..Default::default()
                }
                .from(OWNER)
                .code(dsc_code)
                .argument("sc:auction")
                .argument("5,000")
                .argument("0")
                .argument("60")
                .argument("1,000,000,000,000,000,000")
                .argument("str:maximum delegate-able amount")
                .gas_limit("50,000,000")
                .expect(TxExpect::ok().no_result()),
            )
            // Deploy LS Contract
            .sc_deploy(
                ScDeployStep {
                    id: "deploy-ls".to_string(),
                    ..Default::default()
                }
                .from(OWNER)
                .code(&ls_code)
                .gas_limit("50,000,000")
                .argument(DSC_ADDR)
                .argument(AKF_ADDR)
                .argument(DELEGATION_PROXY_ADDR),
            )
            // Deploy AKF Mock Contract
            .sc_deploy(
                ScDeployStep {
                    id: "deploy-akf".to_string(),
                    ..Default::default()
                }
                .from(OWNER)
                .code(&akf_code),
            )
            // Deploy DELEGATION PROXY MOCK Contract
            .sc_deploy(
                ScDeployStep {
                    id: "deploy-delegation-mock".to_string(),
                    ..Default::default()
                }
                .from(OWNER)
                .code(&deelgation_mock_code),
            )
            // Register LS Token
            .sc_call(
                call_step("register-ls-token", OWNER, LS_ADDR)
                    .function("register_ls_token")
                    .argument("str:LSToken")
                    .gas_limit("50,000,000")
                    .argument(LS_TOKEN),
            )
            .set_state_step(SetStateStep::new().put_account(
                LS_ADDR,
                update_sc_acc(&ls_code, vec![("str:ls_token", LS_TOKEN)]).esdt_roles(
                    LS_TOKEN,
                    vec![
                        "ESDTRoleLocalMint".to_string(),
                        "ESDTRoleLocalBurn".to_string(),
                    ],
                ),
            ));
    }
}

impl TestSetupTrait for TestSetup {
    fn world(&mut self) -> &mut ScenarioWorld {
        &mut self.world
    }
}

fn call_step(tx_id: &str, from: &str, to: &str) -> ScCallStep {
    ScCallStep {
        id: tx_id.into(),
        ..Default::default()
    }
    .from(from)
    .to(to)
}
