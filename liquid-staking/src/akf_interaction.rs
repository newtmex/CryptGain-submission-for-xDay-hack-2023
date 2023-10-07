multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AkfInteraction {
    fn call_akf(&self) -> akf::Proxy<Self::Api> {
        let to = self.get_akf_address();
        self.akf_proxy_obj(to)
    }

    fn set_akf_address(&self, addr: &ManagedAddress) {
        self.require_akf_addr_valid(addr);

        self.akf_address().set_if_empty(addr);
    }

    fn require_akf_addr_valid(&self, addr: &ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(addr),
            "AKF address is not a valid SC address"
        );
    }

    fn get_akf_address(&self) -> ManagedAddress {
        require!(!self.akf_address().is_empty(), "AKF Address not set");

        let addr = self.akf_address().get();
        self.require_akf_addr_valid(&addr);

        addr
    }

    #[proxy]
    fn akf_proxy_obj(&self, addr: ManagedAddress) -> akf::Proxy<Self::Api>;

    #[storage_mapper("akf_address")]
    fn akf_address(&self) -> SingleValueMapper<ManagedAddress>;
}

pub mod akf {

    #[multiversx_sc::proxy]
    pub trait AkfProxy {
        #[endpoint]
        fn add_uru_egld(&self, id: u64);
    }
}
