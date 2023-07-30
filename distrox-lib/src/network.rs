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

    pub async fn insert_node(&self, node: Node) -> Result<(), Error> {
        // WHY???
        let ipld = libipld::cbor::DagCborCodec.encode(&node)?;
        let ipld: libipld::Ipld = libipld::cbor::DagCborCodec.decode(&ipld)?;
        self.ipfs.put_dag(ipld).await?;
        Ok(())
    }

    pub async fn insert_post(&self, node: Post) -> Result<(), Error> {
        // WHY???
        let ipld = libipld::cbor::DagCborCodec.encode(&node)?;
        let ipld: libipld::Ipld = libipld::cbor::DagCborCodec.decode(&ipld)?;
        self.ipfs.put_dag(ipld).await?;
        Ok(())
    }

    pub async fn insert_blob(&self, blob: impl Stream<Item = u8> + Send) -> Result<(), Error> {
        use futures::StreamExt;

        self.ipfs
            .add_unixfs(blob.map(|byte| vec![byte]).map(Ok).boxed())
            .await?;
        Ok(())
    }
}

pub struct BootstrapNodes(Vec<Multiaddr>);

impl From<BootstrapNodes> for Vec<Multiaddr> {
    fn from(value: BootstrapNodes) -> Self {
        value.0
    }
}

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
