use std::path::PathBuf;

use futures::Stream;
use libipld::prelude::Codec;
use rust_ipfs::Multiaddr;

use distrox_types::node::Node;
use distrox_types::post::Post;

use crate::error::Error;

pub struct Network {
    ipfs: rust_ipfs::Ipfs,
}

impl Network {
    pub async fn load(
        storage_path: PathBuf,
        bootstrap_nodes: BootstrapNodes,
        listening_addrs: ListeningAddrs,
    ) -> Result<Self, Error> {
        let ipfs = rust_ipfs::UninitializedIpfs::<network_behaviour::Behaviour>::with_opt(
            rust_ipfs::IpfsOptions {
                ipfs_path: rust_ipfs::StoragePath::Disk(storage_path),
                bootstrap: bootstrap_nodes.into(),
                ..Default::default()
            },
        )
        .add_listening_addrs(listening_addrs.into())
        .enable_mdns()
        .enable_relay(true)
        .enable_relay_server(None)
        .enable_upnp()
        .start()
        .await?;

        Ok(Network { ipfs })
    }

    #[cfg(test)]
    async fn inmemory(listening_addrs: ListeningAddrs) -> Result<Self, Error> {
        let ipfs = rust_ipfs::UninitializedIpfs::<network_behaviour::Behaviour>::with_opt(
            rust_ipfs::IpfsOptions {
                ipfs_path: rust_ipfs::StoragePath::Memory,
                ..Default::default()
            },
        )
        .add_listening_addrs(listening_addrs.into())
        .enable_mdns()
        .enable_relay(true)
        .enable_relay_server(None)
        .enable_upnp()
        .start()
        .await?;

        Ok(Network { ipfs })
    }

    pub async fn listening_addresses(&self) -> Result<Vec<Multiaddr>, Error> {
        self.ipfs.listening_addresses().await.map_err(Error::from)
    }

    pub async fn external_addresses(&self) -> Result<Vec<Multiaddr>, Error> {
        self.ipfs.external_addresses().await.map_err(Error::from)
    }

    pub async fn addrs(&self) -> Result<Vec<(libp2p::PeerId, Vec<Multiaddr>)>, Error> {
        self.ipfs.addrs().await.map_err(Error::from)
    }

    pub async fn add_peer(&self, peer_id: libp2p::PeerId, addr: Multiaddr) -> Result<(), Error> {
        self.ipfs.add_peer(peer_id, addr).await.map_err(Error::from)
    }

    pub async fn connect(
        &self,
        peer_id: libp2p::PeerId,
        addrs: Vec<Multiaddr>,
    ) -> Result<(), Error> {
        let opts = libp2p::swarm::dial_opts::DialOpts::peer_id(peer_id)
            .condition(libp2p::swarm::dial_opts::PeerCondition::Disconnected)
            .addresses(addrs)
            .extend_addresses_through_behaviour()
            .build();

        self.ipfs.connect(opts).await.map_err(Error::from)
    }

    pub async fn connect_without_peer(&self, addr: Multiaddr) -> Result<(), Error> {
        let opts = libp2p::swarm::dial_opts::DialOpts::unknown_peer_id()
            .address(addr)
            .build();

        self.ipfs.connect(opts).await.map_err(Error::from)
    }

    pub async fn insert_node(&self, node: Node) -> Result<cid::Cid, Error> {
        // WHY???
        let ipld = libipld::cbor::DagCborCodec.encode(&node)?;
        let ipld: libipld::Ipld = libipld::cbor::DagCborCodec.decode(&ipld)?;
        self.ipfs.put_dag(ipld).await.map_err(Error::from)
    }

    pub async fn insert_post(&self, node: Post) -> Result<cid::Cid, Error> {
        // WHY???
        let ipld = libipld::cbor::DagCborCodec.encode(&node)?;
        let ipld: libipld::Ipld = libipld::cbor::DagCborCodec.decode(&ipld)?;
        self.ipfs.put_dag(ipld).await.map_err(Error::from)
    }

    pub async fn insert_blob(&self, blob: impl Stream<Item = u8> + Send) -> Result<(), Error> {
        use futures::StreamExt;

        self.ipfs
            .add_unixfs(blob.map(|byte| vec![byte]).map(Ok).boxed())
            .await?;
        Ok(())
    }

    async fn fetch_dag(&self, cid: cid::Cid) -> Result<libipld::Ipld, Error> {
        self.ipfs
            .get_dag(rust_ipfs::path::IpfsPath::new(
                rust_ipfs::path::PathRoot::Ipld(cid),
            ))
            .await
            .map_err(Error::from)
    }

    pub async fn get_post(&self, cid: cid::Cid) -> Result<Post, Error> {
        self.fetch_dag(cid).await.and_then(|ipld| {
            let bytes = libipld::cbor::DagCborCodec.encode(&ipld)?;
            libipld::cbor::DagCborCodec
                .decode(&bytes)
                .map_err(Error::from)
        })
    }

    pub async fn get_node(&self, cid: cid::Cid) -> Result<Node, Error> {
        self.fetch_dag(cid).await.and_then(|ipld| {
            let bytes = libipld::cbor::DagCborCodec.encode(&ipld)?;
            libipld::cbor::DagCborCodec
                .decode(&bytes)
                .map_err(Error::from)
        })
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapNodes(Vec<Multiaddr>);

impl From<BootstrapNodes> for Vec<Multiaddr> {
    fn from(value: BootstrapNodes) -> Self {
        value.0
    }
}

#[derive(Debug, Clone)]
pub struct ListeningAddrs(Vec<Multiaddr>);

impl From<ListeningAddrs> for Vec<Multiaddr> {
    fn from(value: ListeningAddrs) -> Self {
        value.0
    }
}

mod network_behaviour {
    use std::task::{Context, Poll};

    use libp2p::{
        core::Endpoint,
        swarm::{
            ConnectionDenied, ConnectionId, FromSwarm, NewListenAddr, PollParameters, THandler,
            THandlerInEvent, THandlerOutEvent, ToSwarm,
        },
        Multiaddr, PeerId,
    };
    use rust_ipfs::NetworkBehaviour;

    #[derive(Default, Debug)]
    pub struct Behaviour;

    impl NetworkBehaviour for Behaviour {
        type ConnectionHandler = rust_ipfs::libp2p::swarm::dummy::ConnectionHandler;
        type OutEvent = void::Void;

        fn handle_pending_inbound_connection(
            &mut self,
            _: ConnectionId,
            _: &Multiaddr,
            _: &Multiaddr,
        ) -> Result<(), ConnectionDenied> {
            Ok(())
        }

        fn handle_pending_outbound_connection(
            &mut self,
            _: ConnectionId,
            _: Option<PeerId>,
            _: &[Multiaddr],
            _: Endpoint,
        ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
            Ok(vec![])
        }

        fn handle_established_inbound_connection(
            &mut self,
            _: ConnectionId,
            _: PeerId,
            _: &Multiaddr,
            _: &Multiaddr,
        ) -> Result<THandler<Self>, ConnectionDenied> {
            Ok(rust_ipfs::libp2p::swarm::dummy::ConnectionHandler)
        }

        fn handle_established_outbound_connection(
            &mut self,
            _: ConnectionId,
            _: PeerId,
            _: &Multiaddr,
            _: Endpoint,
        ) -> Result<THandler<Self>, ConnectionDenied> {
            Ok(rust_ipfs::libp2p::swarm::dummy::ConnectionHandler)
        }

        fn on_connection_handler_event(
            &mut self,
            _: PeerId,
            _: ConnectionId,
            _: THandlerOutEvent<Self>,
        ) {
        }

        fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
            match event {
                FromSwarm::NewListenAddr(NewListenAddr { addr, .. }) => {
                    println!("Listening on {addr}");
                }
                FromSwarm::AddressChange(_)
                | FromSwarm::ConnectionEstablished(_)
                | FromSwarm::ConnectionClosed(_)
                | FromSwarm::DialFailure(_)
                | FromSwarm::ListenFailure(_)
                | FromSwarm::NewListener(_)
                | FromSwarm::ExpiredListenAddr(_)
                | FromSwarm::ListenerError(_)
                | FromSwarm::ListenerClosed(_)
                | FromSwarm::NewExternalAddr(_)
                | FromSwarm::ExpiredExternalAddr(_) => {}
            }
        }

        fn poll(
            &mut self,
            _: &mut Context,
            _: &mut impl PollParameters,
        ) -> Poll<ToSwarm<Self::OutEvent, THandlerInEvent<Self>>> {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tracing::debug;
    use tracing::info;

    #[tokio::test]
    async fn test_single_node() {
        let _ = env_logger::try_init();
        info!("Starting test");
        let listening_addr = ListeningAddrs(vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()]);
        let node1 = Network::inmemory(listening_addr).await.unwrap();
        info!("Node instantiated");

        let node = Node {
            protocol_version: distrox_types::protocol::ProtocolVersion(0),
            parents: Vec::new(),
            post: None,
        };

        let cid = node1.insert_node(node.clone()).await.unwrap();
        info!(?cid, "Put object to node");
        let received_node = node1.get_node(cid).await.unwrap();
        info!(?received_node, "Received object from node");

        assert_eq!(received_node, node);
    }

    #[tokio::test]
    async fn test_connected_nodes() {
        let _ = env_logger::try_init();
        info!("Starting test");
        let listening_addr = ListeningAddrs(vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()]);
        let (node1, node2) = tokio::try_join!(
            Network::inmemory(listening_addr.clone()),
            Network::inmemory(listening_addr)
        )
        .unwrap();
        info!("Nodes instantiated");

        let node1_addrs = node1.listening_addresses().await.unwrap();
        debug!("Node1 listens: {:?}", node1_addrs);

        let node2_addrs = node2.listening_addresses().await.unwrap();
        debug!("Node2 listens: {:?}", node2_addrs);

        for addr in node1_addrs {
            node2.connect_without_peer(addr).await.unwrap();
        }
        info!("Node1 connected to Node2");

        let node = Node {
            protocol_version: distrox_types::protocol::ProtocolVersion(0),
            parents: Vec::new(),
            post: None,
        };

        let cid = node1.insert_node(node.clone()).await.unwrap();
        info!(?cid, "Put object to node1");
        let received_node = node2.get_node(cid).await.unwrap();
        info!(?received_node, "Received object from node2");

        assert_eq!(received_node, node);
    }
}
