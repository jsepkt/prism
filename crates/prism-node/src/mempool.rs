use prism_core::Transaction;
use prism_crypto::Hash;
use std::collections::{HashMap, VecDeque};

/// Thread-safe in-memory transaction pool
#[derive(Default)]
pub struct Mempool {
    pending: HashMap<Hash, Transaction>,
    queue: VecDeque<Hash>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            queue: VecDeque::new(),
        }
    }

    pub fn insert(&mut self, tx: Transaction) -> bool {
        let hash = tx.hash();
        if self.pending.contains_key(&hash) {
            return false;
        }
        self.pending.insert(hash, tx);
        self.queue.push_back(hash);
        true
    }

    pub fn drain_batch(&mut self, max_count: usize) -> Vec<Transaction> {
        let mut batch = Vec::new();
        while let Some(hash) = self.queue.pop_front() {
            if let Some(tx) = self.pending.remove(&hash) {
                batch.push(tx);
                if batch.len() >= max_count {
                    break;
                }
            }
        }
        batch
    }

    pub fn size(&self) -> usize {
        self.pending.len()
    }
}
