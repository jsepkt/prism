use prism_crypto::PublicKey;
use serde::{Deserialize, Serialize};

/// Registered validator node participating in PoAC consensus
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: PublicKey,
    pub stake: u64,
    pub is_active: bool,
    pub is_slashed: bool,
    pub blocks_produced: u64,
    pub slashed_stake: u64,
}

impl Validator {
    pub fn new(pubkey: PublicKey, stake: u64) -> Self {
        Self {
            pubkey,
            stake,
            is_active: true,
            is_slashed: false,
            blocks_produced: 0,
            slashed_stake: 0,
        }
    }

    /// Slash validator for equivocation / double signing
    pub fn slash(&mut self, penalty_amount: u64) {
        let penalty = penalty_amount.min(self.stake);
        self.stake -= penalty;
        self.slashed_stake += penalty;
        self.is_active = false;
        self.is_slashed = true;
    }
}
