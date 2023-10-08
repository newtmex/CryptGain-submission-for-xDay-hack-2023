pub mod burn;
pub mod mint;
pub mod mint_with_base_pair;

use std::ops::Mul;

use multiversx_sc_scenario::scenario_model::{CheckAccount, CheckEsdtMap, TxExpect};
pub use multiversx_sc_scenario::{
    multiversx_chain_vm::world_mock::BlockInfo,
    scenario_model::{Account, BytesKey, BytesValue, ScCallStep, ScDeployStep, SetStateStep},
    ScenarioWorld,
};
use test_utils::{
    block_state::BlockState,
    helpers::{call_step, check_account_with_owner, update_sc_acc},
    test_setup::TestSetupTrait,
};

// Wasm Files
pub const G_TOKEN_WASM: &str = "file:../../output/g_token.wasm";
pub const ROUTER_WASM: &str = "file:../../dex-outputs/router.wasm";
pub const PAIR_WASM: &str = "file:../../dex-outputs/pair.wasm";

// TOKENS
pub const BASE_PAIR: &str = "str:BASETK-123456";
pub const G_TOKEN: &str = "str:GTK-123456";
pub const LS_TOKEN: &str = "str:LST-123456";
pub const LSLP_TOKEN: &str = "str:LSLP-123456";

// Addresses
pub const OWNER: &str = "address:owner";
pub const G_TOKEN_ADDR: &str = "sc:g_token";

fn g_token_call_step(func: &str, address: &str) -> ScCallStep {
    ScCallStep {
        id: func.into(),
        ..Default::default()
    }
    .to(G_TOKEN_ADDR)
    .function(func)
    .from(address)
}

pub struct TestSetup {
    block_state: BlockState,
    world: ScenarioWorld,
    users: Vec<String>,
}

impl TestSetup {
    fn new() -> Self {
        let mut block_state = BlockState::new(500_000, 14_400);
        let mut world = ScenarioWorld::new();

        world.set_current_dir_from_workspace("g_token");

        world.register_contract(G_TOKEN_WASM, g_token::ContractBuilder);
        world.register_contract(ROUTER_WASM, router::ContractBuilder);
        world.register_contract(PAIR_WASM, pair::ContractBuilder);

        world.set_state_step(block_state.move_block_round(0, None));

        Self {
            block_state,
            world,
            users: vec![],
        }
    }

    fn add_user_address(&mut self, balance: num_bigint::BigUint) -> String {
        let address = format!("address:user{}", self.users.len() + 1);
        self.users.push(address.clone());

        self.world.set_state_step(
            SetStateStep::new().put_account(&address[..], Account::new().balance(balance)),
        );

        address
    }

    fn init_contracts(&mut self) {
        let router_addr = "sc:router";
        let pair_temp_addr = "sc:pair_temp";
        let ls_pair_addr = "sc:ls_base_pair";

        let world = &mut self.world;

        let g_token_code = world.code_expression(G_TOKEN_WASM);
        let router_code = world.code_expression(ROUTER_WASM);
        let pair_code = world.code_expression(PAIR_WASM);

        world
            .set_state_step(
                SetStateStep::new()
                    .put_account(
                        OWNER,
                        Account::new()
                            .balance(num_bigint::BigUint::from(10u32).pow(18).mul(1_000_000u32)),
                    )
                    .new_address(OWNER, 0, pair_temp_addr)
                    .new_address(OWNER, 1, router_addr)
                    .new_address(OWNER, 4, G_TOKEN_ADDR),
            )
            // Pair Template
            .sc_deploy(
                ScDeployStep {
                    id: "deploy-pair-template".to_string(),
                    ..Default::default()
                }
                .from(OWNER)
                .code(&pair_code)
                .gas_limit("600,000,000")
                .argument("str:DUM-123456")
                .argument("str:DUM-654321")
                .argument("sc:zero")
                .argument("sc:zero")
                .argument("0")
                .argument("0")
                .argument("sc:zero"),
            )
            // Deploy Router
            .sc_deploy(
                ScDeployStep {
                    id: "deploy-router".to_string(),
                    ..Default::default()
                }
                .from(OWNER)
                .gas_limit("600,000,000")
                .code(router_code)
                .argument(pair_temp_addr),
            )
            .sc_call(
                call_step("resume_router", OWNER, router_addr)
                    .function("resume")
                    .argument(router_addr),
            )
            .sc_call(
                call_step("setPairCreationEnabled", OWNER, router_addr)
                    .function("setPairCreationEnabled")
                    .argument("true"),
            )
            // Deploy GToken
            .sc_deploy(
                ScDeployStep {
                    id: "deploy".to_string(),
                    ..Default::default()
                }
                .from(OWNER)
                .gas_limit("600,000,000")
                .code(&g_token_code)
                .argument(router_addr)
                .argument(BASE_PAIR),
            )
            .sc_call(
                g_token_call_step("registerGToken", OWNER)
                    .argument("str:GToken")
                    .argument(G_TOKEN)
                    .argument("18")
                    .gas_limit("50,000,000")
                    .egld_value("5,000,000,000,000,000,000"),
            )
            .set_state_step(
                SetStateStep::new()
                    .put_account(
                        G_TOKEN_ADDR,
                        update_sc_acc(&g_token_code, vec![("str:g_token", G_TOKEN)]).esdt_roles(
                            G_TOKEN,
                            vec!["ESDTRoleLocalMint".into(), "ESDTRoleLocalBurn".into()],
                        ),
                    )
                    .new_address(router_addr, 0, ls_pair_addr),
            )
            .sc_call(
                g_token_call_step("router_create_pair", OWNER)
                    .argument(LS_TOKEN)
                    .gas_limit("50,000,000"),
            )
            .sc_call(
                g_token_call_step("router_issue_lp", OWNER)
                    .argument(LS_TOKEN)
                    .argument("str:LSLP")
                    .argument(LSLP_TOKEN)
                    .egld_value(num_bigint::BigUint::from(10u32).pow(18).mul(5u32))
                    .gas_limit("50,000,000")
                    .expect(TxExpect {
                        build_from_response: false,
                        ..TxExpect::ok()
                    }),
            )
            .set_state_step(
                SetStateStep::new()
                    .put_account(
                        ls_pair_addr,
                        update_sc_acc(
                            &pair_code,
                            vec![
                                ("str:lpTokenIdentifier", LSLP_TOKEN),
                                ("str:first_token_id", LS_TOKEN),
                                ("str:second_token_id", BASE_PAIR),
                            ],
                        )
                        .esdt_roles(
                            LSLP_TOKEN,
                            vec!["ESDTRoleLocalMint".into(), "ESDTRoleLocalBurn".into()],
                        ),
                    )
                    .put_account(
                        OWNER,
                        Account::new()
                            .esdt_balance(LS_TOKEN, "400,000")
                            .esdt_balance(BASE_PAIR, "4,000,000"),
                    ),
            )
            .sc_call(
                g_token_call_step("router_set_lp_local_roles", OWNER)
                    .argument(LS_TOKEN)
                    .gas_limit("50,000,000"),
            )
            .sc_call(
                g_token_call_step("pair_add_initial_liquidity", OWNER)
                    .argument(LS_TOKEN)
                    .esdt_transfer(LS_TOKEN, 0, "400,000")
                    .esdt_transfer(BASE_PAIR, 0, "4,000,000")
                    .gas_limit("50,000,000"),
            )
            .sc_call(
                call_step("resume_ls_pair", OWNER, router_addr)
                    .function("resume")
                    .argument(ls_pair_addr)
                    .gas_limit("50,000,000"),
            );
    }
}

impl TestSetupTrait for TestSetup {
    fn world(&mut self) -> &mut ScenarioWorld {
        &mut self.world
    }
}

fn check_g_addr_g_token(balance_expr: &str) -> CheckAccount {
    let mut check = check_account_with_owner(OWNER).esdt_nft_balance_and_attributes(
        G_TOKEN,
        "0",
        balance_expr,
        Option::<&str>::None,
    );

    match &mut check.esdt {
        CheckEsdtMap::Unspecified => todo!(),
        CheckEsdtMap::Star => todo!(),
        CheckEsdtMap::Equal(map) => {
            map.contents
                .entry(G_TOKEN.into())
                .and_modify(|v| v.add_roles_check(vec!["ESDTRoleLocalMint", "ESDTRoleLocalBurn"]));
        },
    }

    check
}
