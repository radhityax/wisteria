use libp2p::identify::{Behaviour as Identify, Config as IdentifyConfig};
use libp2p::kad::{Behaviour as Kademlia, store::MemoryStore};
use libp2p::ping::{Behaviour as Ping, Config as PingConfig};
use libp2p::swarm::NetworkBehaviour;
use libp2p::identity::Keypair;

use crate::p2p::bitswap::BitswapBehaviour;

#[derive(NetworkBehaviour)]
pub struct NodeBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub identify: Identify,
    pub ping: Ping,
    pub mdns: libp2p::mdns::tokio::Behaviour,
    pub bitswap: BitswapBehaviour,
}

impl NodeBehaviour {
    pub fn new(keypair: &Keypair) -> std::io::Result<Self> {
        let peer_id = keypair.public().to_peer_id();

        let kademlia = Kademlia::new(peer_id, MemoryStore::new(peer_id));

        let identify = Identify::new(
            IdentifyConfig::new("/ksh/0.1.0".into(), keypair.public()),
        );

        let ping = Ping::new(PingConfig::new());

        let mdns = libp2p::mdns::tokio::Behaviour::new(
            Default::default(), peer_id,
        )?;

        let bitswap = crate::p2p::bitswap::new_bitswap_behaviour();

        Ok(Self { kademlia, identify, ping, mdns, bitswap })
    }
}
