use libp2p::Multiaddr;
use libp2p_identity::PeerId;

/// An event gets send from the backend to the frontend
#[derive(Debug)]
pub enum Event {
    ConnectionEstablished { address: Multiaddr },
    ConnectionClosed { address: Multiaddr },

    PubSubSubscribe(PeerId),
    PubSubUnsubscribe(PeerId),
    // PubSubMessage
}
