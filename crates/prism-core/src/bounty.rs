use prism_crypto::{Hash, PublicKey};
use serde::{Deserialize, Serialize};

/// Context Schema definition for data query templates
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextSchema {
    pub schema_id: Hash,
    pub name: String,
    pub description: String,
    /// Absolute privacy floor: minimum distinct participants required
    /// before any aggregated query insights or escrow rewards are unlocked.
    pub min_cohort_size: u32,
    pub registered_by: PublicKey,
}

/// An active Query Bounty posted by an AI lab or enterprise
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryBounty {
    pub bounty_id: Hash,
    pub creator: PublicKey,
    pub schema_id: Hash,
    /// Cryptographic commitment of the specific query criteria
    pub criteria_commitment: Hash,
    pub reward_per_proof: u64,
    pub max_participants: u32,
    pub total_escrow: u64,
    pub remaining_escrow: u64,
    /// List of submitted (participant_pubkey, proof_hash)
    pub submissions: Vec<(PublicKey, Hash)>,
    /// Set to true once min_cohort_size is reached and payouts are distributed
    pub cohort_unlocked: bool,
    pub is_settled: bool,
}

impl QueryBounty {
    pub fn new(
        bounty_id: Hash,
        creator: PublicKey,
        schema_id: Hash,
        criteria_commitment: Hash,
        reward_per_proof: u64,
        max_participants: u32,
        total_escrow: u64,
    ) -> Self {
        Self {
            bounty_id,
            creator,
            schema_id,
            criteria_commitment,
            reward_per_proof,
            max_participants,
            total_escrow,
            remaining_escrow: total_escrow,
            submissions: Vec::new(),
            cohort_unlocked: false,
            is_settled: false,
        }
    }
}
