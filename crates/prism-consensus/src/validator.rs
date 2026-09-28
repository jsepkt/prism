use prism_crypto::PublicKey;
use serde::{Deserialize, Serialize};

/// Registered validator node participating in PoAC consensus
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: PublicKey,
    pub stake: u64,
    pub is_active: bool,
    pub blocks_produced: u64,
}

impl Validator {
    pub fn new(pubkey: PublicKey, stake: u64) -> Self {
        Self {
            pubkey,
            stake,
            is_active: true,
            blocks_produced: 0,
        }
    }
}
