use std::ops::Mul;

use multiversx_sc_scenario::{
    multiversx_chain_vm::world_mock::BlockInfo,
    scenario_model::{
        Account, BytesKey, BytesValue, CheckAccount, CheckStateStep, ScCallStep, ScDeployStep,
        SetStateStep,
    },
    ScenarioWorld,
};

pub struct BlockState {
    pub rounds_per_epoch: u64,
    pub info: BlockInfo,
}

impl BlockState {
    pub fn new(start_round: u64, rounds_per_epoch: u64) -> Self {
        let mut info = BlockInfo::new();
        info.block_round = start_round;
        info.block_epoch = start_round / rounds_per_epoch;

        Self {
            rounds_per_epoch,
            info,
        }
    }

    pub fn move_block_round(&mut self, blocks: u64, step: Option<SetStateStep>) -> SetStateStep {
        let step = match step {
            Some(step) => step,
            None => SetStateStep::new(),
        };

        let BlockInfo {
            block_round,
            block_epoch,
            ..
        } = &mut self.info;

        *block_round += blocks;
        *block_epoch = *block_round / self.rounds_per_epoch;

        self.set_and_return_step(step)
    }

    fn set_and_return_step(&self, step: SetStateStep) -> SetStateStep {
        step.block_round(self.info.block_round)
            .block_epoch(self.info.block_epoch)
    }

    pub fn move_block_epoch(&mut self, epochs: u64, step: Option<SetStateStep>) -> SetStateStep {
        let step = match step {
            Some(step) => step,
            None => SetStateStep::new(),
        };

        let BlockInfo {
            block_round,
            block_epoch,
            ..
        } = &mut self.info;

        *block_epoch += epochs;
        *block_round = *block_epoch * self.rounds_per_epoch;

        self.set_and_return_step(step)
    }
}

// Wasm Files
const G_TOKEN_WASM: &str = "file:../output/g_token.wasm";
const ROUTER_WASM: &str = "file:../dex-outputs/router.wasm";
const PAIR_WASM: &str = "file:../dex-outputs/pair.wasm";

// TOKENS
const BASE_PAIR: &str = "str:BASETK-123456";
const G_TOKEN: &str = "str:GTK-123456";
const LS_TOKEN: &str = "str:LST-123456";
const LSLP_TOKEN: &str = "str:LSLP-123456";

// Addresses
const OWNER: &str = "address:owner";
const G_TOKEN_ADDR: &str = "sc:g_token";

fn g_token_call_step(func: &str, address: &str) -> ScCallStep {
    ScCallStep {
        id: func.into(),
        ..Default::default()
    }
    .to(G_TOKEN_ADDR)
    .function(func)
    .from(address)
}

fn update_sc_acc<K, V>(code: &BytesValue, storage_values: Vec<(K, V)>) -> Account
where
    K: Into<BytesKey>,
    V: Into<BytesValue>,
{
    let mut account_state = Account::new().update(true).code(code);

    for (key, value) in storage_values {
        account_state.storage.insert(key.into(), value.into());
    }

    account_state
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
                ScDeployStep::new()
                    .from(OWNER)
                    .code(&pair_code)
                    .argument("str:DUM-123456")
                    .argument("str:DUM-654321")
                    .argument("sc:zero")
                    .argument("sc:zero")
                    .argument("0")
                    .argument("0")
                    .argument("sc:zero"),
            )
            // // Deploy Router
            .sc_deploy(
                ScDeployStep::new()
                    .from(OWNER)
                    .code(router_code)
                    .argument(pair_temp_addr),
            )
            .sc_call(
                ScCallStep::new()
                    .to(router_addr)
                    .from(OWNER)
                    .function("resume")
                    .argument(router_addr),
            )
            .sc_call(
                ScCallStep::new()
                    .to(router_addr)
                    .from(OWNER)
                    .function("setPairCreationEnabled")
                    .argument("true"),
            )
            // Deploy GToken
            .sc_deploy(
                ScDeployStep::new()
                    .from(OWNER)
                    .code(&g_token_code)
                    .argument(router_addr)
                    .argument(BASE_PAIR),
            )
            .sc_call(
                g_token_call_step("registerGToken", OWNER)
                    .argument("str:GToken")
                    .argument(G_TOKEN)
                    .argument("18")
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
            .sc_call(g_token_call_step("router_create_pair", OWNER).argument(LS_TOKEN))
            .sc_call(
                g_token_call_step("router_issue_lp", OWNER)
                    .argument(LS_TOKEN)
                    .argument("str:LSLP")
                    .argument(LSLP_TOKEN)
                    .egld_value(num_bigint::BigUint::from(10u32).pow(18).mul(5u32)),
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
            .sc_call(g_token_call_step("router_set_lp_local_roles", OWNER).argument(LS_TOKEN))
            .sc_call(
                g_token_call_step("pair_add_initial_liquidity", OWNER)
                    .argument(LS_TOKEN)
                    .esdt_transfer(LS_TOKEN, 0, "400,000")
                    .esdt_transfer(BASE_PAIR, 0, "4,000,000"),
            )
            .sc_call(
                ScCallStep::new()
                    .to(router_addr)
                    .from(OWNER)
                    .function("resume")
                    .argument(ls_pair_addr),
            );
    }
}

#[test]
fn mint() {
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
                        .esdt_balance(LS_TOKEN, "0")
                        .esdt_balance(LSLP_TOKEN, "408,756")
                        .esdt_balance(BASE_PAIR, "2,087")
                        .esdt_balance(G_TOKEN, "4,004,760"),
                )
                .put_account(
                    user,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "180,000")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "0")
                        .esdt_balance(G_TOKEN, "90,427"),
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
                        .esdt_balance(LS_TOKEN, "0")
                        .esdt_balance(LSLP_TOKEN, "418,285")
                        .esdt_balance(BASE_PAIR, "3,971")
                        .esdt_balance(G_TOKEN, "4,009,301"),
                )
                .put_account(
                    user,
                    CheckAccount::new()
                        .esdt_balance(LS_TOKEN, "160,000")
                        .esdt_balance(LSLP_TOKEN, "0")
                        .esdt_balance(BASE_PAIR, "0")
                        .esdt_balance(G_TOKEN, "176,702"),
                ),
        );
}
