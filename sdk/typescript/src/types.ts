export interface Account {
  balance: number;
  nonce: number;
  reputation: number;
  verified_proofs_count: number;
  is_hardware_attested: boolean;
}

export interface ContextSchema {
  schema_id: string;
  name: string;
  description: string;
  min_cohort_size: number;
  registered_by: string;
}

export interface QueryBounty {
  bounty_id: string;
  creator: string;
  schema_id: string;
  criteria_commitment: string;
  reward_per_proof: number;
  max_participants: number;
  total_escrow: number;
  remaining_escrow: number;
  submissions: [string, string][];
  cohort_unlocked: boolean;
  is_settled: boolean;
}

export interface NumericCriteria {
  category: string;
  metric: string;
  min_value?: number;
  max_value?: number;
}

export interface ZkProof {
  circuit_id: string;
  public_inputs: number[];
  proof_bytes: number[];
}

export interface HardwareAttestation {
  platform: 'AppleAppAttest' | 'AndroidPlayIntegrity' | 'HardwareSecurityModule' | 'DevnetMock';
  device_root_hash: string;
  nonce: number;
  attestation_token: number[];
}

export type TransactionPayload =
  | { Transfer: { to: string; amount: number } }
  | { RegisterContextSchema: { schema_id: string; name: string; description: string; min_cohort_size: number } }
  | { CreateQueryBounty: { bounty_id: string; schema_id: string; criteria_commitment: string; reward_per_proof: number; max_participants: number; escrow_amount: number } }
  | { SubmitContextProof: { bounty_id: string; zk_proof: ZkProof; hardware_attestation: HardwareAttestation } }
  | { ExecuteIntent: { intent_id: string; solver: string; recipient: string; amount: number; fulfillment_hash: string } };

export interface Transaction {
  sender: string;
  nonce: number;
  fee: number;
  payload: TransactionPayload;
  signature: string;
}

export interface NodeHealth {
  status: string;
  block_height: number;
  latest_block_hash: string;
  validator: string;
  pending_mempool_txs: number;
}
