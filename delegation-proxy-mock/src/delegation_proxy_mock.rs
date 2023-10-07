#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait DelegationProxyMock {
    #[init]
    fn init(&self) {}

    #[endpoint]
    fn claim_rewards(&self) {}
}
