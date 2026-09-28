use prism_crypto::{hash, Hash, PublicKey};
use serde::{Deserialize, Serialize};

/// High-level practical life domains for intent commerce
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentDomain {
    TravelFlight,
    HotelAccommodation,
    EnergyUtilityNegotiation,
    SubscriptionOptimization,
    GroceryDirectDelivery,
}

/// A human intent order declared by the user and locked in escrow
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserIntent {
    pub intent_id: Hash,
    pub user: PublicKey,
    pub domain: IntentDomain,
    pub description: String,
    pub max_budget: u64,
    pub escrow_locked: u64,
    pub created_at: i64,
}

impl UserIntent {
    pub fn new(
        user: PublicKey,
        domain: IntentDomain,
        description: &str,
        max_budget: u64,
        escrow_locked: u64,
    ) -> Self {
        let mut data = Vec::new();
        data.extend_from_slice(user.as_bytes());
        data.extend_from_slice(description.as_bytes());
        data.extend_from_slice(&max_budget.to_le_bytes());
        data.extend_from_slice(&escrow_locked.to_le_bytes());
        let intent_id = hash(&data);

        Self {
            intent_id,
            user,
            domain,
            description: description.to_string(),
            max_budget,
            escrow_locked,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}
