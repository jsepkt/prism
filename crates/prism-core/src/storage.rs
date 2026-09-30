use crate::block::Block;
use crate::state::BlockchainState;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

/// Production-grade persistent storage manager for Prism Network ledger.
/// Handles atomic state snapshots, write-ahead logs (WAL), and historical block logs.
#[derive(Clone, Debug)]
pub struct PersistentLedgerStorage {
    data_dir: PathBuf,
}

impl PersistentLedgerStorage {
    /// Initialize storage rooted in the provided directory.
    /// Creates directory trees if absent.
    pub fn new(data_dir: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let dir = data_dir.into();
        fs::create_dir_all(&dir)?;
        fs::create_dir_all(dir.join("blocks"))?;
        Ok(Self { data_dir: dir })
    }

    /// Path to the primary atomic state snapshot file
    pub fn state_file_path(&self) -> PathBuf {
        self.data_dir.join("state.json")
    }

    /// Path to the block history folder
    pub fn blocks_dir_path(&self) -> PathBuf {
        self.data_dir.join("blocks")
    }

    /// Atomically persist current blockchain state snapshot to disk.
    /// Uses a temporary file + atomic rename to prevent corruption on unexpected crashes.
    pub fn save_state(&self, state: &BlockchainState) -> Result<(), std::io::Error> {
        let tmp_path = self.data_dir.join("state.json.tmp");
        let final_path = self.state_file_path();

        let json_bytes = serde_json::to_vec_pretty(state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        {
            let mut file = File::create(&tmp_path)?;
            file.write_all(&json_bytes)?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, final_path)?;
        Ok(())
    }

    /// Load the most recent blockchain state snapshot from disk, if present.
    pub fn load_state(&self) -> Result<Option<BlockchainState>, std::io::Error> {
        let path = self.state_file_path();
        if !path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let state: BlockchainState = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Some(state))
    }

    /// Persist a finalized block into the historical block storage.
    pub fn append_block(&self, block: &Block) -> Result<(), std::io::Error> {
        let filename = format!("{:010}.json", block.header.height);
        let block_path = self.blocks_dir_path().join(filename);

        let json_bytes = serde_json::to_vec_pretty(block)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = File::create(block_path)?;
        file.write_all(&json_bytes)?;
        file.sync_all()?;

        // Also append line to the append-only block log (WAL)
        let wal_path = self.data_dir.join("blocks.jsonl");
        let compact_json = serde_json::to_string(block)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut wal_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(wal_path)?;
        writeln!(wal_file, "{}", compact_json)?;
        wal_file.sync_all()?;

        Ok(())
    }

    /// Load all historical blocks in ascending height order.
    pub fn load_all_blocks(&self) -> Result<Vec<Block>, std::io::Error> {
        let blocks_dir = self.blocks_dir_path();
        let mut entries = Vec::new();

        if blocks_dir.exists() {
            for entry in fs::read_dir(blocks_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    entries.push(path);
                }
            }
        }

        entries.sort();

        let mut blocks = Vec::new();
        for path in entries {
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let block: Block = serde_json::from_str(&contents)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            blocks.push(block);
        }

        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_crypto::{Keypair, Signature};

    #[test]
    fn test_persistent_storage_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("prism_test_storage_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let storage = PersistentLedgerStorage::new(&temp_dir).unwrap();

        // 1. Initial state should be None
        assert!(storage.load_state().unwrap().is_none());

        // 2. Create state and save
        let mut state = BlockchainState::new();
        let kp = Keypair::generate();
        state.set_balance(kp.public_key(), 5_000);
        state.block_height = 42;

        storage.save_state(&state).unwrap();

        // 3. Load state and verify integrity
        let loaded = storage.load_state().unwrap().expect("State should exist");
        assert_eq!(loaded.block_height, 42);
        assert_eq!(loaded.get_account(&kp.public_key()).balance, 5_000);

        // 4. Save and load blocks
        let genesis_block = Block::genesis(kp.public_key(), Signature([1u8; 64]));
        storage.append_block(&genesis_block).unwrap();

        let blocks = storage.load_all_blocks().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].header.height, 0);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
