pub mod client;
pub mod evaluator;
pub mod vault;

pub use client::PrismEdgeNode;
pub use evaluator::{EdgeEvaluator, NumericCriteria};
pub use vault::{ContextRecord, SovereignContextVault};

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::QueryBounty;
    use prism_crypto::{hash, Keypair};

    #[test]
    fn test_edge_node_evaluation_and_proof_generation() {
        let keypair = Keypair::generate();
        let mut edge_node = PrismEdgeNode::new(keypair);

        // Record health metrics locally
        edge_node.vault.record_numeric("health", "weekly_run_distance_km", 22.5);
        edge_node.vault.record_numeric("health", "avg_sleep_hours", 7.8);

        let schema_id = hash(b"HEALTH_QUERY_SCHEMA");
        let criteria = NumericCriteria {
            category: "health".to_string(),
            metric: "weekly_run_distance_km".to_string(),
            min_value: Some(15.0),
            max_value: None,
        };

        let bounty = QueryBounty::new(
            hash(b"BOUNTY_1"),
            Keypair::generate().public_key(),
            schema_id,
            criteria.commitment(),
            100,
            10,
            1000,
        );

        let tx_opt = edge_node.evaluate_and_respond_to_bounty(&bounty, &criteria, 2);
        assert!(tx_opt.is_some());

        let tx = tx_opt.unwrap();
        assert!(tx.verify_signature().is_ok());
        assert_eq!(tx.nonce, 0);
        assert_eq!(edge_node.nonce, 1);
    }

    #[test]
    fn test_edge_node_rejection_when_criteria_unmet() {
        let keypair = Keypair::generate();
        let mut edge_node = PrismEdgeNode::new(keypair);

        // User only ran 5 km
        edge_node.vault.record_numeric("health", "weekly_run_distance_km", 5.0);

        let criteria = NumericCriteria {
            category: "health".to_string(),
            metric: "weekly_run_distance_km".to_string(),
            min_value: Some(15.0),
            max_value: None,
        };

        let bounty = QueryBounty::new(
            hash(b"BOUNTY_1"),
            Keypair::generate().public_key(),
            hash(b"SCHEMA"),
            criteria.commitment(),
            100,
            10,
            1000,
        );

        let tx_opt = edge_node.evaluate_and_respond_to_bounty(&bounty, &criteria, 2);
        assert!(tx_opt.is_none());
    }
}
