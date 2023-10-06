use std::collections::HashMap;

use g_token::config::Config;
use multiversx_sc::types::TokenIdentifier;
use multiversx_sc_scenario::{
    multiversx_chain_vm::world_mock::{AccountData, BlockInfo},
    scenario_model::{
        Account, AddressValue, BytesKey, BytesValue, ScCallStep, ScDeployStep, SetStateStep,
        TransferStep,
    },
    DebugApi, ScenarioWorld, WhiteboxContract,
};
use pair::{config::ConfigModule, Pair};
use rand::Rng;

use crate::helpers::{big_to_pow_18, write};

pub type TokenBalance = HashMap<String, num_bigint::BigUint>;

#[derive(Debug)]
struct User {
    address: String,
    wallet: TokenBalance,
}
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
pub const G_TOKEN_WASM: &str = "file:../g_token/output/g_token.wasm";
pub const ROUTER_WASM: &str = "file:../g_token/dex-outputs/router.wasm";
pub const PAIR_WASM: &str = "file:../g_token/dex-outputs/pair.wasm";

// TOKENS
pub const NATIVE_TOKEN: &str = "str:NTV-123456";
pub const LS_TOKEN: &str = "str:LST-123456";
pub const BASE_TOKEN: &str = "str:BASE-123456";
pub const G_TOKEN: &str = "str:GTK-123456";
pub const TOKEN_IDS: [&str; 4] = [NATIVE_TOKEN, LS_TOKEN, BASE_TOKEN, G_TOKEN];

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

pub struct FuzzerData {
    block_state: BlockState,
    world: ScenarioWorld,
    users: Vec<String>,
}

impl FuzzerData {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut block_state = BlockState::new(500_000, 1_200);
        let mut world = ScenarioWorld::new();
        let [native_token, ls_token, base_token, g_token] = TOKEN_IDS;

        world.set_current_dir_from_workspace("fuzz");

        world.register_contract(G_TOKEN_WASM, g_token::ContractBuilder);
        world.register_contract(ROUTER_WASM, router::ContractBuilder);
        world.register_contract(PAIR_WASM, pair::ContractBuilder);

        let router_addr = "sc:router";
        let pair_temp_addr = "sc:pair_temp";

        let g_token_code = world.code_expression(G_TOKEN_WASM);
        let router_code = world.code_expression(ROUTER_WASM);
        let pair_code = world.code_expression(PAIR_WASM);

        world
            .set_state_step(
                block_state
                    .move_block_round(0, None)
                    .put_account(
                        OWNER,
                        Account::new()
                            .balance(big_to_pow_18(1_000_000u32))
                            .esdt_balance(native_token, big_to_pow_18(7_000u32))
                            .esdt_balance(ls_token, big_to_pow_18(200_000u32))
                            .esdt_balance(base_token, big_to_pow_18(32_000_000u32)),
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
            // Deploy Router
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
                    .argument(base_token),
            )
            .sc_call(
                g_token_call_step("registerGToken", OWNER)
                    .argument("str:GToken")
                    .argument(g_token)
                    .argument("18")
                    .egld_value("5,000,000,000,000,000,000"),
            )
            .set_state_step(SetStateStep::new().put_account(
                G_TOKEN_ADDR,
                update_sc_acc(&g_token_code, vec![("str:g_token", g_token)]).esdt_roles(
                    g_token,
                    vec!["ESDTRoleLocalMint".into(), "ESDTRoleLocalBurn".into()],
                ),
            ));

        let create_lp_pool = |world: &mut ScenarioWorld,
                              g_pair: &str,
                              g_pair_deposit: num_bigint::BigUint,
                              base_pair_deposit: num_bigint::BigUint| {
            let pair_ticker = &g_pair[..7];
            let lp_token = &format!("{}LP-123456", pair_ticker)[..];
            let pair_addr = &pair_addr_from_token_id(g_pair)[..];

            let router_current_nonce = world
                .get_state()
                .accounts
                .get(&AddressValue::from(router_addr).to_vm_address())
                .unwrap()
                .nonce;

            world
                .set_state_step(SetStateStep::new().new_address(
                    router_addr,
                    router_current_nonce,
                    pair_addr,
                ))
                .sc_call(g_token_call_step("router_create_pair", OWNER).argument(g_pair))
                .sc_call(
                    g_token_call_step("router_issue_lp", OWNER)
                        .argument(g_pair)
                        .argument("str:LPToken")
                        .argument(lp_token)
                        .egld_value(big_to_pow_18(5u32)),
                )
                .set_state_step(
                    SetStateStep::new().put_account(
                        pair_addr,
                        update_sc_acc(
                            &pair_code,
                            vec![
                                ("str:lpTokenIdentifier", lp_token),
                                ("str:first_token_id", g_pair),
                                ("str:second_token_id", base_token),
                            ],
                        )
                        .esdt_roles(
                            lp_token,
                            vec!["ESDTRoleLocalMint".into(), "ESDTRoleLocalBurn".into()],
                        ),
                    ),
                )
                .sc_call(g_token_call_step("router_set_lp_local_roles", OWNER).argument(g_pair))
                .sc_call(
                    g_token_call_step("pair_add_initial_liquidity", OWNER)
                        .argument(g_pair)
                        .esdt_transfer(g_pair, 0, g_pair_deposit)
                        .esdt_transfer(base_token, 0, base_pair_deposit),
                )
                .sc_call(
                    ScCallStep::new()
                        .to(router_addr)
                        .from(OWNER)
                        .function("resume")
                        .argument(pair_addr),
                );
        };

        create_lp_pool(
            &mut world,
            native_token,
            big_to_pow_18(1000u32),
            big_to_pow_18(100_000u32),
        );
        create_lp_pool(
            &mut world,
            ls_token,
            big_to_pow_18(3_000u32),
            big_to_pow_18(100_000u32),
        );

        // Send all GTokens to OWNER so we can create an lp pool for it
        let send_g_tokens_to_owner = |world: &mut ScenarioWorld| {
            let g_token_supply = world
                .get_state()
                .accounts
                .get(&AddressValue::from(G_TOKEN_ADDR).to_vm_address())
                .unwrap()
                .esdt
                .get_esdt_balance(g_token[4..].as_bytes(), 0);
            assert!(
                g_token_supply > 0u32.into(),
                "No GToken bal for GToken contract"
            );
            world.transfer_step(
                TransferStep::new()
                    .from(G_TOKEN_ADDR)
                    .to(OWNER)
                    .esdt_transfer(g_token, 0, &g_token_supply),
            );

            g_token_supply
        };

        let g_token_supply = send_g_tokens_to_owner(&mut world);
        create_lp_pool(&mut world, g_token, g_token_supply.clone(), g_token_supply);
        send_g_tokens_to_owner(&mut world);

        Self {
            world,
            block_state,
            users: vec![],
        }
    }

    pub fn seed_accounts(mut self, max_users: u32) -> Self {
        let mut rng = rand::thread_rng();

        for token_id in TOKEN_IDS {
            let mut users_count = 1;
            let max_ratio = 100u32;
            while let Some(bal) = self.get_esdt_balance_not_zero(OWNER, &token_id[4..]) {
                let ratio = (users_count * max_ratio) / max_users;
                let amt_to_share = if ratio < max_ratio {
                    let mut min = 5;
                    let mut max = ratio;
                    if min > max {
                        core::mem::swap(&mut min, &mut max);
                    }

                    bal * rng.gen_range(min..=max) / max_ratio
                } else {
                    bal
                };

                let user_addr = format!("address:user{users_count}");

                if self.get_maybe_account_data(&user_addr).is_none() {
                    self.world.set_state_step(
                        SetStateStep::new().put_account(&user_addr[..], Account::new()),
                    );

                    self.users.push(user_addr.clone());
                }

                self.send_token(OWNER, &user_addr, token_id, amt_to_share);

                users_count += 1;
            }
        }

        self
    }

    pub fn move_block_round(&mut self) {
        self.world
            .set_state_step(self.block_state.move_block_round(1, None));
    }

    pub fn get_users(&mut self) -> Vec<String> {
        self.users.clone()
    }

    pub fn send_token(
        &mut self,
        from: &str,
        to: &str,
        token_id: &str,
        amount: num_bigint::BigUint,
    ) {
        self.world.transfer_step(
            TransferStep::new()
                .from(from)
                .to(to)
                .esdt_transfer(token_id, 0, amount),
        );
    }

    pub fn swap_fixed_input(&mut self, user: &str, pair_addr: &str, token_in: TokenTransfer) {
        let pair_whitebox = WhiteboxContract::new(pair_addr, pair::contract_obj);

        let mut token_out = TokenTransfer::default();

        self.world.whitebox_query(&pair_whitebox, |sc| {
            let first_id = sc.first_token_id().get().to_string();
            let second_id = sc.second_token_id().get().to_string();
            let total_fee_percent = sc.total_fee_percent().get();

            let id_in = &token_in.token_id[4..];

            let id_out = if first_id == id_in {
                second_id
            } else {
                first_id
            };

            // Apply slippage on amount in
            let amount_in = &token_in.amount * (pair::config::MAX_PERCENTAGE - total_fee_percent)
                / pair::config::MAX_PERCENTAGE;

            let bytes = sc
                .get_amount_out_view(TokenIdentifier::from(id_in), amount_in.into())
                .to_bytes_be();

            token_out.amount = num_bigint::BigUint::from_bytes_be(bytes.as_slice());
            // token_out.amount = 1u32.into();
            token_out.token_id = format!("str:{id_out}");
        });

        write(format!("Price before: {}", self.price_feed(&pair_whitebox)));
        self.world.sc_call(
            ScCallStep::new()
                .from(user)
                .to(pair_addr)
                .function("swapTokensFixedInput")
                .argument(token_out.token_id)
                .argument(BytesValue::from(token_out.amount.to_str_radix(10)))
                .esdt_transfer(token_in.token_id, 0, token_in.amount),
        );
        write(format!(
            "Price after:  {}\n",
            self.price_feed(&pair_whitebox)
        ));
    }

    fn price_feed(
        &mut self,
        pair_whitebox: &WhiteboxContract<pair::ContractObj<DebugApi>>,
    ) -> String {
        let mut feed = "".to_string();

        self.world.whitebox_query(pair_whitebox, |sc| {
            let first_id = sc.first_token_id().get().to_string();
            let second_id = sc.second_token_id().get().to_string();

            let first_reserve = sc.pair_reserve(&(first_id[..].into())).get();
            let second_reserve = sc.pair_reserve(&(second_id[..].into())).get();

            let price = (second_reserve * 10_000u32 / first_reserve)
                .to_u64()
                .unwrap() as f64
                / 10_000.;

            let first_id = get_ticker(&first_id);
            let second_id = get_ticker(&second_id);

            feed = format!("{first_id}/{second_id} {price}");
        });

        feed
    }

    pub fn swap_fixed_output(
        &mut self,
        user: &str,
        pair_addr: &str,
        token_in: TokenTransfer,
        token_out: TokenTransfer,
    ) {
        self.world.sc_call(
            ScCallStep::new()
                .from(user)
                .to(pair_addr)
                .function("swapTokensFixedOutput")
                .argument(token_out.token_id)
                .argument(BytesValue::from(token_out.amount.to_str_radix(10)))
                .esdt_transfer(token_in.token_id, 0, token_in.amount),
        );
    }

    pub fn mint(
        &mut self,
        user: &str,
        collateral_id: &str,
        amount: num_bigint::BigUint,
        g_pair: Option<&str>,
    ) {
        let mut step = ScCallStep::new()
            .from(user)
            .to(G_TOKEN_ADDR)
            .function("mint")
            .argument("99,00")
            .esdt_transfer(collateral_id, 0, amount);

        if let Some(g_pair) = g_pair {
            step = step.argument(g_pair);
        }

        self.world.sc_call(step);
    }

    pub fn get_maybe_account_data(&mut self, address: &str) -> Option<&AccountData> {
        self.world
            .get_state()
            .accounts
            .get(&AddressValue::from(address).to_vm_address())
    }

    pub fn get_account_data(&mut self, address: &str) -> &AccountData {
        self.get_maybe_account_data(address)
            .unwrap_or_else(|| panic!("Account for {address} not found"))
    }

    pub fn get_esdt_balance(&mut self, address: &str, token_id: &str) -> num_bigint::BigUint {
        self.get_account_data(address)
            .esdt
            .get_esdt_balance(token_id.as_bytes(), 0)
    }

    pub fn get_esdt_balance_not_zero(
        &mut self,
        address: &str,
        token_id: &str,
    ) -> Option<num_bigint::BigUint> {
        let bal = self.get_esdt_balance(address, token_id);

        if bal == 0u32.into() {
            None
        } else {
            Some(bal)
        }
    }

    pub fn arbitrage<const N: usize>(&mut self, token_ids: [&str; N]) -> Vec<(String, f64, f64)> {
        let mut rng = rand::thread_rng();

        let user = self
            .users
            .get(rng.gen_range(0..self.users.len()))
            .expect("user get out of range")
            .clone();

        let g_token_whitebox = WhiteboxContract::new(G_TOKEN_ADDR, g_token::contract_obj);

        let precission = 10_000f64;
        let mut price_feeds = vec![];
        let collect_price_feed =
            |f: &mut FuzzerData, token_id: &str, price_feeds: &mut Vec<(String, u32, u16)>| {
                let pair_whitebox = get_pair_whitebox(token_id);
                let feed = f.price_feed(&pair_whitebox);
                let feed = feed.split(' ').collect::<Vec<&str>>();

                let pair = feed.first().expect("No pair found").to_string();
                let price = feed
                    .get(1)
                    .expect("No price found")
                    .parse::<f64>()
                    .expect("unable to parse price")
                    * precission;

                let mut g_ratio = 0;
                f.world.whitebox_query(&g_token_whitebox, |sc| {
                    g_ratio = sc.pair_g_ratio(&TokenIdentifier::from(&token_id[4..]));
                });

                price_feeds.push((pair, price as u32, g_ratio));
            };

        for token_id in token_ids {
            collect_price_feed(self, token_id, &mut price_feeds);
        }

        price_feeds.sort_by(|a, b| b.1.cmp(&a.1));

        let user = &user[..];
        for (pair, price, g_ratio) in price_feeds.clone().into_iter() {
            let token_id = *pair.split('/').collect::<Vec<&str>>().first().unwrap();
            let token_id = format!("str:{token_id}-123456");

            let should_trade = rng.gen_bool(0.55);
            if should_trade {
                let (token_in_id, token_out_id) = if price > precission as u32 {
                    (&token_id[..], BASE_TOKEN)
                } else {
                    (BASE_TOKEN, &token_id[..])
                };

                let amount = self.get_esdt_balance(user, &token_in_id[4..])
                    * rng.gen_range(3..7u32)
                    / 100u32;

                if amount >= 4_000u32.into() {
                    let pair_addr = pair_addr_from_token_id(if token_in_id == BASE_TOKEN {
                        token_out_id
                    } else {
                        token_in_id
                    });

                    write(format!(
                        "{user}: Buy: {}, Sell: {}",
                        get_ticker(token_out_id),
                        get_ticker(token_in_id)
                    ));

                    self.swap_fixed_input(
                        user,
                        &pair_addr,
                        TokenTransfer {
                            token_id: token_in_id.into(),
                            amount,
                        },
                    );
                }
            } else {
                let mut try_mint = |collateral_id: &str| {
                    let collateral_id = if G_TOKEN.contains(collateral_id) {
                        BASE_TOKEN
                    } else {
                        collateral_id
                    };
                    let amount = self.get_esdt_balance(user, &collateral_id[4..])
                        * rng.gen_range(3..7u32)
                        / 100u32;

                    let threshold = &amount / 2u32;
                    let threshold = if collateral_id != BASE_TOKEN {
                        threshold * price / precission as u32
                    } else {
                        threshold * precission as u32 / price
                    } * 1u32
                        / 100u32
                        / price;

                    if threshold >= 400u32.into() && amount >= 4_000u32.into() {
                        write(format!(
                            "{user}: {pair}, Mint with: {}; price: {} amount {}",
                            get_ticker(collateral_id),
                            price as f64 / precission,
                            amount
                        ));
                        self.mint(user, collateral_id, amount, Some(&token_id));
                        true
                    } else {
                        false
                    }
                };

                // We should mint when working with only one pair
                if (token_ids.len() < 2 || g_ratio <= g_token::config::G_RATIO_BALANCE_FACTOR)
                    && !try_mint(&token_id)
                {
                    try_mint(BASE_TOKEN);
                }
            }
        }

        // Updadte price feed
        let mut price_feeds = vec![];
        for token_id in token_ids {
            collect_price_feed(self, token_id, &mut price_feeds);
        }
        let price_feeds = price_feeds
            .into_iter()
            .map(|(pair, price, ratio)| {
                (
                    pair,
                    price as f64 / precission,
                    ratio as f64 / g_token::config::G_RATIO_BALANCE_FACTOR as f64,
                )
            })
            .collect::<Vec<(String, f64, f64)>>();
        write(format!("Price Feeds {:#?}", price_feeds));

        price_feeds
    }

    pub fn dump_account(&mut self, address: &str) {
        let account = self.get_account_data(address);
        let mut user = User {
            address: address.into(),
            wallet: TokenBalance::new(),
        };

        for token_id in TOKEN_IDS {
            let ticker = get_ticker(token_id);
            let balance = account.esdt.get_esdt_balance(token_id[4..].as_bytes(), 0);

            let is_new = user.wallet.insert(ticker.into(), balance).is_none();
            assert!(is_new, "duplicate ticker {} for {}", ticker, address);
        }

        write(format!("{:#?}\n", user));
        drop(user.address);
    }
}

#[derive(Default, Debug)]
pub struct TokenTransfer {
    pub token_id: String,
    pub amount: num_bigint::BigUint,
}

fn get_ticker(id: &str) -> &str {
    if &id[..4] == "str:" { &id[4..] } else { id }
        .split('-')
        .collect::<Vec<&str>>()[0]
}

pub fn pair_addr_from_token_id(token_id: &str) -> String {
    format!("sc:{}_pair", get_ticker(token_id))
}

fn get_pair_whitebox(token_id: &str) -> WhiteboxContract<pair::ContractObj<DebugApi>> {
    let pair_addr = pair_addr_from_token_id(token_id);

    WhiteboxContract::new(pair_addr, pair::contract_obj)
}
