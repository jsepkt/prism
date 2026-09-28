use serde::{Deserialize, Serialize};

/// Account state in the Prism Ledger
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
    pub reputation: u32,
    pub verified_proofs_count: u64,
    pub is_hardware_attested: bool,
}

impl Account {
    pub fn new(balance: u64) -> Self {
        Self {
            balance,
            nonce: 0,
            reputation: 100,
            verified_proofs_count: 0,
            is_hardware_attested: false,
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new(0)
    }
}
