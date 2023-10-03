use crate::PairInfo;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait Config {
    #[inline]
    fn add_dust(&self, token_id: &TokenIdentifier, amt: BigUint) {
        self.token_dust(token_id).update(|old| *old += amt);
    }

    fn get_pair_info(&self, g_pair_id: &TokenIdentifier) -> PairInfo<Self::Api> {
        self.pair_map()
            .get(g_pair_id)
            .unwrap_or_else(|| sc_panic!("unrecognized g_pair token"))
    }

    #[storage_mapper("token_dust")]
    fn token_dust(&self, id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("pair_map")]
    fn pair_map(&self) -> MapMapper<TokenIdentifier, PairInfo<Self::Api>>;

    #[storage_mapper("base_pair")]
    fn base_pair(&self) -> FungibleTokenMapper;
}
