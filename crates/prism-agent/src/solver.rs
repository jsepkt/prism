use crate::intent::{IntentDomain, UserIntent};
use prism_core::{Transaction, TransactionPayload};
use prism_crypto::{hash, Hash, Keypair, PublicKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolverError {
    #[error("Intent constraints cannot be met")]
    UnsatisfiableConstraints,
    #[error("Cost exceeds maximum budget: cost {cost} > budget {budget}")]
    BudgetExceeded { cost: u64, budget: u64 },
    #[error("Domain unsupported by this solver: {0:?}")]
    UnsupportedDomain(IntentDomain),
}

/// A verifiable bid submitted by an autonomous solver agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SolverBid {
    pub intent_id: Hash,
    pub solver_pubkey: PublicKey,
    pub proposed_cost: u64,
    pub fulfillment_hash: Hash,
    pub service_provider: PublicKey,
}

/// An autonomous agent that monitors user intents, negotiates, and executes transactions
pub struct AutonomousSolver {
    pub keypair: Keypair,
    pub supported_domains: Vec<IntentDomain>,
    pub discount_margin: f64,
}

impl AutonomousSolver {
    pub fn new(keypair: Keypair, domains: Vec<IntentDomain>, discount_margin: f64) -> Self {
        Self {
            keypair,
            supported_domains: domains,
            discount_margin,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.keypair.public_key()
    }

    /// Evaluates whether the solver can fulfill the user's intent under budget
    pub fn evaluate_intent(
        &self,
        intent: &UserIntent,
        service_provider: PublicKey,
    ) -> Result<SolverBid, SolverError> {
        if !self.supported_domains.contains(&intent.domain) {
            return Err(SolverError::UnsupportedDomain(intent.domain.clone()));
        }

        // Simulate optimal algorithmic negotiation: achieves price below budget
        let proposed_cost = (intent.max_budget as f64 * (1.0 - self.discount_margin)) as u64;
        if proposed_cost > intent.max_budget {
            return Err(SolverError::BudgetExceeded {
                cost: proposed_cost,
                budget: intent.max_budget,
            });
        }

        let fulfillment_proof_data = format!("FULFILLED:{}:{}:{}", intent.intent_id, proposed_cost, service_provider);
        let fulfillment_hash = hash(fulfillment_proof_data.as_bytes());

        Ok(SolverBid {
            intent_id: intent.intent_id,
            solver_pubkey: self.keypair.public_key(),
            proposed_cost,
            fulfillment_hash,
            service_provider,
        })
    }

    /// Construct a verified ExecuteIntent transaction to unlock on-chain escrow
    pub fn construct_execution_tx(
        &self,
        intent: &UserIntent,
        bid: &SolverBid,
        nonce: u64,
        fee: u64,
    ) -> Transaction {
        let payload = TransactionPayload::ExecuteIntent {
            intent_id: intent.intent_id,
            solver: bid.solver_pubkey,
            recipient: bid.service_provider,
            amount: bid.proposed_cost,
            fulfillment_hash: bid.fulfillment_hash,
        };

        let mut unsigned = Vec::new();
        unsigned.extend_from_slice(self.keypair.public_key().as_bytes());
        unsigned.extend_from_slice(&nonce.to_le_bytes());
        unsigned.extend_from_slice(&fee.to_le_bytes());
        let payload_json = serde_json::to_vec(&payload).expect("Serialization cannot fail");
        unsigned.extend_from_slice(&payload_json);

        let signature = self.keypair.sign(&unsigned);

        Transaction {
            sender: self.keypair.public_key(),
            nonce,
            fee,
            payload,
            signature,
        }
    }
}
