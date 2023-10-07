multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DelegationInteraction {
    fn call_delegation(&self) -> delegation::Proxy<Self::Api> {
        let to = self.get_delegation_address();
        self.delegation_proxy_obj(to)
    }

    fn set_delegation_address(&self, addr: &ManagedAddress) {
        self.require_delegation_addr_valid(addr);

        self.delegation_address().set_if_empty(addr);
    }

    fn require_delegation_addr_valid(&self, addr: &ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(addr),
            "delegation address is not a valid SC address"
        );
    }

    fn get_delegation_address(&self) -> ManagedAddress {
        require!(
            !self.delegation_address().is_empty(),
            "delegation Address not set"
        );

        let addr = self.delegation_address().get();
        self.require_delegation_addr_valid(&addr);

        addr
    }

    #[proxy]
    fn delegation_proxy_obj(&self, addr: ManagedAddress) -> delegation::Proxy<Self::Api>;

    #[storage_mapper("delegation_address")]
    fn delegation_address(&self) -> SingleValueMapper<ManagedAddress>;
}

pub mod delegation {

    #[multiversx_sc::proxy]
    pub trait DelegationProxy {
        #[endpoint]
        fn claim_rewards(&self);
    }
}
