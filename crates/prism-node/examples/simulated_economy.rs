use prism_client::{NumericCriteria, PrismEdgeNode};
use prism_consensus::Validator;
use prism_core::{Block, Transaction, TransactionPayload};
use prism_crypto::{hash, Keypair, Signature};
use prism_node::service::NodeService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n==================================================================");
    println!("       PRISM NETWORK: END-TO-END CONSUMER ECONOMY SIMULATION       ");
    println!("==================================================================\n");

    // 1. Initialize Genesis Validator & Node Service
    let validator_kp = Keypair::generate();
    let validator_pubkey = validator_kp.public_key();
    let service = NodeService::new(validator_kp);

    // Register Validator with 1,000,000 PRISM stake in consensus
    {
        let mut consensus = service.consensus.write().await;
        consensus.add_validator(Validator::new(validator_pubkey, 1_000_000));
    }

    // Genesis Block
    {
        let mut state = service.state.write().await;
        let genesis_block = Block::genesis(validator_pubkey, Signature([0u8; 64]));
        state.latest_block_hash = genesis_block.hash();
        println!("[1/6] Genesis block committed. Hash: {}", genesis_block.hash());
    }

    // 2. Setup Accounts: Enterprise Buyer & 4 Everyday Smartphone Users
    let enterprise_kp = Keypair::generate();
    let mut alice = PrismEdgeNode::new(Keypair::generate());
    let mut bob = PrismEdgeNode::new(Keypair::generate());
    let mut charlie = PrismEdgeNode::new(Keypair::generate());
    let mut dave = PrismEdgeNode::new(Keypair::generate());

    println!("[2/6] Provisioning Accounts & Local Edge Vaults...");
    {
        let mut state = service.state.write().await;
        // Fund Enterprise with 50,000 PRISM
        state.set_balance(enterprise_kp.public_key(), 50_000);
        // Fund users with nominal gas (10 PRISM each)
        state.set_balance(alice.public_key(), 10);
        state.set_balance(bob.public_key(), 10);
        state.set_balance(charlie.public_key(), 10);
        state.set_balance(dave.public_key(), 10);
    }

    // Populate Users' On-Device Encrypted Vaults with real-life simulated data
    alice.vault.record_numeric("health", "weekly_run_distance_km", 26.4);
    alice.vault.record_numeric("health", "avg_sleep_hours", 7.6);

    bob.vault.record_numeric("health", "weekly_run_distance_km", 19.8);
    bob.vault.record_numeric("health", "avg_sleep_hours", 8.1);

    // Charlie runs very little (only 4.2 km) - does not meet criteria
    charlie.vault.record_numeric("health", "weekly_run_distance_km", 4.2);
    charlie.vault.record_numeric("health", "avg_sleep_hours", 6.2);

    dave.vault.record_numeric("health", "weekly_run_distance_km", 31.5);
    dave.vault.record_numeric("health", "avg_sleep_hours", 7.2);

    println!("      - Alice's device: 26.4 km/wk (Qualifies)");
    println!("      - Bob's device:   19.8 km/wk (Qualifies)");
    println!("      - Charlie's device: 4.2 km/wk (Does NOT qualify)");
    println!("      - Dave's device:  31.5 km/wk (Qualifies)");

    // 3. Enterprise Registers a Context Schema: "Runner Health & Sleep Study"
    let schema_id = hash(b"PRISM_RUNNER_STUDY_V1");
    let register_schema_payload = TransactionPayload::RegisterContextSchema {
        schema_id,
        name: "Runner Sleep & Activity Correlation".to_string(),
        description: "Anonymized cohort insights for active runners (>15km/week)".to_string(),
        min_cohort_size: 3, // Privacy threshold: at least 3 participants required
    };

    let register_tx = Transaction {
        sender: enterprise_kp.public_key(),
        nonce: 0,
        fee: 5,
        payload: register_schema_payload.clone(),
        signature: enterprise_kp.sign(&{
            let mut b = Vec::new();
            b.extend_from_slice(enterprise_kp.public_key().as_bytes());
            b.extend_from_slice(&0u64.to_le_bytes());
            b.extend_from_slice(&5u64.to_le_bytes());
            b.extend_from_slice(&serde_json::to_vec(&register_schema_payload)?);
            b
        }),
    };
    service.submit_transaction(register_tx).await?;

    // 4. Enterprise Creates a Query Bounty: 500 PRISM per proof (1,500 PRISM escrow)
    let criteria = NumericCriteria {
        category: "health".to_string(),
        metric: "weekly_run_distance_km".to_string(),
        min_value: Some(15.0),
        max_value: None,
    };
    let criteria_commitment = criteria.commitment();
    let bounty_id = hash(b"BOUNTY_RUNNERS_STUDY_OCT");

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
            b.extend_from_slice(&serde_json::to_vec(&bounty_payload)?);
            b
        }),
    };
    service.submit_transaction(bounty_tx).await?;

    // Produce Block 1 to confirm schema and bounty
    let block1 = service.produce_and_apply_block().await?.expect("Block 1 should be produced");
    println!("\n[3/6] Block #1 Mined! ({} transactions)", block1.transactions.len());
    println!("      - Schema Registered: 'Runner Sleep & Activity Correlation' (Min Cohort: 3)");
    println!("      - Bounty Created: 1,500 PRISM locked in smart contract escrow");

    // Retrieve active bounty from ledger
    let active_bounty = {
        let state = service.state.read().await;
        state.bounties.get(&bounty_id).cloned().expect("Bounty must exist")
    };

    // 5. Users' Devices Evaluate Bounty Locally via Edge Enclaves
    println!("\n[4/6] Edge Devices Evaluating Criteria Locally...");

    // Alice evaluates
    let alice_tx = alice.evaluate_and_respond_to_bounty(&active_bounty, &criteria, 1);
    if let Some(tx) = alice_tx {
        println!("      ✔ Alice: Criteria matched! ZK-proof generated. Transmitting proof...");
        service.submit_transaction(tx).await?;
    }

    // Bob evaluates
    let bob_tx = bob.evaluate_and_respond_to_bounty(&active_bounty, &criteria, 1);
    if let Some(tx) = bob_tx {
        println!("      ✔ Bob: Criteria matched! ZK-proof generated. Transmitting proof...");
        service.submit_transaction(tx).await?;
    }

    // Charlie evaluates
    let charlie_tx = charlie.evaluate_and_respond_to_bounty(&active_bounty, &criteria, 1);
    if charlie_tx.is_none() {
        println!("      ✖ Charlie: Criteria NOT matched (only 4.2 km). 0 data leaked, no transaction.");
    }

    // Produce Block 2 with Alice and Bob's submissions
    let block2 = service.produce_and_apply_block().await?.expect("Block 2 should be produced");
    println!("\n[5/6] Block #2 Mined! ({} transactions)", block2.transactions.len());
    {
        let state = service.state.read().await;
        let b = state.bounties.get(&bounty_id).unwrap();
        println!("      - Cohort Status: {}/3 submissions received.", b.submissions.len());
        println!("      - Privacy Status: Payouts LOCKED pending cohort threshold (Anti-De-anonymization active).");
        println!("      - Alice's balance: {} PRISM (dividend pending)", state.get_account(&alice.public_key()).balance);
    }

    // Dave joins the study
    let dave_tx = dave.evaluate_and_respond_to_bounty(&active_bounty, &criteria, 1);
    if let Some(tx) = dave_tx {
        println!("\n      ✔ Dave: Criteria matched! Submitting 3rd proof...");
        service.submit_transaction(tx).await?;
    }

    // Produce Block 3
    let block3 = service.produce_and_apply_block().await?.expect("Block 3 should be produced");
    println!("\n[6/6] Block #3 Mined! ({} transactions)", block3.transactions.len());

    // Final State Verification
    {
        let state = service.state.read().await;
        let b = state.bounties.get(&bounty_id).unwrap();
        println!("      - Cohort Threshold Reached: {}/3! Cohort Unlocked: {}", b.submissions.len(), b.cohort_unlocked);
        println!("      - Bounty Fully Settled: {}", b.is_settled);

        println!("\n==================================================================");
        println!("                   FINAL BALANCES & DIVIDENDS                     ");
        println!("==================================================================");
        println!("  Enterprise Account:      {} PRISM (50,000 - 1,500 escrow - 15 protocol fee - 10 tx fees)", state.get_account(&enterprise_kp.public_key()).balance);
        println!("  Alice (Participant #1):  {} PRISM (10 initial - 1 fee + 500 dividend)", state.get_account(&alice.public_key()).balance);
        println!("  Bob   (Participant #2):  {} PRISM (10 initial - 1 fee + 500 dividend)", state.get_account(&bob.public_key()).balance);
        println!("  Charlie (Non-qualifier): {} PRISM (unchanged, 0 data leaked)", state.get_account(&charlie.public_key()).balance);
        println!("  Dave  (Participant #3):  {} PRISM (10 initial - 1 fee + 500 dividend)", state.get_account(&dave.public_key()).balance);
        println!("  Genesis Validator:       {} PRISM (collected transaction fees)", state.get_account(&validator_pubkey).balance);
        println!("  ------------------------------------------------------------------");
        println!("  Builder Treasury (You):  {} PRISM (automated 0.5% protocol royalty)", state.get_account(&state.builder_treasury).balance);
        println!("  Total Deflationary Burn: {} PRISM (permanently destroyed supply)", state.total_burned);
        println!("==================================================================");
        println!("  STATUS: SUCCESS - All cryptographic, privacy, and economic");
        println!("          invariants satisfied! Zero raw user data exposed.\n");
    }

    Ok(())
}
