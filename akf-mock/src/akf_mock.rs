#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait AkfMock {
    #[init]
    fn init(&self) {}

    #[payable("EGLD")]
    #[endpoint]
    fn add_uru_egld(&self, _id: u64) {}
}
