multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait DSCProxy {
    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self);

    #[endpoint(unDelegate)]
    fn undelegate(&self, egld_amount: BigUint);

    #[endpoint(withdraw)]
    fn withdraw(&self);

    #[endpoint(claimRewards)]
    fn claim_rewards(&self);

    #[endpoint(addNodes)]
    fn add_nodes(
        &self,
        bls_signatures_pair: &MultiValueEncoded<MultiValue2<ManagedBuffer, ManagedBuffer>>,
    );

    #[endpoint(stakeNodes)]
    fn stake_nodes(&self, keys: &MultiValueEncoded<ManagedBuffer>);

    #[endpoint(unStakeNodes)]
    fn un_stake_nodes(&self, keys: &MultiValueEncoded<ManagedBuffer>);

    #[endpoint(removeNodes)]
    fn remove_nodes(&self, keys: &MultiValueEncoded<ManagedBuffer>);

    #[endpoint(correctNodesStatus)]
    fn correct_nodes_status(&self);

    #[view(getUserActiveStake)]
    fn get_user_active_stake(&self, user_address: ManagedAddress) -> BigUint;

    #[view(getTotalActiveStake)]
    fn get_total_active_stake(&self) -> BigUint;
}
