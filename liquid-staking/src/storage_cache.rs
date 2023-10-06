multiversx_sc::imports!();

pub trait StorageCacheTraitBounds =
    crate::liquidity_pool::LiquidityPoolModule + crate::funds::FundsModule;

pub struct StorageCache<'a, C: StorageCacheTraitBounds> {
    sc_ref: &'a C,
    pub ls_token_id: TokenIdentifier<C::Api>,
    pub ls_token_supply: BigUint<C::Api>,
    pub delegated_egld: BigUint<C::Api>,
    pub pending_delegation: BigUint<C::Api>,
    pub reward_per_share: BigUint<C::Api>,
}

impl<'a, C: StorageCacheTraitBounds> StorageCache<'a, C> {
    pub fn new(sc_ref: &'a C) -> Self {
        StorageCache {
            ls_token_id: sc_ref.ls_token().get_token_id(),
            ls_token_supply: sc_ref.ls_token_supply().get(),
            delegated_egld: sc_ref.delegated_egld().get(),
            pending_delegation: sc_ref.pending_delegation().get(),
            reward_per_share: sc_ref.reward_per_share().get(),
            sc_ref,
        }
    }

    pub fn commit(self) {
        // Just like calling drop with self
    }
}

impl<'a, C: StorageCacheTraitBounds> Drop for StorageCache<'a, C> {
    fn drop(&mut self) {
        // commit changes to storage for mutable fields
        self.sc_ref.ls_token_supply().set(&self.ls_token_supply);
        self.sc_ref.delegated_egld().set(&self.delegated_egld);
        self.sc_ref
            .pending_delegation()
            .set(&self.pending_delegation);
        self.sc_ref.reward_per_share().set(&self.reward_per_share);
    }
}
