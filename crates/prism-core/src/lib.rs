pub mod account;
pub mod block;
pub mod bounty;
pub mod error;
pub mod state;
pub mod transaction;

pub use account::Account;
pub use block::{Block, BlockHeader};
pub use bounty::{ContextSchema, QueryBounty};
pub use error::StateError;
pub use state::BlockchainState;
pub use transaction::{Transaction, TransactionPayload};

#[cfg(test)]
mod tests {
    use super::*;
    use prism_crypto::{
        hash, AttestationPlatform, HardwareAttestation, Hash, Keypair, ZkProof,
    };

    #[test]
    fn test_genesis_and_transfer() {
        let mut state = BlockchainState::new();
        let alice_kp = Keypair::generate();
        let bob_kp = Keypair::generate();
        let validator_kp = Keypair::generate();

        state.set_balance(alice_kp.public_key(), 1_000);

        let tx = Transaction {
            sender: alice_kp.public_key(),
            nonce: 0,
            fee: 10,
            payload: TransactionPayload::Transfer {
                to: bob_kp.public_key(),
                amount: 300,
            },
            signature: alice_kp.sign(&{
                let mut b = Vec::new();
                b.extend_from_slice(alice_kp.public_key().as_bytes());
                b.extend_from_slice(&0u64.to_le_bytes());
                b.extend_from_slice(&10u64.to_le_bytes());
                b.extend_from_slice(&serde_json::to_vec(&TransactionPayload::Transfer {
                    to: bob_kp.public_key(),
                    amount: 300,
                }).unwrap());
                b
            }),
        };

        let result = state.apply_transaction(&tx, &validator_kp.public_key());
        assert!(result.is_ok());

        assert_eq!(state.get_account(&alice_kp.public_key()).balance, 690); // 1000 - 10 fee - 300
        assert_eq!(state.get_account(&bob_kp.public_key()).balance, 300);
        assert_eq!(state.get_account(&validator_kp.public_key()).balance, 10);
    }

    #[test]
    fn test_privacy_preserving_cohort_bounty_lifecycle() {
        let mut state = BlockchainState::new();
        let enterprise_kp = Keypair::generate();
        let validator_kp = Keypair::generate();

        state.set_balance(enterprise_kp.public_key(), 100_000);

        // 1. Register Context Schema with min_cohort_size = 3
        let schema_id = hash(b"FITNESS_SLEEP_CORRELATION_V1");
        let reg_payload = TransactionPayload::RegisterContextSchema {
            schema_id,
            name: "Fitness and Sleep Correlation".to_string(),
            description: "Anonymized sleep duration for runners".to_string(),
            min_cohort_size: 3,
        };

        let reg_tx = Transaction {
            sender: enterprise_kp.public_key(),
            nonce: 0,
            fee: 5,
            payload: reg_payload.clone(),
            signature: enterprise_kp.sign(&{
                let mut b = Vec::new();
                b.extend_from_slice(enterprise_kp.public_key().as_bytes());
                b.extend_from_slice(&0u64.to_le_bytes());
                b.extend_from_slice(&5u64.to_le_bytes());
                b.extend_from_slice(&serde_json::to_vec(&reg_payload).unwrap());
                b
            }),
        };
        assert!(state.apply_transaction(&reg_tx, &validator_kp.public_key()).is_ok());

        // 2. Enterprise posts a Query Bounty with 1,500 token escrow (500 per proof, max 3)
        let bounty_id = hash(b"BOUNTY_OCTOBER_RUNNERS");
        let criteria_commitment = hash(b"runs_weekly > 15 && sleep_hrs > 7");
        let bounty_payload = TransactionPayload::CreateQueryBounty {
            bounty_id,
            schema_id,
            criteria_commitment,
            reward_per_proof: 500,
            max_participants: 3,
            escrow_amount: 1500,
        };

        let bounty_tx = Transaction {
            sender: enterprise_kp.public_key(),
            nonce: 1,
            fee: 5,
            payload: bounty_payload.clone(),
            signature: enterprise_kp.sign(&{
                let mut b = Vec::new();
                b.extend_from_slice(enterprise_kp.public_key().as_bytes());
                b.extend_from_slice(&1u64.to_le_bytes());
                b.extend_from_slice(&5u64.to_le_bytes());
                b.extend_from_slice(&serde_json::to_vec(&bounty_payload).unwrap());
                b
            }),
        };
        assert!(state.apply_transaction(&bounty_tx, &validator_kp.public_key()).is_ok());

        // 3. User 1 submits verified context proof
        let user1_kp = Keypair::generate();
        state.set_balance(user1_kp.public_key(), 10); // gas fee

        let zk_proof = ZkProof {
            circuit_id: schema_id,
            public_inputs: criteria_commitment.as_bytes().to_vec(),
            proof_bytes: vec![1, 2, 3, 4],
        };
        let attestation = HardwareAttestation {
            platform: AttestationPlatform::DevnetMock,
            device_root_hash: Hash::ZERO,
            nonce: 0,
            attestation_token: vec![99; 32],
        };

        let submit_payload1 = TransactionPayload::SubmitContextProof {
            bounty_id,
            zk_proof: zk_proof.clone(),
            hardware_attestation: attestation.clone(),
        };

        let submit_tx1 = Transaction {
            sender: user1_kp.public_key(),
            nonce: 0,
            fee: 2,
            payload: submit_payload1.clone(),
            signature: user1_kp.sign(&{
                let mut b = Vec::new();
                b.extend_from_slice(user1_kp.public_key().as_bytes());
                b.extend_from_slice(&0u64.to_le_bytes());
                b.extend_from_slice(&2u64.to_le_bytes());
                b.extend_from_slice(&serde_json::to_vec(&submit_payload1).unwrap());
                b
            }),
        };
        assert!(state.apply_transaction(&submit_tx1, &validator_kp.public_key()).is_ok());

        // At this point, cohort size is 1 < 3. Payout must NOT be distributed yet for privacy!
        assert_eq!(state.get_account(&user1_kp.public_key()).balance, 8); // 10 - 2 fee

        // 4. User 2 and User 3 submit
        let user2_kp = Keypair::generate();
        state.set_balance(user2_kp.public_key(), 10);
        let submit_payload2 = TransactionPayload::SubmitContextProof {
            bounty_id,
            zk_proof: zk_proof.clone(),
            hardware_attestation: attestation.clone(),
        };
        let submit_tx2 = Transaction {
            sender: user2_kp.public_key(),
            nonce: 0,
            fee: 2,
            payload: submit_payload2.clone(),
            signature: user2_kp.sign(&{
                let mut b = Vec::new();
                b.extend_from_slice(user2_kp.public_key().as_bytes());
                b.extend_from_slice(&0u64.to_le_bytes());
                b.extend_from_slice(&2u64.to_le_bytes());
                b.extend_from_slice(&serde_json::to_vec(&submit_payload2).unwrap());
                b
            }),
        };
        assert!(state.apply_transaction(&submit_tx2, &validator_kp.public_key()).is_ok());

        let user3_kp = Keypair::generate();
        state.set_balance(user3_kp.public_key(), 10);
        let submit_payload3 = TransactionPayload::SubmitContextProof {
            bounty_id,
            zk_proof,
            hardware_attestation: attestation,
        };
        let submit_tx3 = Transaction {
            sender: user3_kp.public_key(),
            nonce: 0,
            fee: 2,
            payload: submit_payload3.clone(),
            signature: user3_kp.sign(&{
                let mut b = Vec::new();
                b.extend_from_slice(user3_kp.public_key().as_bytes());
                b.extend_from_slice(&0u64.to_le_bytes());
                b.extend_from_slice(&2u64.to_le_bytes());
                b.extend_from_slice(&serde_json::to_vec(&submit_payload3).unwrap());
                b
            }),
        };
        assert!(state.apply_transaction(&submit_tx3, &validator_kp.public_key()).is_ok());

        // Now cohort threshold (3) is satisfied: User 3 receives payout immediately!
        let user3_acc = state.get_account(&user3_kp.public_key());
        assert_eq!(user3_acc.balance, 10 - 2 + 500); // initial 10 - 2 fee + 500 dividend
        assert!(user3_acc.is_hardware_attested);
        assert_eq!(user3_acc.verified_proofs_count, 1);

        // Verify protocol fee was split: 50% to builder treasury, 50% burned
        let treasury_balance = state.get_account(&state.builder_treasury).balance;
        assert_eq!(treasury_balance, 7); // 0.5% of 1500
        assert_eq!(state.total_burned, 8); // 0.5% burned
    }
}
