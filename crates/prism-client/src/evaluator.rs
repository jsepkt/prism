use crate::vault::SovereignContextVault;
use prism_crypto::{hash, AttestationPlatform, HardwareAttestation, Hash, ZkProof};
use serde::{Deserialize, Serialize};

/// Query predicate evaluated locally inside the device's secure enclave
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NumericCriteria {
    pub category: String,
    pub metric: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl NumericCriteria {
    pub fn commitment(&self) -> Hash {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        hash(&serialized)
    }

    pub fn evaluate(&self, vault: &SovereignContextVault) -> bool {
        if let Some(val) = vault.get_numeric(&self.category, &self.metric) {
            if let Some(min) = self.min_value {
                if val < min {
                    return false;
                }
            }
            if let Some(max) = self.max_value {
                if val > max {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

/// Evaluator that produces cryptographic validity proofs on the edge device
pub struct EdgeEvaluator;

impl EdgeEvaluator {
    /// Evaluates a criteria against the local vault. If matched, generates a ZK proof and hardware attestation.
    pub fn prove_criteria(
        schema_id: &Hash,
        criteria: &NumericCriteria,
        vault: &SovereignContextVault,
        account_nonce: u64,
    ) -> Option<(ZkProof, HardwareAttestation)> {
        if !criteria.evaluate(vault) {
            return None;
        }

        let criteria_commitment = criteria.commitment();

        // In production, this runs inside the device NPU / WebAssembly micro-prover
        let zk_proof = ZkProof {
            circuit_id: *schema_id,
            public_inputs: criteria_commitment.as_bytes().to_vec(),
            proof_bytes: hash(&criteria_commitment.0).0.to_vec(),
        };

        // Hardware attestation proving execution on genuine device hardware
        let attestation = HardwareAttestation {
            platform: AttestationPlatform::DevnetMock,
            device_root_hash: Hash::ZERO,
            nonce: account_nonce,
            attestation_token: vec![42u8; 32],
        };

        Some((zk_proof, attestation))
    }
}
