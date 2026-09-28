use crate::evaluator::{EdgeEvaluator, NumericCriteria};
use crate::vault::SovereignContextVault;
use prism_core::{QueryBounty, Transaction, TransactionPayload};
use prism_crypto::{Hash, Keypair, PublicKey};

/// The Sovereign Edge Client running locally on the user's smartphone or PC
pub struct PrismEdgeNode {
    pub keypair: Keypair,
    pub vault: SovereignContextVault,
    pub nonce: u64,
}

impl PrismEdgeNode {
    pub fn new(keypair: Keypair) -> Self {
        Self {
            keypair,
            vault: SovereignContextVault::new(),
            nonce: 0,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.keypair.public_key()
    }

    /// Evaluates an active network bounty against the local vault.
    /// If eligible, constructs and signs a SubmitContextProof transaction.
    pub fn evaluate_and_respond_to_bounty(
        &mut self,
        bounty: &QueryBounty,
        criteria: &NumericCriteria,
        fee: u64,
    ) -> Option<Transaction> {
        let (zk_proof, hardware_attestation) = EdgeEvaluator::prove_criteria(
            &bounty.schema_id,
            criteria,
            &self.vault,
            self.nonce,
        )?;

        let payload = TransactionPayload::SubmitContextProof {
            bounty_id: bounty.bounty_id,
            zk_proof,
            hardware_attestation,
        };

        let tx = self.create_and_sign_tx(payload, fee);
        self.nonce += 1;
        Some(tx)
    }

    /// Create and sign a standard transfer transaction
    pub fn create_transfer(&mut self, to: PublicKey, amount: u64, fee: u64) -> Transaction {
        let payload = TransactionPayload::Transfer { to, amount };
        let tx = self.create_and_sign_tx(payload, fee);
        self.nonce += 1;
        tx
    }

    /// Create and sign an intent execution transaction
    pub fn create_intent_execution(
        &mut self,
        intent_id: Hash,
        solver: PublicKey,
        recipient: PublicKey,
        amount: u64,
        fulfillment_hash: Hash,
        fee: u64,
    ) -> Transaction {
        let payload = TransactionPayload::ExecuteIntent {
            intent_id,
            solver,
            recipient,
            amount,
            fulfillment_hash,
        };
        let tx = self.create_and_sign_tx(payload, fee);
        self.nonce += 1;
        tx
    }

    fn create_and_sign_tx(&self, payload: TransactionPayload, fee: u64) -> Transaction {
        let mut unsigned_bytes = Vec::new();
        unsigned_bytes.extend_from_slice(self.keypair.public_key().as_bytes());
        unsigned_bytes.extend_from_slice(&self.nonce.to_le_bytes());
        unsigned_bytes.extend_from_slice(&fee.to_le_bytes());
        let payload_json = serde_json::to_vec(&payload).expect("Serialization cannot fail");
        unsigned_bytes.extend_from_slice(&payload_json);

        let signature = self.keypair.sign(&unsigned_bytes);

        Transaction {
            sender: self.keypair.public_key(),
            nonce: self.nonce,
            fee,
            payload,
            signature,
        }
    }
}
