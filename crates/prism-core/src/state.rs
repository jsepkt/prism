use crate::account::Account;
use crate::block::Block;
use crate::bounty::{ContextSchema, QueryBounty};
use crate::error::StateError;
use crate::transaction::{Transaction, TransactionPayload};
use prism_crypto::{hash, Hash, PublicKey};
use std::collections::{HashMap, HashSet};

/// In-memory and persistent state of the Prism Network ledger
#[derive(Clone, Debug)]
pub struct BlockchainState {
    pub block_height: u64,
    pub latest_block_hash: Hash,
    pub accounts: HashMap<PublicKey, Account>,
    pub schemas: HashMap<Hash, ContextSchema>,
    pub bounties: HashMap<Hash, QueryBounty>,
    pub processed_txs: HashSet<Hash>,
    /// Anonymous Genesis Builder treasury address collecting protocol royalties
    pub builder_treasury: PublicKey,
    /// Cumulative tokens permanently burned by protocol activity
    pub total_burned: u64,
}

impl BlockchainState {
    pub fn new() -> Self {
        Self {
            block_height: 0,
            latest_block_hash: Hash::ZERO,
            accounts: HashMap::new(),
            schemas: HashMap::new(),
            bounties: HashMap::new(),
            processed_txs: HashSet::new(),
            builder_treasury: PublicKey([7u8; 32]),
            total_burned: 0,
        }
    }

    /// Retrieve an existing account or return an empty account with zero balance
    pub fn get_account(&self, pubkey: &PublicKey) -> Account {
        self.accounts.get(pubkey).cloned().unwrap_or_default()
    }

    /// Get mutable reference to an account, creating it if absent
    pub fn get_account_mut(&mut self, pubkey: &PublicKey) -> &mut Account {
        self.accounts.entry(*pubkey).or_insert_with(Account::default)
    }

    /// Set initial balance for genesis allocations
    pub fn set_balance(&mut self, pubkey: PublicKey, balance: u64) {
        let account = self.get_account_mut(&pubkey);
        account.balance = balance;
    }

    /// Calculate a deterministic state root commitment
    pub fn compute_state_root(&self) -> Hash {
        let mut data = Vec::new();
        data.extend_from_slice(&self.block_height.to_le_bytes());
        data.extend_from_slice(self.latest_block_hash.as_bytes());
        data.extend_from_slice(&(self.accounts.len() as u64).to_le_bytes());
        data.extend_from_slice(&(self.bounties.len() as u64).to_le_bytes());
        hash(&data)
    }

    /// Apply an entire block to the ledger state
    pub fn apply_block(&mut self, block: &Block) -> Result<(), StateError> {
        if block.header.height != self.block_height + 1 && !(self.block_height == 0 && block.header.height == 0) {
            return Err(StateError::InvalidBlock(format!(
                "Invalid block height: expected {}, got {}",
                self.block_height + 1,
                block.header.height
            )));
        }

        if block.header.height > 0 && block.header.prev_hash != self.latest_block_hash {
            return Err(StateError::InvalidBlock("Previous block hash mismatch".to_string()));
        }

        // Verify Merkle tree of transactions
        let computed_tx_root = Block::compute_tx_root(&block.transactions);
        if block.header.tx_root != computed_tx_root {
            return Err(StateError::InvalidBlock("Transaction Merkle root mismatch".to_string()));
        }

        // Apply every transaction in the block
        for tx in &block.transactions {
            self.apply_transaction(tx, &block.header.validator)?;
        }

        self.block_height = block.header.height;
        self.latest_block_hash = block.hash();
        Ok(())
    }

    /// Process a single transaction against current state
    pub fn apply_transaction(&mut self, tx: &Transaction, block_validator: &PublicKey) -> Result<(), StateError> {
        let tx_hash = tx.hash();
        if self.processed_txs.contains(&tx_hash) {
            return Err(StateError::DuplicateTransaction(tx_hash.to_hex()));
        }

        // 1. Cryptographic signature check
        tx.verify_signature()?;

        // 2. Sender account and nonce check
        let sender_acc = self.get_account(&tx.sender);
        if tx.nonce != sender_acc.nonce {
            return Err(StateError::InvalidNonce {
                expected: sender_acc.nonce,
                got: tx.nonce,
            });
        }

        // 3. Ensure sender can pay transaction gas fee
        if sender_acc.balance < tx.fee {
            return Err(StateError::InsufficientBalance {
                required: tx.fee,
                available: sender_acc.balance,
            });
        }

        // Deduct fee and reward block validator
        {
            let sender = self.get_account_mut(&tx.sender);
            sender.balance -= tx.fee;
            sender.nonce += 1;
        }
        {
            let validator = self.get_account_mut(block_validator);
            validator.balance += tx.fee;
        }

        // 4. Process transaction payload
        match &tx.payload {
            TransactionPayload::Transfer { to, amount } => {
                let sender_balance = self.get_account(&tx.sender).balance;
                if sender_balance < *amount {
                    return Err(StateError::InsufficientBalance {
                        required: *amount,
                        available: sender_balance,
                    });
                }
                self.get_account_mut(&tx.sender).balance -= amount;
                self.get_account_mut(to).balance += amount;
            }

            TransactionPayload::RegisterContextSchema {
                schema_id,
                name,
                description,
                min_cohort_size,
            } => {
                if self.schemas.contains_key(schema_id) {
                    return Err(StateError::SchemaAlreadyExists(schema_id.to_hex()));
                }
                // Privacy protection: enforce minimum cohort size
                if *min_cohort_size < 3 {
                    return Err(StateError::CohortTooSmall {
                        min: 3,
                        got: *min_cohort_size,
                    });
                }
                self.schemas.insert(
                    *schema_id,
                    ContextSchema {
                        schema_id: *schema_id,
                        name: name.clone(),
                        description: description.clone(),
                        min_cohort_size: *min_cohort_size,
                        registered_by: tx.sender,
                    },
                );
            }

            TransactionPayload::CreateQueryBounty {
                bounty_id,
                schema_id,
                criteria_commitment,
                reward_per_proof,
                max_participants,
                escrow_amount,
            } => {
                if !self.schemas.contains_key(schema_id) {
                    return Err(StateError::SchemaNotFound(schema_id.to_hex()));
                }
                if self.bounties.contains_key(bounty_id) {
                    return Err(StateError::BountyAlreadyExists(bounty_id.to_hex()));
                }
                let protocol_fee = (*escrow_amount * 1) / 100; // 1% Protocol Fee
                let builder_royalty = protocol_fee / 2;        // 0.5% Builder Royalty
                let burn_amount = protocol_fee - builder_royalty; // 0.5% Burned

                let total_required = *escrow_amount + protocol_fee;
                let sender_balance = self.get_account(&tx.sender).balance;
                if sender_balance < total_required {
                    return Err(StateError::InsufficientBalance {
                        required: total_required,
                        available: sender_balance,
                    });
                }

                // Lock escrow from buyer + deduct protocol fee
                self.get_account_mut(&tx.sender).balance -= total_required;

                // Credit builder royalty to anonymous treasury and record deflationary burn
                let treasury = self.builder_treasury;
                self.get_account_mut(&treasury).balance += builder_royalty;
                self.total_burned += burn_amount;

                let bounty = QueryBounty::new(
                    *bounty_id,
                    tx.sender,
                    *schema_id,
                    *criteria_commitment,
                    *reward_per_proof,
                    *max_participants,
                    *escrow_amount,
                );
                self.bounties.insert(*bounty_id, bounty);
            }

            TransactionPayload::SubmitContextProof {
                bounty_id,
                zk_proof,
                hardware_attestation,
            } => {
                // Verify hardware anti-sybil token
                hardware_attestation.verify(tx.nonce)?;

                let (schema_id, criteria_commitment) = {
                    let bounty = self.bounties.get(bounty_id)
                        .ok_or_else(|| StateError::BountyNotFound(bounty_id.to_hex()))?;

                    if bounty.is_settled {
                        return Err(StateError::BountyExhausted);
                    }

                    // Verify participant hasn't already submitted
                    if bounty.submissions.iter().any(|(pk, _)| pk == &tx.sender) {
                        return Err(StateError::DuplicateProofSubmission);
                    }
                    (bounty.schema_id, bounty.criteria_commitment)
                };

                // Verify ZK Proof matches the schema circuit
                zk_proof.verify(&schema_id, criteria_commitment.as_bytes())?;

                // Register proof submission and check cohort
                let proof_hash = hash(&zk_proof.proof_bytes);
                let min_cohort_size = self.schemas.get(&schema_id)
                    .map(|s| s.min_cohort_size)
                    .ok_or_else(|| StateError::SchemaNotFound(schema_id.to_hex()))?;

                let (participants_to_reward, reward_per_proof) = {
                    let bounty = self.bounties.get_mut(bounty_id)
                        .ok_or_else(|| StateError::BountyNotFound(bounty_id.to_hex()))?;
                    bounty.submissions.push((tx.sender, proof_hash));

                    let was_unlocked = bounty.cohort_unlocked;
                    if bounty.submissions.len() as u32 >= min_cohort_size && !was_unlocked {
                        bounty.cohort_unlocked = true;
                    }

                    let mut to_pay = Vec::new();
                    if bounty.cohort_unlocked {
                        if !was_unlocked {
                            // Cohort just unlocked! Pay all accumulated submissions in this cohort
                            for (pk, _) in &bounty.submissions {
                                if bounty.remaining_escrow >= bounty.reward_per_proof {
                                    bounty.remaining_escrow -= bounty.reward_per_proof;
                                    to_pay.push(*pk);
                                }
                            }
                        } else {
                            // Cohort was already unlocked, pay current submitter
                            if bounty.remaining_escrow >= bounty.reward_per_proof {
                                bounty.remaining_escrow -= bounty.reward_per_proof;
                                to_pay.push(tx.sender);
                            }
                        }

                        if bounty.submissions.len() as u32 >= bounty.max_participants || bounty.remaining_escrow < bounty.reward_per_proof {
                            bounty.is_settled = true;
                        }
                    }
                    (to_pay, bounty.reward_per_proof)
                };

                // Mark account as verified edge node
                let participant = self.get_account_mut(&tx.sender);
                participant.is_hardware_attested = true;
                participant.verified_proofs_count += 1;
                participant.reputation = participant.reputation.saturating_add(10);

                // Distribute dividends to all cohort members
                for recipient_pk in participants_to_reward {
                    let acc = self.get_account_mut(&recipient_pk);
                    acc.balance += reward_per_proof;
                }
            }

            TransactionPayload::ExecuteIntent {
                intent_id: _,
                solver,
                recipient,
                amount,
                fulfillment_hash: _,
            } => {
                let protocol_fee = (*amount * 1) / 100; // 1% Protocol Fee
                let builder_royalty = protocol_fee / 2;        // 0.5% Builder Royalty
                let burn_amount = protocol_fee - builder_royalty; // 0.5% Burned

                let total_required = *amount + protocol_fee;
                let sender_balance = self.get_account(&tx.sender).balance;
                if sender_balance < total_required {
                    return Err(StateError::InsufficientBalance {
                        required: total_required,
                        available: sender_balance,
                    });
                }
                // Release escrowed intent funds directly to fulfillment recipient and reward solver
                self.get_account_mut(&tx.sender).balance -= total_required;
                self.get_account_mut(recipient).balance += amount;

                let treasury = self.builder_treasury;
                self.get_account_mut(&treasury).balance += builder_royalty;
                self.total_burned += burn_amount;

                let solver_acc = self.get_account_mut(solver);
                solver_acc.reputation = solver_acc.reputation.saturating_add(5);
            }
        }

        self.processed_txs.insert(tx_hash);
        Ok(())
    }
}

impl Default for BlockchainState {
    fn default() -> Self {
        Self::new()
    }
}
