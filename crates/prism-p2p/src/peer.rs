use crate::message::P2pMessage;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// An active peer connection
#[derive(Clone)]
pub struct Peer {
    pub id: String,
    pub address: SocketAddr,
    pub height: u64,
    pub tx: mpsc::Sender<P2pMessage>,
}

/// Thread-safe peer manager
#[derive(Clone, Default)]
pub struct PeerRegistry {
    peers: Arc<RwLock<HashMap<String, Peer>>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_peer(&self, peer: Peer) {
        let mut map = self.peers.write().await;
        map.insert(peer.id.clone(), peer);
    }

    pub async fn remove_peer(&self, id: &str) {
        let mut map = self.peers.write().await;
        map.remove(id);
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub async fn broadcast(&self, message: P2pMessage) {
        let map = self.peers.read().await;
        for (id, peer) in map.iter() {
            if let Err(e) = peer.tx.send(message.clone()).await {
                tracing::warn!("Failed to send to peer {}: {}", id, e);
            }
        }
    }

    pub async fn update_height(&self, id: &str, height: u64) {
        let mut map = self.peers.write().await;
        if let Some(peer) = map.get_mut(id) {
            peer.height = height;
        }
    }
}
