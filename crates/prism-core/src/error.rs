use prism_crypto::CryptoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },

    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("Invalid transaction signature")]
    InvalidTransactionSignature,

    #[error("Duplicate transaction: {0}")]
    DuplicateTransaction(String),

    #[error("Schema already registered: {0}")]
    SchemaAlreadyExists(String),

    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Cohort size too small for privacy: min required {min}, got {got}")]
    CohortTooSmall { min: u32, got: u32 },

    #[error("Bounty already exists: {0}")]
    BountyAlreadyExists(String),

    #[error("Bounty not found: {0}")]
    BountyNotFound(String),

    #[error("Bounty escrow exhausted or expired")]
    BountyExhausted,

    #[error("Participant already submitted proof for bounty")]
    DuplicateProofSubmission,

    #[error("Cohort threshold not yet met for payout (privacy protection active)")]
    CohortThresholdNotMet,

    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
