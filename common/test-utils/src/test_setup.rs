use multiversx_sc_scenario::{
    mandos_system::run_trace::ScenarioTrace, scenario_model::{Scenario, AddressValue}, ScenarioWorld, multiversx_chain_vm::world_mock::AccountData,
};

pub trait TestSetupTrait {
    fn world(&mut self) -> &mut ScenarioWorld;

    fn trace(&mut self, name: &str, path: &str, f: impl FnOnce(&mut Self)) {
        let trace = ScenarioTrace {
            scenario_trace: Scenario {
                name: Some(name.to_string()),
                ..Default::default()
            },
            addr_to_pretty_string_map: Default::default(),
        };

        self.world().start_trace_with(trace);
        f(self);
        self.world()
            .write_scenario_trace(&format!("scenarios/{path}"));
    }

    fn get_account_data(&mut self, address_expr: &str) -> AccountData {
        let address_value = AddressValue::from(address_expr);
        self.world()
            .get_state()
            .accounts
            .get(&address_value.to_vm_address())
            .unwrap()
            .clone()
    }

    fn get_account_current_nonce(&mut self, address_expr: &str)->u64{
        self.get_account_data(address_expr).nonce
    }
}
