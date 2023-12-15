use futures::channel::{mpsc, oneshot};
use futures::prelude::*;

use libp2p::identify;
use libp2p::multiaddr::Protocol;
use libp2p::{
    kad,
    request_response::{self, OutboundRequestId, ResponseChannel},
    swarm::{Swarm, SwarmEvent},
    PeerId,
};

use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::debug;

use crate::command::command_handler;
use crate::command::Command;
use crate::network::{Behaviour, BehaviourEvent};
use crate::protocol::Request;
use crate::protocol::Response;

/// Represents various events that can occur in the network.
///
/// This enum defines different types of network events that the application may need to handle.
///
/// # Variants
///
/// * `InboundRequest` - Represents an inbound request event with the request data and a response channel.
///
/// # Examples
///
/// Handling an `InboundRequest` event:
///
/// ```ignore
/// match event {
///     Event::InboundRequest { request, channel } => {
///         // Handle the request and possibly send a response back using the channel.
///     },
/// }
/// ```
#[derive(Debug)]
pub enum Event {
    InboundRequest {
        request: Request,
        channel: ResponseChannel<Response>,
    },
}

/// Manages the event loop for network operations.
///
/// This struct encapsulates the logic to handle events from the libp2p Swarm, process incoming commands,
/// and maintain various aspects of network state.
///
/// # Fields
///
/// * `swarm` - The libp2p Swarm instance handling network behaviours.
/// * `command_receiver` - Receiver for incoming commands.
/// * `event_sender` - Sender for outgoing network events.
/// * `pending_dial` - Tracks pending dial operations.
/// * `pending_start_providing` - Tracks pending operations to start providing a record in the Kademlia DHT.
/// * `pending_get_providers` - Tracks pending operations to get providers for a record in the Kademlia DHT.
/// * `pending_request_share` - Tracks pending share request operations.
/// * `pending_register_share` - Tracks pending operations to register a share.
/// * `pending_refresh_share` - Tracks pending operations to refresh a share.
///
/// # Examples
///
/// Creating and running an `EventLoop`:
///
/// ```ignore
/// let event_loop = EventLoop::new(swarm, command_receiver, event_sender);
/// event_loop.run().await;
/// ```
pub struct EventLoop {
    pub swarm: Swarm<Behaviour>,
    pub command_receiver: mpsc::Receiver<Command>,
    pub event_sender: mpsc::Sender<Event>,
    pub pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pub pending_start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    pub pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    pub pending_request_share:
        HashMap<OutboundRequestId, oneshot::Sender<Result<(u8, Vec<u8>), Box<dyn Error + Send>>>>,
    pub pending_register_share:
        HashMap<OutboundRequestId, oneshot::Sender<Result<bool, Box<dyn Error + Send>>>>,
    pub pending_refresh_share:
        HashMap<OutboundRequestId, oneshot::Sender<Result<bool, Box<dyn Error + Send>>>>,
}

impl EventLoop {
    /// Constructs a new `EventLoop`.
    ///
    /// # Arguments
    ///
    /// * `swarm` - The libp2p Swarm to be used for networking.
    /// * `command_receiver` - A channel receiver for incoming commands.
    /// * `event_sender` - A channel sender for outgoing events.
    ///
    /// # Returns
    ///
    /// An instance of `EventLoop`.
    pub fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
            pending_request_share: Default::default(),
            pending_register_share: Default::default(),
            pending_refresh_share: Default::default(),
        }
    }

    /// Runs the event loop.
    ///
    /// This method continuously listens for events from the Swarm and incoming commands,
    /// and handles them appropriately.
    ///
    /// # Examples
    ///
    /// Running the event loop:
    ///
    /// ```ignore
    /// event_loop.run().await;
    /// ```
    pub async fn run(mut self) {
        loop {
            futures::select! {
                event = self.swarm.next() => self.handle_event(event.expect("Swarm stream to be infinite.")).await  ,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    // Command channel closed, thus shutting down the network event loop.
                    None=>  return,
                },
            }
        }
    }

    /// Handles a single event from the Swarm.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle.
    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result: kad::QueryResult::StartProviding(_),
                    ..
                },
            )) => {
                let sender: oneshot::Sender<()> = self
                    .pending_start_providing
                    .remove(&id)
                    .expect("Completed query to be previously pending.");
                let _ = sender.send(());
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result:
                        kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                            providers,
                            ..
                        })),
                    ..
                },
            )) => {
                if let Some(sender) = self.pending_get_providers.remove(&id) {
                    sender.send(providers).expect("Receiver not to be dropped");

                    // Finish the query. We are only interested in the first result.
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .query_mut(&id)
                        .unwrap()
                        .finish();
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(e)) => {
                debug!("Identify message: {e:?}");
                if let identify::Event::Received {
                    peer_id,
                    info:
                        identify::Info {
                            listen_addrs,
                            protocols,
                            observed_addr,
                            ..
                        },
                } = e
                {
                    debug!("identify::Event::Received observed_addr: {}", observed_addr);

                    self.swarm.add_external_address(observed_addr);
                    
                     // TODO: The following should no longer be necessary after https://github.com/libp2p/rust-libp2p/pull/4371.
                    if protocols.iter().any(|p| *p == kad::PROTOCOL_NAME) {
                        for addr in listen_addrs {
                            self.swarm
                                .behaviour_mut()
                                .kademlia
                                .add_address(&peer_id, addr.clone());
                        }
                    }
                }
                let _ = self.swarm.behaviour_mut().kademlia.bootstrap();
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    result:
                        kad::QueryResult::GetProviders(Ok(
                            kad::GetProvidersOk::FinishedWithNoAdditionalRecord { .. },
                        )),
                    ..
                },
            )) => {}
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::RoutingUpdated {
                peer,
                addresses,
                ..
            })) => {
                let address = addresses.first().to_owned();
                //this is likely useless since these peers are already in the routing table
                debug!("adding peer {peer} with address {address}");
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer, address);
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .add_explicit_peer(&peer);

                let _ = self.swarm.behaviour_mut().kademlia.bootstrap();
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(_)) => {}
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    debug!("Received request: {request:?} from {channel:?}");
                    self.event_sender
                        .send(Event::InboundRequest {
                            request: request,
                            channel,
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => match response {
                    Response::GetShare(res) => {
                        debug!("Received response for share {}.", request_id);
                        let _ = self
                            .pending_request_share
                            .remove(&request_id)
                            .expect("Request to still be pending.")
                            .send(Ok(res.share));
                    }
                    Response::RegisterShare(res) => {
                        debug!("Received response to register share {}.", res.success);
                        let _ = self
                            .pending_register_share
                            .remove(&request_id)
                            .expect("Request to still be pending.")
                            .send(Ok(res.success));
                    }
                    Response::RefreshShares(res) => {
                        debug!("Received response to refresh shares {}.", res.success);
                        let _ = self
                            .pending_refresh_share
                            .remove(&request_id)
                            .expect("Request to still be pending.")
                            .send(Ok(res.success));
                    }
                },
            },

            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                libp2p::request_response::Event::OutboundFailure {
                    peer,
                    request_id,
                    error,
                    ..
                },
            )) => {
                debug!("Request to {peer} failed with error: {error}");
                let _ = self.pending_register_share.remove(&request_id);
                let _ = self.pending_request_share.remove(&request_id);
                let _ = self.pending_refresh_share.remove(&request_id);
            }

            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::InboundFailure {
                    peer,
                    request_id,
                    error,
                    ..
                },
            )) => {
                debug!("InboundFailure Request failed with error: {error}, peer: {peer}, request_id: {request_id}");
            }

            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::ResponseSent { .. },
            )) => {}

            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                debug!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }

                let _ = self.swarm.behaviour_mut().kademlia.bootstrap();
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing {
                peer_id: Some(_peer_id),
                ..
            } => {}
            e => debug!("unhandled provider event: {e:?}"),
        }
    }

    /// Handles a single command from the command channel.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to handle.
    async fn handle_command(&mut self, command: Command) {
        command_handler(self, command).await;
    }
}
