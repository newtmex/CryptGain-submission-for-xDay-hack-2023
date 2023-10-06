#![no_std]

pub mod proxy;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DelegationModule {
    fn call_dsc(&self) -> proxy::Proxy<Self::Api> {
        let to = self.get_dsc_address();
        self.dsc_proxy(to)
    }

    fn set_dsc_address(&self, addr: &ManagedAddress) {
        self.require_dsc_addr_valid(addr);

        self.dsc_address().set_if_empty(addr);
    }

    fn require_dsc_addr_valid(&self, addr: &ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(addr),
            "DSC address is not a valid SC address"
        );
    }

    #[view(getDSCAddress)]
    fn get_dsc_address(&self) -> ManagedAddress {
        require!(!self.dsc_address().is_empty(), "DSC Address not set");

        let addr = self.dsc_address().get();
        self.require_dsc_addr_valid(&addr);

        addr
    }

    #[proxy]
    fn dsc_proxy(&self, dsc_addr: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[storage_mapper("dsc::address")]
    fn dsc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
