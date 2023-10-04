use router::factory::PairTokens;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopDecode, TopEncode, NestedDecode, NestedEncode)]
pub struct PairInfo<M: ManagedTypeApi> {
    pub g_token_supply: BigUint<M>,
    pub lp_token_supply: BigUint<M>,
    pub lp_token_id: TokenIdentifier<M>,
    pub tokens: PairTokens<M>,
    pub address: ManagedAddress<M>,
}

impl<M: ManagedTypeApi> PairInfo<M> {
    pub fn new(tokens: PairTokens<M>, address: ManagedAddress<M>) -> Self {
        Self {
            g_token_supply: 0u32.into(),
            lp_token_supply: 0u32.into(),
            lp_token_id: Self::placeholder_lp_id(),
            tokens,
            address,
        }
    }

    pub fn check_lp_id(&self, token_id: &TokenIdentifier<M>) -> bool {
        &self.lp_token_id == token_id
    }

    /// - Reduces supply of `LP Token` and `GToken`,
    /// - Returns `LP Payment`
    fn take_tokens_from_supply(&mut self, g_amount: &BigUint<M>) -> EsdtTokenPayment<M> {
        if g_amount > &self.g_token_supply {
            M::error_api_impl().signal_error(b"GToken amount too large")
        }

        let lp_amount = g_amount * &self.lp_token_supply / &self.g_token_supply;

        self.lp_token_supply -= &lp_amount;
        self.g_token_supply -= g_amount;

        (self.lp_token_id.clone(), 0, lp_amount).into()
    }

    fn placeholder_lp_id() -> TokenIdentifier<M> {
        TokenIdentifier::from("Pending")
    }

    /// The first token that is checked becomes the lp_token_id
    fn set_or_check_lp_id(&mut self, token_id: &TokenIdentifier<M>) -> bool {
        if self.lp_token_id == Self::placeholder_lp_id() {
            self.lp_token_id = token_id.clone();

            return true;
        }

        self.check_lp_id(token_id)
    }
}

type GRatio = u16;

#[derive(NestedDecode, NestedEncode, Clone)]
pub struct GRatioExtreme<M: ManagedTypeApi>(TokenIdentifier<M>, GRatio);

/// One to three decimal places
const G_RATIO_BALANCE_FACTOR: GRatio = 1_000;

/// 0.10%
const MIN_FEE: u32 = 10;
/// 5.00%
const AVG_FEE: u32 = 5_00;
/// 65.53%
const MAX_FEE: u32 = 65_53;
/// 100.00%
const FEE_DIVISION_CONSTANT: u32 = 10_000;

#[multiversx_sc::module]
pub trait Config {
    #[inline]
    fn add_dust(&self, token_id: &TokenIdentifier, amt: BigUint) {
        self.token_dust(token_id).update(|old| *old += amt);
    }

    /// Returns user GToken amount
    fn add_g_supply(
        &self,
        g_pair: TokenIdentifier,
        g_supply: &BigUint,
        lp_payment: EsdtTokenPayment,
    ) -> BigUint {
        let user_fee_ratio = FEE_DIVISION_CONSTANT - self.pair_fee_ratio(&g_pair);

        self.g_token_supply().update(|old| *old += g_supply);
        self.pair_map().entry(g_pair).and_modify(|pair_info| {
            pair_info.g_token_supply += g_supply;

            require!(
                pair_info.set_or_check_lp_id(&lp_payment.token_identifier),
                "Pair lp token id missmatch"
            );
            pair_info.lp_token_supply += lp_payment.amount;
        });

        g_supply * user_fee_ratio / FEE_DIVISION_CONSTANT
    }

    /// Returns LP Payment
    fn remove_g_supply(
        &self,
        g_pair: TokenIdentifier,
        g_token_payment: EsdtTokenPayment,
    ) -> EsdtTokenPayment {
        let (_, __, g_token_amount) = g_token_payment.into_tuple();

        self.g_token().burn(&g_token_amount);
        self.g_token_supply().update(|old| *old -= &g_token_amount);

        let mut lp_payment = None;
        self.pair_map().entry(g_pair).and_modify(|pair_info| {
            lp_payment = Some(pair_info.take_tokens_from_supply(&g_token_amount))
        });

        lp_payment.unwrap_or_else(|| sc_panic!("Unable to make LP token payment"))
    }

    fn set_base_pair(&self, token_id: TokenIdentifier) {
        if self.base_pair().is_empty() {
            self.base_pair().set_token_id(token_id);
        }
    }

    fn get_pair_info(&self, g_pair: &TokenIdentifier) -> PairInfo<Self::Api> {
        self.pair_map()
            .get(g_pair)
            .unwrap_or_else(|| sc_panic!("unrecognized g_pair token"))
    }

    fn g_tokens_per_pair(&self) -> BigUint {
        let pair_count = self.pair_map().len();
        require!(pair_count > 0, "No available pairs");

        let g_token_supply = self.g_token_supply().get();

        let g_ratio = g_token_supply / pair_count as u32;
        if g_ratio <= 1 {
            1u32.into()
        } else {
            g_ratio
        }
    }

    fn pair_fee_ratio(&self, g_pair: &TokenIdentifier) -> u32 {
        let g_ratio = self.pair_g_ratio(g_pair);

        let (
            //
            min_in,
            max_in,
            min_out,
            max_out,
        ) = if g_ratio <= G_RATIO_BALANCE_FACTOR {
            (1, G_RATIO_BALANCE_FACTOR as u32, MIN_FEE, AVG_FEE)
        } else {
            (
                G_RATIO_BALANCE_FACTOR as u32 + 1,
                GRatio::MAX as u32,
                AVG_FEE + 1,
                MAX_FEE,
            )
        };

        math::linear_interpolation::<Self::Api, u32>(
            min_in,
            max_in,
            g_ratio as u32,
            min_out,
            max_out,
        )
    }

    #[view(getPairGRatio)]
    /// - GRatio is in three decimal places
    /// - Ranges `1..=u16::MAX`
    fn pair_g_ratio(&self, g_pair: &TokenIdentifier) -> GRatio {
        let pair_g_supply = self
            .pair_map()
            .get(g_pair)
            .unwrap_or_else(|| sc_panic!("pair info not found"))
            .g_token_supply;
        if pair_g_supply <= 1 {
            return 1;
        }

        let g_tokens_per_pair = self.g_tokens_per_pair();

        let ratio = pair_g_supply * G_RATIO_BALANCE_FACTOR as u64 / g_tokens_per_pair;

        (ratio.to_u64().unwrap_or(GRatio::MAX as u64) as GRatio).clamp(1, GRatio::MAX)
    }

    #[storage_mapper("token_dust")]
    fn token_dust(&self, id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("g_token_supply")]
    fn g_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("pair_map")]
    fn pair_map(&self) -> MapMapper<TokenIdentifier, PairInfo<Self::Api>>;

    #[storage_mapper("base_pair")]
    fn base_pair(&self) -> FungibleTokenMapper;

    #[storage_mapper("g_token")]
    fn g_token(&self) -> FungibleTokenMapper;
}
