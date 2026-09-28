use crate::validator::Validator;
use prism_core::{Block, BlockHeader, BlockchainState, Transaction};
use prism_crypto::{Keypair, Signature};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("No active validators available")]
    NoValidators,
    #[error("Unauthorized block producer: expected {expected}, got {got}")]
    UnauthorizedProducer { expected: String, got: String },
    #[error("Invalid block signature")]
    InvalidBlockSignature,
    #[error("Invalid previous hash")]
    InvalidPrevHash,
    #[error("Invalid block height")]
    InvalidHeight,
}

/// Proof-of-Attested-Context (PoAC) Consensus Engine
pub struct PoacConsensusEngine {
    pub validators: Vec<Validator>,
}

impl PoacConsensusEngine {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    pub fn add_validator(&mut self, validator: Validator) {
        self.validators.push(validator);
    }

    /// Selects the designated block producer for a given block height
    pub fn select_leader(&self, height: u64) -> Option<&Validator> {
        let active: Vec<&Validator> = self.validators.iter().filter(|v| v.is_active).collect();
        if active.is_empty() {
            return None;
        }
        let index = (height as usize) % active.len();
        Some(active[index])
    }

    /// Produce a new signed block
    pub fn produce_block(
        &self,
        validator_keypair: &Keypair,
        state: &BlockchainState,
        transactions: Vec<Transaction>,
    ) -> Result<Block, ConsensusError> {
        let next_height = state.block_height + 1;
        let designated = self.select_leader(next_height)
            .ok_or(ConsensusError::NoValidators)?;

        if designated.pubkey != validator_keypair.public_key() {
            return Err(ConsensusError::UnauthorizedProducer {
                expected: designated.pubkey.to_hex(),
                got: validator_keypair.public_key().to_hex(),
            });
        }

        let tx_root = Block::compute_tx_root(&transactions);
        let state_root = state.compute_state_root();

        let mut header = BlockHeader {
            height: next_height,
            prev_hash: state.latest_block_hash,
            timestamp: chrono::Utc::now().timestamp(),
            tx_root,
            state_root,
            validator: validator_keypair.public_key(),
            validator_signature: Signature([0u8; 64]),
        };

        // Sign block header
        let signing_bytes = header.signing_bytes();
        let sig = validator_keypair.sign(&signing_bytes);
        header.validator_signature = sig;

        Ok(Block {
            header,
            transactions,
        })
    }

    /// Validate block proposal before applying to state
    pub fn validate_block_header(
        &self,
        block: &Block,
        state: &BlockchainState,
    ) -> Result<(), ConsensusError> {
        if block.header.height != state.block_height + 1 {
            return Err(ConsensusError::InvalidHeight);
        }
        if block.header.prev_hash != state.latest_block_hash {
            return Err(ConsensusError::InvalidPrevHash);
        }

        let designated = self.select_leader(block.header.height)
            .ok_or(ConsensusError::NoValidators)?;

        if designated.pubkey != block.header.validator {
            return Err(ConsensusError::UnauthorizedProducer {
                expected: designated.pubkey.to_hex(),
                got: block.header.validator.to_hex(),
            });
        }

        // Verify cryptographic signature of validator
        let signing_bytes = block.header.signing_bytes();
        block.header.validator.verify(&signing_bytes, &block.header.validator_signature)
            .map_err(|_| ConsensusError::InvalidBlockSignature)?;

        Ok(())
    }
}
