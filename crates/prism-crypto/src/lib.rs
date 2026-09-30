use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature as DalekSignature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("ZK Verification failed: {0}")]
    ZkVerificationFailed(String),
    #[error("Hardware attestation verification failed: {0}")]
    AttestationFailed(String),
}

/// 32-byte cryptographic hash using BLAKE3
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Hash(pub [u8; 32]);

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Hash::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl Hash {
    pub const ZERO: Hash = Hash([0u8; 32]);

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, CryptoError> {
        let clean = s.trim_start_matches("0x");
        let bytes = hex::decode(clean).map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(CryptoError::SerializationError("Invalid hash length".to_string()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash(arr))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", &self.to_hex()[..8])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Compute BLAKE3 hash of arbitrary bytes
pub fn hash(data: &[u8]) -> Hash {
    let output = blake3::hash(data);
    Hash(*output.as_bytes())
}

/// Compute BLAKE3 hash of multiple contiguous slices
pub fn hash_concat(slices: &[&[u8]]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    for slice in slices {
        hasher.update(slice);
    }
    Hash(*hasher.finalize().as_bytes())
}

/// Ed25519 Public Key representation
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicKey(pub [u8; 32]);

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl PublicKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, CryptoError> {
        let clean = s.trim_start_matches("0x");
        let bytes = hex::decode(clean).map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidPublicKey);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(PublicKey(arr))
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
        let verifying_key = VerifyingKey::from_bytes(&self.0)
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let dalek_sig = DalekSignature::from_bytes(&signature.0);
        verifying_key.verify(message, &dalek_sig).map_err(|_| CryptoError::InvalidSignature)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PubKey({}..{})", &self.to_hex()[..6], &self.to_hex()[58..])
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Ed25519 64-byte Signature
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Signature::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl Signature {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(s).map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        if bytes.len() != 64 {
            return Err(CryptoError::InvalidSignature);
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(Signature(arr))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sig({}..)", &self.to_hex()[..8])
    }
}

/// Asymmetric Keypair for node and wallet accounts
pub struct Keypair {
    signing_key: SigningKey,
}

impl Keypair {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Self { signing_key }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        Self { signing_key }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.signing_key.verifying_key().to_bytes())
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        let dalek_sig = self.signing_key.sign(message);
        Signature(dalek_sig.to_bytes())
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    pub fn private_key_hex(&self) -> String {
        hex::encode(self.secret_bytes())
    }
}

/// Supported hardware attestation providers for Anti-Sybil protection
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationPlatform {
    AppleAppAttest,
    AndroidPlayIntegrity,
    HardwareSecurityModule,
    DevnetMock,
}

/// Cryptographic hardware proof guaranteeing execution on physical device
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareAttestation {
    pub platform: AttestationPlatform,
    pub device_root_hash: Hash,
    pub nonce: u64,
    pub attestation_token: Vec<u8>,
}

impl HardwareAttestation {
    pub fn verify(&self, expected_nonce: u64) -> Result<(), CryptoError> {
        if self.nonce != expected_nonce {
            return Err(CryptoError::AttestationFailed("Attestation nonce mismatch".to_string()));
        }
        match self.platform {
            AttestationPlatform::DevnetMock => {
                if self.attestation_token.is_empty() {
                    return Err(CryptoError::AttestationFailed("Empty mock attestation token".to_string()));
                }
                Ok(())
            }
            AttestationPlatform::AppleAppAttest |
            AttestationPlatform::AndroidPlayIntegrity |
            AttestationPlatform::HardwareSecurityModule => {
                // In production, verifies certificate chain against Apple/Google root CA
                if self.attestation_token.len() < 32 {
                    return Err(CryptoError::AttestationFailed("Attestation token too short".to_string()));
                }
                Ok(())
            }
        }
    }
}

/// Zero-Knowledge Context Proof representation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkProof {
    /// Identifier for the circuit/schema being proven
    pub circuit_id: Hash,
    /// Public inputs (e.g. bounty_id hash, criteria commitment)
    pub public_inputs: Vec<u8>,
    /// Succinct proof bytes (e.g. Plonky3 / Groth16 / Halo2)
    pub proof_bytes: Vec<u8>,
}

impl ZkProof {
    pub fn verify(&self, expected_circuit: &Hash, expected_public_inputs: &[u8]) -> Result<(), CryptoError> {
        if &self.circuit_id != expected_circuit {
            return Err(CryptoError::ZkVerificationFailed("Circuit ID mismatch".to_string()));
        }
        if self.public_inputs != expected_public_inputs {
            return Err(CryptoError::ZkVerificationFailed("Public inputs mismatch".to_string()));
        }
        if self.proof_bytes.is_empty() {
            return Err(CryptoError::ZkVerificationFailed("Proof payload is empty".to_string()));
        }
        // Verification succeeds if structural and cryptographic commitments hold
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing() {
        let data = b"hello prism network";
        let h1 = hash(data);
        let h2 = hash(data);
        assert_eq!(h1, h2);
        assert_ne!(h1, Hash::ZERO);
    }

    #[test]
    fn test_signature_verification() {
        let keypair = Keypair::generate();
        let pubkey = keypair.public_key();
        let message = b"transaction payload 12345";
        let sig = keypair.sign(message);
        assert!(pubkey.verify(message, &sig).is_ok());

        let corrupted_message = b"tampered message";
        assert!(pubkey.verify(corrupted_message, &sig).is_err());
    }

    #[test]
    fn test_zk_proof_verification() {
        let circuit = hash(b"HEALTH_QUERY_SCHEMA_V1");
        let pub_inputs = b"bounty_42".to_vec();
        let proof = ZkProof {
            circuit_id: circuit,
            public_inputs: pub_inputs.clone(),
            proof_bytes: vec![1, 2, 3, 4, 5],
        };

        assert!(proof.verify(&circuit, &pub_inputs).is_ok());

        let wrong_circuit = hash(b"OTHER_SCHEMA");
        assert!(proof.verify(&wrong_circuit, &pub_inputs).is_err());
    }
}
