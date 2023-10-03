use crate::config;
use router::{factory::PairTokens, ProxyTrait as _};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait RouterInteraction: config::Config {
    #[only_owner]
    #[endpoint]
    fn router_create_pair(&self, g_pair: TokenIdentifier) {
        let base_pair = self.base_pair().get_token_id();

        let address: ManagedAddress<Self::Api> = self
            .call_router()
            .create_pair_endpoint(
                &g_pair,
                &base_pair,
                ManagedAddress::zero(),
                OptionalValue::<MultiValue2<u64, u64>>::None,
                MultiValueEncoded::new(),
            )
            .execute_on_dest_context();

        let is_new = self
            .pair_map()
            .insert(
                g_pair.clone(),
                crate::PairInfo {
                    tokens: PairTokens {
                        first_token_id: g_pair,
                        second_token_id: base_pair,
                    },
                    address,
                },
            )
            .is_none();

        require!(is_new, "GPair allready exists");
    }

    #[only_owner]
    #[endpoint]
    #[payable("EGLD")]
    fn router_issue_lp(
        &self,
        g_pair: TokenIdentifier,
        lp_token_display_name: ManagedBuffer,
        lp_token_ticker: ManagedBuffer,
    ) {
        let issue_cost = self.call_value().egld_value().clone_value();
        let pair_address = self.get_pair_info(&g_pair).address;

        self.call_router()
            .issue_lp_token(pair_address, lp_token_display_name, lp_token_ticker)
            .with_egld_transfer(issue_cost)
            .transfer_execute();
    }

    #[only_owner]
    #[endpoint]
    fn router_set_lp_local_roles(&self, g_pair: TokenIdentifier) {
        let pair_address = self.get_pair_info(&g_pair).address;

        self.call_router()
            .set_local_roles(pair_address)
            .execute_on_dest_context()
    }

    fn set_router_addr(&self, addr: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&addr),
            "Invalid router address"
        );

        self.router_addr().set_if_empty(addr);
    }

    fn call_router(&self) -> router::Proxy<Self::Api> {
        let addr = self.router_addr().get();
        require!(
            self.blockchain().is_smart_contract(&addr),
            "Invalid router address"
        );

        self.router_proxy_obj(addr)
    }

    #[proxy]
    fn router_proxy_obj(&self, addr: ManagedAddress) -> router::Proxy<Self::Api>;

    #[storage_mapper("router_addr")]
    fn router_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
