use crate::message::P2pMessage;
use crate::peer::{Peer, PeerRegistry};
use prism_core::{Block, Transaction};
use std::net::SocketAddr;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Handshake failed: {0}")]
    Handshake(String),
}

/// Events received from the peer network to be processed by the node
pub enum NetworkEvent {
    BlockReceived(Block),
    TransactionReceived(Transaction),
    PeerConnected(String),
    PeerDisconnected(String),
}

pub struct P2pNode {
    pub node_id: String,
    pub p2p_port: u16,
    pub registry: PeerRegistry,
    event_sender: mpsc::Sender<NetworkEvent>,
}

impl P2pNode {
    pub fn new(node_id: String, p2p_port: u16) -> (Self, mpsc::Receiver<NetworkEvent>) {
        let (tx, rx) = mpsc::channel(1000);
        let registry = PeerRegistry::new();
        let node = Self {
            node_id,
            p2p_port,
            registry,
            event_sender: tx,
        };
        (node, rx)
    }

    /// Broadcast a block to all active peers
    pub async fn broadcast_block(&self, block: Block) {
        self.registry.broadcast(P2pMessage::BroadcastBlock { block }).await;
    }

    /// Broadcast a transaction to all active peers
    pub async fn broadcast_transaction(&self, tx: Transaction) {
        self.registry.broadcast(P2pMessage::BroadcastTransaction { transaction: tx }).await;
    }

    /// Start listening for incoming peer connections
    pub async fn start_listener(&self) -> Result<(), P2pError> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.p2p_port).parse().unwrap();
        let listener = TcpListener::bind(addr).await?;
        info!("Prism P2P Network listening on {}", addr);

        let registry = self.registry.clone();
        let event_tx = self.event_sender.clone();
        let node_id = self.node_id.clone();
        let port = self.p2p_port;

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((socket, peer_addr)) => {
                        let reg = registry.clone();
                        let ev_tx = event_tx.clone();
                        let nid = node_id.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(socket, peer_addr, reg, ev_tx, nid, port).await {
                                warn!("Connection error with {}: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("TCP accept error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect outbound to a bootstrap peer
    pub async fn connect_to_peer(&self, peer_addr: SocketAddr) -> Result<(), P2pError> {
        let socket = TcpStream::connect(peer_addr).await?;
        let registry = self.registry.clone();
        let event_tx = self.event_sender.clone();
        let node_id = self.node_id.clone();
        let port = self.p2p_port;

        tokio::spawn(async move {
            if let Err(e) = Self::handle_connection(socket, peer_addr, registry, event_tx, node_id, port).await {
                warn!("Connection error with {}: {}", peer_addr, e);
            }
        });

        Ok(())
    }

    async fn handle_connection(
        mut socket: TcpStream,
        peer_addr: SocketAddr,
        registry: PeerRegistry,
        event_sender: mpsc::Sender<NetworkEvent>,
        local_id: String,
        local_port: u16,
    ) -> Result<(), P2pError> {
        // 1. Send Handshake
        let handshake = P2pMessage::Handshake {
            node_id: local_id.clone(),
            p2p_port: local_port,
            current_height: 0,
        };
        Self::write_message(&mut socket, &handshake).await?;

        // 2. Read Remote Handshake
        let remote_msg = Self::read_message(&mut socket).await?;
        let (remote_id, remote_height) = match remote_msg {
            P2pMessage::Handshake { node_id, current_height, .. } => (node_id, current_height),
            _ => return Err(P2pError::Handshake("Unexpected initial message".to_string())),
        };

        info!("P2P Handshake successful with peer: {} ({})", remote_id, peer_addr);

        // 3. Register Peer with outbound channel
        let (out_tx, mut out_rx) = mpsc::channel::<P2pMessage>(100);
        let peer = Peer {
            id: remote_id.clone(),
            address: peer_addr,
            height: remote_height,
            tx: out_tx,
        };
        registry.add_peer(peer).await;
        let _ = event_sender.send(NetworkEvent::PeerConnected(remote_id.clone())).await;

        let (mut reader, mut writer) = socket.into_split();

        // Spawn outbound writer task
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = out_rx.recv().await {
                if let Err(e) = Self::write_message_to_writer(&mut writer, &msg).await {
                    warn!("Writer error: {}", e);
                    break;
                }
            }
        });

        // Reader loop
        let read_res = loop {
            match Self::read_message_from_reader(&mut reader).await {
                Ok(msg) => match msg {
                    P2pMessage::BroadcastBlock { block } => {
                        let _ = event_sender.send(NetworkEvent::BlockReceived(block)).await;
                    }
                    P2pMessage::BroadcastTransaction { transaction } => {
                        let _ = event_sender.send(NetworkEvent::TransactionReceived(transaction)).await;
                    }
                    P2pMessage::Ping => {
                        // Handled
                    }
                    _ => {}
                },
                Err(e) => {
                    break Err(e);
                }
            }
        };

        writer_task.abort();
        registry.remove_peer(&remote_id).await;
        let _ = event_sender.send(NetworkEvent::PeerDisconnected(remote_id.clone())).await;
        info!("Peer disconnected: {}", remote_id);

        read_res
    }

    async fn write_message(stream: &mut TcpStream, msg: &P2pMessage) -> Result<(), P2pError> {
        let payload = serde_json::to_vec(msg)?;
        let len = payload.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&payload).await?;
        stream.flush().await?;
        Ok(())
    }

    async fn write_message_to_writer(writer: &mut tokio::net::tcp::OwnedWriteHalf, msg: &P2pMessage) -> Result<(), P2pError> {
        let payload = serde_json::to_vec(msg)?;
        let len = payload.len() as u32;
        writer.write_all(&len.to_be_bytes()).await?;
        writer.write_all(&payload).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn read_message(stream: &mut TcpStream) -> Result<P2pMessage, P2pError> {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;
        let msg = serde_json::from_slice(&buf)?;
        Ok(msg)
    }

    async fn read_message_from_reader(reader: &mut tokio::net::tcp::OwnedReadHalf) -> Result<P2pMessage, P2pError> {
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;
        let msg = serde_json::from_slice(&buf)?;
        Ok(msg)
    }
}
