multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FundsModule {
    #[storage_mapper("pending_delegation")]
    fn pending_delegation(&self) -> SingleValueMapper<BigUint>;
}
