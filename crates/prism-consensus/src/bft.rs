use crate::validator::Validator;
use prism_crypto::{CryptoError, Hash, Keypair, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Minimum required staking balance to participate as an active PoAC consensus validator (100,000 PRISM)
pub const MIN_VALIDATOR_STAKE: u64 = 100_000;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BftError {
    #[error("Validator does not meet minimum stake requirement of {required} PRISM (has {actual})")]
    InsufficientStake { required: u64, actual: u64 },
    #[error("Validator is not in active validator set: {0}")]
    UnknownValidator(String),
    #[error("Validator is slashed and barred from consensus: {0}")]
    ValidatorSlashed(String),
    #[error("Invalid vote signature")]
    InvalidVoteSignature,
    #[error("Equivocation detected: validator double-signed differing proposals at height {height}, round {round}")]
    EquivocationDetected { height: u64, round: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteStage {
    Prevote,
    Precommit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BftVote {
    pub stage: VoteStage,
    pub height: u64,
    pub round: u32,
    pub block_hash: Hash,
    pub validator: PublicKey,
    pub signature: Signature,
}

impl BftVote {
    pub fn new(
        stage: VoteStage,
        height: u64,
        round: u32,
        block_hash: Hash,
        keypair: &Keypair,
    ) -> Self {
        let mut vote = Self {
            stage,
            height,
            round,
            block_hash,
            validator: keypair.public_key(),
            signature: Signature([0u8; 64]),
        };
        let msg = vote.signing_bytes();
        vote.signature = keypair.sign(&msg);
        vote
    }

    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        let stage_byte: u8 = match self.stage {
            VoteStage::Prevote => 1,
            VoteStage::Precommit => 2,
        };
        bytes.push(stage_byte);
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.round.to_le_bytes());
        bytes.extend_from_slice(self.block_hash.as_bytes());
        bytes
    }

    pub fn verify_signature(&self) -> Result<(), CryptoError> {
        let msg = self.signing_bytes();
        self.validator.verify(&msg, &self.signature)
    }
}

/// Cryptographic proof of validator equivocation (double-signing conflicting blocks)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivocationProof {
    pub vote_a: BftVote,
    pub vote_b: BftVote,
}

impl EquivocationProof {
    pub fn new(vote_a: BftVote, vote_b: BftVote) -> Self {
        Self { vote_a, vote_b }
    }

    /// Validates the equivocation proof and returns the guilty validator's public key
    pub fn verify(&self) -> Result<PublicKey, BftError> {
        if self.vote_a.validator != self.vote_b.validator {
            return Err(BftError::InvalidVoteSignature);
        }
        if self.vote_a.height != self.vote_b.height
            || self.vote_a.round != self.vote_b.round
            || self.vote_a.stage != self.vote_b.stage
        {
            return Err(BftError::InvalidVoteSignature);
        }
        if self.vote_a.block_hash == self.vote_b.block_hash {
            return Err(BftError::InvalidVoteSignature);
        }

        self.vote_a
            .verify_signature()
            .map_err(|_| BftError::InvalidVoteSignature)?;
        self.vote_b
            .verify_signature()
            .map_err(|_| BftError::InvalidVoteSignature)?;

        Ok(self.vote_a.validator)
    }
}

/// Round state accumulator tracking prevotes and precommits towards 2/3+ BFT quorum
#[derive(Clone, Debug)]
pub struct BftRoundTracker {
    pub height: u64,
    pub round: u32,
    pub prevotes: HashMap<PublicKey, BftVote>,
    pub precommits: HashMap<PublicKey, BftVote>,
}

impl BftRoundTracker {
    pub fn new(height: u64, round: u32) -> Self {
        Self {
            height,
            round,
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
        }
    }

    /// Compute total active stake and the 2/3+ quorum threshold (> 66.66%)
    pub fn compute_quorum_threshold(validators: &[Validator]) -> (u64, u64) {
        let total_stake: u64 = validators
            .iter()
            .filter(|v| v.is_active && !v.is_slashed && v.stake >= MIN_VALIDATOR_STAKE)
            .map(|v| v.stake)
            .sum();

        let threshold = (total_stake * 2) / 3 + 1;
        (total_stake, threshold)
    }

    /// Record a verified vote and check if 2/3+ quorum is reached for a block proposal
    pub fn record_vote(
        &mut self,
        vote: BftVote,
        validators: &[Validator],
    ) -> Result<Option<Hash>, BftError> {
        if vote.height != self.height || vote.round != self.round {
            return Ok(None);
        }

        // 1. Verify validator identity and active standing
        let val_info = validators
            .iter()
            .find(|v| v.pubkey == vote.validator)
            .ok_or_else(|| BftError::UnknownValidator(vote.validator.to_hex()))?;

        if val_info.is_slashed {
            return Err(BftError::ValidatorSlashed(vote.validator.to_hex()));
        }

        if val_info.stake < MIN_VALIDATOR_STAKE {
            return Err(BftError::InsufficientStake {
                required: MIN_VALIDATOR_STAKE,
                actual: val_info.stake,
            });
        }

        // 2. Verify signature
        vote.verify_signature()
            .map_err(|_| BftError::InvalidVoteSignature)?;

        // 3. Equivocation check
        let map = match vote.stage {
            VoteStage::Prevote => &mut self.prevotes,
            VoteStage::Precommit => &mut self.precommits,
        };

        if let Some(existing) = map.get(&vote.validator) {
            if existing.block_hash != vote.block_hash {
                return Err(BftError::EquivocationDetected {
                    height: vote.height,
                    round: vote.round,
                });
            }
        } else {
            map.insert(vote.validator, vote.clone());
        }

        // 4. Calculate accumulated stake for the voted block_hash
        let (_, threshold) = Self::compute_quorum_threshold(validators);

        let mut stake_by_hash: HashMap<Hash, u64> = HashMap::new();
        for (pubkey, v) in map.iter() {
            if let Some(val) = validators.iter().find(|x| &x.pubkey == pubkey) {
                *stake_by_hash.entry(v.block_hash).or_insert(0) += val.stake;
            }
        }

        if let Some(&accumulated) = stake_by_hash.get(&vote.block_hash) {
            if accumulated >= threshold {
                return Ok(Some(vote.block_hash));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bft_vote_signing_and_verification() {
        let kp = Keypair::generate();
        let block_hash = Hash([7u8; 32]);
        let vote = BftVote::new(VoteStage::Prevote, 1, 0, block_hash, &kp);

        assert!(vote.verify_signature().is_ok());
        assert_eq!(vote.height, 1);
        assert_eq!(vote.round, 0);
        assert_eq!(vote.block_hash, block_hash);
    }

    #[test]
    fn test_bft_quorum_aggregation_and_threshold() {
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();
        let kp3 = Keypair::generate();
        let kp4 = Keypair::generate();

        let validators = vec![
            Validator::new(kp1.public_key(), 100_000),
            Validator::new(kp2.public_key(), 100_000),
            Validator::new(kp3.public_key(), 100_000),
            Validator::new(kp4.public_key(), 100_000),
        ];

        let (total_stake, threshold) = BftRoundTracker::compute_quorum_threshold(&validators);
        assert_eq!(total_stake, 400_000);
        assert_eq!(threshold, 266_667); // > 2/3 of 400,000 is 266,667

        let mut tracker = BftRoundTracker::new(1, 0);
        let block_hash = Hash([42u8; 32]);

        // Vote 1 (100k) - below quorum
        let v1 = BftVote::new(VoteStage::Prevote, 1, 0, block_hash, &kp1);
        assert_eq!(tracker.record_vote(v1, &validators).unwrap(), None);

        // Vote 2 (200k) - below quorum
        let v2 = BftVote::new(VoteStage::Prevote, 1, 0, block_hash, &kp2);
        assert_eq!(tracker.record_vote(v2, &validators).unwrap(), None);

        // Vote 3 (300k >= 266,667) - QUORUM ACHIEVED!
        let v3 = BftVote::new(VoteStage::Prevote, 1, 0, block_hash, &kp3);
        assert_eq!(tracker.record_vote(v3, &validators).unwrap(), Some(block_hash));
    }

    #[test]
    fn test_bft_equivocation_detection_and_slashing_proof() {
        let kp = Keypair::generate();
        let hash_a = Hash([1u8; 32]);
        let hash_b = Hash([2u8; 32]);

        let vote_a = BftVote::new(VoteStage::Prevote, 5, 0, hash_a, &kp);
        let vote_b = BftVote::new(VoteStage::Prevote, 5, 0, hash_b, &kp);

        let proof = EquivocationProof::new(vote_a.clone(), vote_b.clone());
        let guilty_validator = proof.verify().expect("Proof must be valid");
        assert_eq!(guilty_validator, kp.public_key());

        // Test in round tracker with 2 validators (total stake 200k, threshold 133,334)
        let kp2 = Keypair::generate();
        let validators = vec![
            Validator::new(kp.public_key(), 100_000),
            Validator::new(kp2.public_key(), 100_000),
        ];
        let mut tracker = BftRoundTracker::new(5, 0);
        assert_eq!(tracker.record_vote(vote_a, &validators).unwrap(), None);

        // Submitting conflicting vote from same validator must trigger EquivocationDetected
        let err = tracker.record_vote(vote_b, &validators).unwrap_err();
        assert_eq!(
            err,
            BftError::EquivocationDetected {
                height: 5,
                round: 0
            }
        );
    }

    #[test]
    fn test_bft_minimum_stake_enforcement() {
        let kp = Keypair::generate();
        // Only 50,000 PRISM (below 100k minimum)
        let validators = vec![Validator::new(kp.public_key(), 50_000)];

        let mut tracker = BftRoundTracker::new(1, 0);
        let vote = BftVote::new(VoteStage::Prevote, 1, 0, Hash([1u8; 32]), &kp);

        let err = tracker.record_vote(vote, &validators).unwrap_err();
        assert_eq!(
            err,
            BftError::InsufficientStake {
                required: MIN_VALIDATOR_STAKE,
                actual: 50_000,
            }
        );
    }
}
