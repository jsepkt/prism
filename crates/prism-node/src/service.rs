use crate::mempool::Mempool;
use prism_consensus::PoacConsensusEngine;
use prism_core::{Block, BlockchainState, PersistentLedgerStorage, Transaction};
use prism_crypto::{Hash, Keypair};
use prism_p2p::P2pNode;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core service orchestrating consensus, state, and transaction processing
#[derive(Clone)]
pub struct NodeService {
    pub state: Arc<RwLock<BlockchainState>>,
    pub mempool: Arc<RwLock<Mempool>>,
    pub consensus: Arc<RwLock<PoacConsensusEngine>>,
    pub validator_keypair: Arc<Keypair>,
    pub blocks: Arc<RwLock<Vec<Block>>>,
    pub p2p: Arc<RwLock<Option<Arc<P2pNode>>>>,
    pub storage: Option<Arc<PersistentLedgerStorage>>,
}

impl NodeService {
    pub fn new(validator_keypair: Keypair) -> Self {
        Self {
            state: Arc::new(RwLock::new(BlockchainState::new())),
            mempool: Arc::new(RwLock::new(Mempool::new())),
            consensus: Arc::new(RwLock::new(PoacConsensusEngine::new())),
            validator_keypair: Arc::new(validator_keypair),
            blocks: Arc::new(RwLock::new(Vec::new())),
            p2p: Arc::new(RwLock::new(None)),
            storage: None,
        }
    }

    pub fn with_storage(mut self, storage: PersistentLedgerStorage) -> Self {
        self.storage = Some(Arc::new(storage));
        self
    }

    pub async fn set_p2p(&self, p2p: Arc<P2pNode>) {
        let mut guard = self.p2p.write().await;
        *guard = Some(p2p);
    }

    /// Submit a transaction from RPC to mempool with validation and gossip to peers
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<Hash, String> {
        tx.verify_signature().map_err(|e| format!("Invalid signature: {}", e))?;

        let tx_hash = tx.hash();
        let mut mempool = self.mempool.write().await;
        if mempool.insert(tx.clone()) {
            drop(mempool);
            let p2p_guard = self.p2p.read().await;
            if let Some(p2p) = p2p_guard.as_ref() {
                p2p.broadcast_transaction(tx).await;
            }
            Ok(tx_hash)
        } else {
            Err("Transaction already in mempool".to_string())
        }
    }

    /// Apply an incoming block received from a P2P peer
    pub async fn apply_incoming_block(&self, block: Block) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.apply_block(&block).map_err(|e| format!("Failed to apply peer block: {}", e))?;
        let mut blocks = self.blocks.write().await;
        blocks.push(block.clone());

        if let Some(storage) = &self.storage {
            let _ = storage.save_state(&state);
            let _ = storage.append_block(&block);
        }

        Ok(())
    }

    /// Attempt to produce and commit a block with pending transactions
    pub async fn produce_and_apply_block(&self) -> Result<Option<Block>, String> {
        let mut mempool = self.mempool.write().await;
        let batch = mempool.drain_batch(100);

        let state_guard = self.state.read().await;
        let consensus_guard = self.consensus.read().await;

        let block = consensus_guard
            .produce_block(&self.validator_keypair, &state_guard, batch)
            .map_err(|e| format!("Block production error: {}", e))?;

        drop(state_guard);
        drop(consensus_guard);

        // Apply block to state
        let mut state_mut = self.state.write().await;
        state_mut.apply_block(&block).map_err(|e| format!("Failed to apply block: {}", e))?;

        // Archive block
        let mut blocks_guard = self.blocks.write().await;
        blocks_guard.push(block.clone());
        drop(blocks_guard);

        // Persist to disk
        if let Some(storage) = &self.storage {
            let _ = storage.save_state(&state_mut);
            let _ = storage.append_block(&block);
        }

        // Broadcast to P2P network
        let p2p_guard = self.p2p.read().await;
        if let Some(p2p) = p2p_guard.as_ref() {
            p2p.broadcast_block(block.clone()).await;
        }

        Ok(Some(block))
    }
}
