pub mod engine;
pub mod validator;

pub use engine::{ConsensusError, PoacConsensusEngine};
pub use validator::Validator;

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::BlockchainState;
    use prism_crypto::Keypair;

    #[test]
    fn test_poac_block_production_and_validation() {
        let mut engine = PoacConsensusEngine::new();
        let val_kp = Keypair::generate();
        let validator = Validator::new(val_kp.public_key(), 10_000);
        engine.add_validator(validator);

        let state = BlockchainState::new();

        // Produce block 1
        let block_res = engine.produce_block(&val_kp, &state, vec![]);
        assert!(block_res.is_ok());

        let block = block_res.unwrap();
        assert_eq!(block.header.height, 1);
        assert_eq!(block.header.validator, val_kp.public_key());

        // Validate block header against state
        let val_res = engine.validate_block_header(&block, &state);
        assert!(val_res.is_ok());

        // Test rejection by imposter validator
        let imposter_kp = Keypair::generate();
        let imposter_res = engine.produce_block(&imposter_kp, &state, vec![]);
        assert!(imposter_res.is_err());
    }
}
