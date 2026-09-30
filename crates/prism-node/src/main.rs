mod api;
mod mempool;
mod service;

use api::create_router;
use prism_consensus::Validator;
use prism_core::Block;
use prism_crypto::{Keypair, Signature};
use prism_p2p::{NetworkEvent, P2pNode};
use service::NodeService;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("============================================================");
    info!("             PRISM NETWORK NODE v0.1.0                      ");
    info!("     Sovereign Context Ledger & Edge-AI Autonomous Economy  ");
    info!("============================================================");

    // 1. Initialize Data Directory & Persistent Keystore
    let data_dir = std::env::var("PRISM_DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let data_path = std::path::Path::new(&data_dir);
    std::fs::create_dir_all(data_path)?;

    let key_path = data_path.join("validator.key");
    let validator_kp = if let Ok(env_key) = std::env::var("PRISM_VALIDATOR_KEY") {
        let clean = env_key.trim().trim_start_matches("0x");
        let bytes = hex::decode(clean)?;
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes[..32]);
        info!("Loaded validator key from PRISM_VALIDATOR_KEY environment");
        Keypair::from_bytes(&seed)
    } else if key_path.exists() {
        let hex_str = std::fs::read_to_string(&key_path)?;
        let clean = hex_str.trim().trim_start_matches("0x");
        let bytes = hex::decode(clean)?;
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes[..32]);
        info!("Loaded persistent validator key from {}", key_path.display());
        Keypair::from_bytes(&seed)
    } else {
        let kp = Keypair::generate();
        std::fs::write(&key_path, kp.private_key_hex())?;
        info!("Generated new persistent validator key saved to {}", key_path.display());
        kp
    };

    let validator_pubkey = validator_kp.public_key();
    info!("Validator Public Key: {}", validator_pubkey);

    // 2. Initialize Persistent Storage Engine & Core Node Service
    let storage = prism_core::PersistentLedgerStorage::new(&data_dir)?;
    let service = NodeService::new(validator_kp).with_storage(storage.clone());

    // Register Validator in PoAC consensus engine
    {
        let mut consensus = service.consensus.write().await;
        consensus.add_validator(Validator::new(validator_pubkey, 1_000_000));
    }

    // Load existing persistent state or initialize Genesis Block
    if let Some(existing_state) = storage.load_state()? {
        info!("Loaded persistent ledger state at block height #{}", existing_state.block_height);
        let mut state = service.state.write().await;
        *state = existing_state;
        drop(state);

        let existing_blocks = storage.load_all_blocks()?;
        info!("Loaded {} persistent blocks from disk storage", existing_blocks.len());
        let mut blocks = service.blocks.write().await;
        *blocks = existing_blocks;
    } else {
        info!("No existing ledger state found. Initializing genesis state...");
        let mut state = service.state.write().await;
        state.set_balance(validator_pubkey, 10_000_000); // 10M PRISM genesis allocation
        let genesis_block = Block::genesis(validator_pubkey, Signature([0u8; 64]));
        state.latest_block_hash = genesis_block.hash();
        storage.save_state(&state)?;
        storage.append_block(&genesis_block)?;
        let mut blocks = service.blocks.write().await;
        blocks.push(genesis_block.clone());
        info!("Genesis block committed and saved to disk. Hash: {}", genesis_block.hash());
    }

    // 3. Initialize P2P Network Engine
    let p2p_port: u16 = std::env::var("PRISM_P2P_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9000);
    let node_id = format!("prism_node_{}", &validator_pubkey.to_hex()[..8]);
    let (p2p_node, mut p2p_events) = P2pNode::new(node_id, p2p_port);
    let p2p_arc = Arc::new(p2p_node);
    p2p_arc.start_listener().await?;
    service.set_p2p(p2p_arc.clone()).await;

    // Connect to bootstrap peers if specified
    if let Ok(peers_str) = std::env::var("PRISM_BOOTSTRAP_PEERS") {
        for peer in peers_str.split(',') {
            let trimmed = peer.trim();
            if let Ok(addr) = trimmed.parse::<SocketAddr>() {
                info!("Connecting to bootstrap peer: {}", addr);
                let p2p_c = p2p_arc.clone();
                tokio::spawn(async move {
                    if let Err(e) = p2p_c.connect_to_peer(addr).await {
                        tracing::warn!("Failed to connect to peer {}: {}", addr, e);
                    }
                });
            }
        }
    }

    // Process incoming P2P network events
    let svc_p2p = service.clone();
    tokio::spawn(async move {
        while let Some(event) = p2p_events.recv().await {
            match event {
                NetworkEvent::BlockReceived(block) => {
                    info!("Received block #{} from P2P network (hash: {})", block.header.height, block.hash());
                    let _ = svc_p2p.apply_incoming_block(block).await;
                }
                NetworkEvent::TransactionReceived(tx) => {
                    info!("Received transaction {} from P2P gossip", tx.hash());
                    let _ = svc_p2p.submit_transaction(tx).await;
                }
                NetworkEvent::PeerConnected(id) => {
                    info!("New peer connected to mesh: {}", id);
                }
                NetworkEvent::PeerDisconnected(id) => {
                    info!("Peer disconnected from mesh: {}", id);
                }
            }
        }
    });

    // 4. Spawn Background Consensus Block Production Loop
    let svc_clone = service.clone();
    tokio::spawn(async move {
        info!("PoAC Block Producer daemon started (tick rate: 3.0s)");
        let mut interval = tokio::time::interval(Duration::from_millis(3000));
        loop {
            interval.tick().await;
            match svc_clone.produce_and_apply_block().await {
                Ok(Some(block)) => {
                    if !block.transactions.is_empty() {
                        info!(
                            "Produced Block #{} with {} transactions | Hash: {}",
                            block.header.height,
                            block.transactions.len(),
                            block.hash()
                        );
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::error!("Consensus tick error: {}", e);
                }
            }
        }
    });

    // 5. Start HTTP JSON-RPC Server
    let app = create_router(service);
    let port: u16 = std::env::var("PRISM_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8545);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Prism JSON-RPC & REST Server listening on http://127.0.0.1:{}", port);
    info!("Health endpoint available at http://127.0.0.1:{}/health", port);
    info!("Interactive Web Dashboard available at http://127.0.0.1:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
