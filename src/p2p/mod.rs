use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{Swarm, PeerId, SwarmBuilder, noise, yamux};
use anyhow::Result;

mod behaviour;
use behaviour::NodeBehaviour;

pub struct P2pNode {
    pub swarm: Swarm<NodeBehaviour>,
    pub peer_id: PeerId,
}

impl P2pNode {
    pub fn new() -> Result<Self> {
        let swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                Default::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_dns()?
            .with_behaviour(|key| {
                NodeBehaviour::new(key)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })?
            .with_swarm_config(|cfg| cfg)
            .build();

        let peer_id = *swarm.local_peer_id();
        Ok(Self { swarm, peer_id })
    }

    pub fn listen_on(&mut self, addr: &str) -> Result<()> {
        let addr: libp2p::Multiaddr = addr.parse()?;
        self.swarm.listen_on(addr)?;
        Ok(())
    }

    pub fn dial(&mut self, addr: &str) -> Result<()> {
        let addr: libp2p::Multiaddr = addr.parse()?;
        self.swarm.dial(addr)?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {}", address);
                }
                SwarmEvent::Behaviour(event) => {
                    println!("Behaviour event: {:?}", event);
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("Connected to {}", peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    println!("Disconnected from {}", peer_id);
                }
                SwarmEvent::IncomingConnectionError { error, .. } => {
                    log::warn!("Incoming connection error: {}", error);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    log::warn!("Outgoing connection error to {:?}: {}", peer_id,
                        error);
                }
                _ => {}
            }
        }
    }
}
