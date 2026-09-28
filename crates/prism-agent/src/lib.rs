pub mod intent;
pub mod solver;

pub use intent::{IntentDomain, UserIntent};
pub use solver::{AutonomousSolver, SolverBid, SolverError};

#[cfg(test)]
mod tests {
    use super::*;
    use prism_crypto::Keypair;

    #[test]
    fn test_intent_lifecycle_and_solver_bidding() {
        let user_kp = Keypair::generate();
        let airline_provider_kp = Keypair::generate();
        let solver_kp = Keypair::generate();

        // 1. User declares an intent: "Book flight to Denver under $400 with extra legroom"
        let intent = UserIntent::new(
            user_kp.public_key(),
            IntentDomain::TravelFlight,
            "Direct flight SFO -> DEN under 400 USD with extra legroom",
            400,
            400,
        );

        assert_eq!(intent.max_budget, 400);

        // 2. Autonomous Solver evaluates the intent with a 15% negotiated discount
        let solver = AutonomousSolver::new(solver_kp, vec![IntentDomain::TravelFlight], 0.15);
        let bid_res = solver.evaluate_intent(&intent, airline_provider_kp.public_key());
        assert!(bid_res.is_ok());

        let bid = bid_res.unwrap();
        // 400 * (1 - 0.15) = 340
        assert_eq!(bid.proposed_cost, 340);
        assert_eq!(bid.service_provider, airline_provider_kp.public_key());

        // 3. Construct on-chain execution transaction
        let tx = solver.construct_execution_tx(&intent, &bid, 0, 2);
        assert!(tx.verify_signature().is_ok());

        match tx.payload {
            prism_core::TransactionPayload::ExecuteIntent { amount, recipient, .. } => {
                assert_eq!(amount, 340);
                assert_eq!(recipient, airline_provider_kp.public_key());
            }
            _ => panic!("Expected ExecuteIntent transaction"),
        }
    }
}
