use multiversx_sc_scenario::{
    multiversx_chain_vm::world_mock::BlockInfo, scenario_model::SetStateStep,
};

pub struct BlockState {
    pub rounds_per_epoch: u64,
    pub info: BlockInfo,
}

impl BlockState {
    pub fn new(start_round: u64, rounds_per_epoch: u64) -> Self {
        let mut info = BlockInfo::new();
        info.block_round = start_round;
        info.block_epoch = start_round / rounds_per_epoch;

        Self {
            rounds_per_epoch,
            info,
        }
    }

    pub fn move_block_round(&mut self, blocks: u64, step: Option<SetStateStep>) -> SetStateStep {
        let step = match step {
            Some(step) => step,
            None => SetStateStep::new(),
        };

        let BlockInfo {
            block_round,
            block_epoch,
            ..
        } = &mut self.info;

        *block_round += blocks;
        *block_epoch = *block_round / self.rounds_per_epoch;

        self.set_and_return_step(step)
    }

    fn set_and_return_step(&self, step: SetStateStep) -> SetStateStep {
        step.block_round(self.info.block_round)
            .block_epoch(self.info.block_epoch)
    }

    pub fn move_block_epoch(&mut self, epochs: u64, step: Option<SetStateStep>) -> SetStateStep {
        let step = match step {
            Some(step) => step,
            None => SetStateStep::new(),
        };

        let BlockInfo {
            block_round,
            block_epoch,
            ..
        } = &mut self.info;

        *block_epoch += epochs;
        *block_round = *block_epoch * self.rounds_per_epoch;

        self.set_and_return_step(step)
    }
}
