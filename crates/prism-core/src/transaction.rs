use crate::error::StateError;
use prism_crypto::{hash, HardwareAttestation, Hash, PublicKey, Signature, ZkProof};
use serde::{Deserialize, Serialize};

/// High-level transaction payloads supported by Prism
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// Standard peer-to-peer balance transfer
    Transfer {
        to: PublicKey,
        amount: u64,
    },
    /// Register a new schema for context queries with privacy floors
    RegisterContextSchema {
        schema_id: Hash,
        name: String,
        description: String,
        min_cohort_size: u32,
    },
    /// Post a funded query bounty with escrow locked in the contract
    CreateQueryBounty {
        bounty_id: Hash,
        schema_id: Hash,
        criteria_commitment: Hash,
        reward_per_proof: u64,
        max_participants: u32,
        escrow_amount: u64,
    },
    /// Submit a verified edge ZK proof with hardware attestation
    SubmitContextProof {
        bounty_id: Hash,
        zk_proof: ZkProof,
        hardware_attestation: HardwareAttestation,
    },
    /// Execute an autonomous intent escrow
    ExecuteIntent {
        intent_id: Hash,
        solver: PublicKey,
        recipient: PublicKey,
        amount: u64,
        fulfillment_hash: Hash,
    },
}

/// A signed transaction ready for broadcast and inclusion
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: PublicKey,
    pub nonce: u64,
    pub fee: u64,
    pub payload: TransactionPayload,
    pub signature: Signature,
}

impl Transaction {
    /// Construct the canonical byte payload to sign or hash
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.sender.as_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.fee.to_le_bytes());
        let payload_json = serde_json::to_vec(&self.payload).expect("Payload serialization cannot fail");
        bytes.extend_from_slice(&payload_json);
        bytes
    }

    /// Compute unique transaction hash (txid)
    pub fn hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }

    /// Verify that the transaction signature matches the sender's public key
    pub fn verify_signature(&self) -> Result<(), StateError> {
        let msg = self.signing_bytes();
        self.sender.verify(&msg, &self.signature).map_err(|_| StateError::InvalidTransactionSignature)
    }
}
