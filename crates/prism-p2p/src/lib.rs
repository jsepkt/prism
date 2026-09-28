pub mod message;
pub mod network;
pub mod peer;

pub use message::P2pMessage;
pub use network::{NetworkEvent, P2pError, P2pNode};
pub use peer::{Peer, PeerRegistry};

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::Transaction;
    use prism_crypto::{Keypair, PublicKey, Signature};

    #[tokio::test]
    async fn test_p2p_handshake_and_broadcast() {
        let (node1, mut events1) = P2pNode::new("node_alpha".to_string(), 9101);
        let (node2, mut events2) = P2pNode::new("node_beta".to_string(), 9102);

        // Start listener on node 1
        node1.start_listener().await.expect("Node 1 listener should start");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Node 2 connects to Node 1
        let addr = "127.0.0.1:9101".parse().unwrap();
        node2.connect_to_peer(addr).await.expect("Node 2 should connect to Node 1");

        // Wait for connection event
        let ev1 = events1.recv().await.expect("Node 1 should receive event");
        match ev1 {
            NetworkEvent::PeerConnected(id) => assert_eq!(id, "node_beta"),
            _ => panic!("Expected PeerConnected event"),
        }

        let ev2 = events2.recv().await.expect("Node 2 should receive event");
        match ev2 {
            NetworkEvent::PeerConnected(id) => assert_eq!(id, "node_alpha"),
            _ => panic!("Expected PeerConnected event"),
        }

        // Test broadcasting a transaction from Node 2 to Node 1
        let kp = Keypair::generate();
        let dummy_tx = Transaction {
            sender: kp.public_key(),
            nonce: 0,
            fee: 1,
            payload: prism_core::TransactionPayload::Transfer {
                to: PublicKey([1u8; 32]),
                amount: 100,
            },
            signature: Signature([0u8; 64]),
        };

        node2.broadcast_transaction(dummy_tx.clone()).await;

        let ev_tx = events1.recv().await.expect("Node 1 should receive broadcasted tx");
        match ev_tx {
            NetworkEvent::TransactionReceived(tx) => {
                assert_eq!(tx.sender, dummy_tx.sender);
                assert_eq!(tx.fee, 1);
            }
            _ => panic!("Expected TransactionReceived event"),
        }
    }
}
