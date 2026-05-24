use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::kad::{self, QueryId, RecordKey, GetProvidersOk};
use libp2p::request_response::{self as req_res, OutboundRequestId};
use libp2p::{Swarm, PeerId, SwarmBuilder, noise, yamux};
use anyhow::Result;

use cid::Cid;
use std::sync::Arc;
use std::collections::HashSet;

pub mod bitswap;
mod behaviour;
use behaviour::{NodeBehaviour, NodeBehaviourEvent};
use bitswap::{BitswapRequest, BitswapResponse};
use crate::block::Block;
use crate::store::BlockStore;

pub struct P2pNode {
    pub swarm: Swarm<NodeBehaviour>,
    pub peer_id: PeerId,
    store: Arc<dyn BlockStore>,
    pending_requests: HashSet<OutboundRequestId>,
}

impl P2pNode {
    pub fn new(store: Arc<dyn BlockStore>) -> Result<Self> {
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
        Ok(Self {
            swarm,
            peer_id,
            store,
            pending_requests: HashSet::new(),
        })
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

    pub fn request_block(&mut self, peer: PeerId, cid: &Cid) -> OutboundRequestId {
        let req = BitswapRequest(cid.to_bytes());
        let id = self.swarm.behaviour_mut().bitswap.send_request(&peer, req);
        self.pending_requests.insert(id);
        id
    }

    pub fn start_providing(&mut self, cid: &Cid) -> Result<QueryId> {
        let key = RecordKey::new(&cid.to_bytes());
        self.swarm.behaviour_mut().kademlia.start_providing(key)
            .map_err(Into::into)
    }

    pub fn find_providers(&mut self, cid: &Cid) -> QueryId {
        let key = RecordKey::new(&cid.to_bytes());
        self.swarm.behaviour_mut().kademlia.get_providers(key)
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {}", address);
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
                    log::warn!("Outgoing connection error to {:?}: {}", peer_id, error);
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Bitswap(event)) => {
                    self.handle_bitswap_event(event).await?;
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Kademlia(event)) => {
                    self.handle_kademlia_event(event)?;
                }
                SwarmEvent::Behaviour(_) => {}
                _ => {}
            }
        }
    }

    async fn handle_bitswap_event(
        &mut self,
        event: req_res::Event<BitswapRequest, BitswapResponse>,
    ) -> Result<()> {
        match event {
            req_res::Event::Message {
                peer,
                message:
                    req_res::Message::Request {
                        request_id: _,
                        request,
                        channel,
                    },
                ..
            } => {
                let cid = match Cid::read_bytes(request.0.as_slice()) {
                    Ok(cid) => cid,
                    Err(_) => return Ok(()),
                };
                println!("Bitswap request from {} for {}", peer, cid);

                if let Some(block) = self.store.get(&cid).await? {
                    let res = BitswapResponse(block.data().to_vec());
                    if let Err(e) = self.swarm.behaviour_mut().bitswap.send_response(channel, res)
                    {
                        log::warn!("Failed to send response: {:?}", e);
                    }
                } else {
                    if let Err(e) = self.swarm.behaviour_mut().bitswap.send_response(
                        channel,
                        BitswapResponse(vec![]),
                    ) {
                        log::warn!("Failed to send response: {:?}", e);
                    }
                }
            }
            req_res::Event::Message {
                peer,
                message:
                    req_res::Message::Response {
                        request_id,
                        response,
                    },
                ..
            } => {
                self.pending_requests.remove(&request_id);
                if response.0.is_empty() {
                    println!("Peer {} doesn't have the block", peer);
                } else {
                    let block = Block::new(response.0)?;
                    self.store.put(&block).await?;
                    println!("Received block {} from {}", block.cid(), peer);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_kademlia_event(
        &mut self,
        event: kad::Event,
    ) -> Result<()> {
        if let kad::Event::OutboundQueryProgressed { result, .. } = event {
            if let kad::QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                key: _,
                providers,
            })) = result
            {
                println!("Found {} providers for {:?}", providers.len(), providers);
            } else if let kad::QueryResult::GetProviders(Err(e)) = result {
                log::warn!("FindProviders failed: {:?}", e);
            }
        }
        Ok(())
    }
}
