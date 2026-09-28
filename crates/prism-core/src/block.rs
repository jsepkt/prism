use crate::transaction::Transaction;
use prism_crypto::{hash, hash_concat, Hash, PublicKey, Signature};
use serde::{Deserialize, Serialize};

/// Header metadata for a Prism block
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub prev_hash: Hash,
    pub timestamp: i64,
    pub tx_root: Hash,
    pub state_root: Hash,
    pub validator: PublicKey,
    pub validator_signature: Signature,
}

impl BlockHeader {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(self.prev_hash.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(self.tx_root.as_bytes());
        bytes.extend_from_slice(self.state_root.as_bytes());
        bytes.extend_from_slice(self.validator.as_bytes());
        bytes
    }

    pub fn hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// A complete block containing header and verified transactions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    /// Calculate the Merkle/hash root of transactions in the block
    pub fn compute_tx_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return Hash::ZERO;
        }
        let mut hashes: Vec<Hash> = transactions.iter().map(|tx| tx.hash()).collect();
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(hash_concat(&[chunk[0].as_bytes(), chunk[1].as_bytes()]));
                } else {
                    next_level.push(hash_concat(&[chunk[0].as_bytes(), chunk[0].as_bytes()]));
                }
            }
            hashes = next_level;
        }
        hashes[0]
    }

    /// Construct the official genesis block
    pub fn genesis(validator: PublicKey, validator_signature: Signature) -> Self {
        let header = BlockHeader {
            height: 0,
            prev_hash: Hash::ZERO,
            timestamp: 0,
            tx_root: Hash::ZERO,
            state_root: Hash::ZERO,
            validator,
            validator_signature,
        };
        Self {
            header,
            transactions: Vec::new(),
        }
    }
}
