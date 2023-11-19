use futures::channel::oneshot;
use libp2p::request_response::ResponseChannel;
use libp2p::{core::Multiaddr, multiaddr::Protocol, PeerId};

use crate::event::EventLoop;
use crate::protocol::{
    GetShareRequest, GetShareResponse, RefreshShareRequest, RefreshShareResponse,
    RegisterShareRequest, RegisterShareResponse, Request, Response,
};
use crate::sss::Polynomial;
use std::collections::{hash_map, HashSet};
use std::error::Error;
use tracing::debug;

/// Represents commands that can be issued to the network.
///
/// This enum defines various network operations, such as starting to listen for incoming connections,
/// dialing a peer, starting to provide a key in the DHT, requesting shares, etc.
///
/// # Variants
///
/// * `StartListening` - Command to start listening on a specified address.
/// * `Dial` - Command to dial a specific peer.
/// * `StartProviding` - Command to start providing a key in the Kademlia DHT.
/// * `GetProviders` - Command to get providers for a key in the DHT.
/// * `GetAllProviders` - Command to get all providers in the network.
/// * `RequestShare` - Command to request a share from a peer.
/// * `RespondShare` - Command to respond to a share request.
/// * `RequestRegisterShare` - Command to request registration of a share.
/// * `RespondRegisterShare` - Command to respond to a share registration request.
/// * `RequestRefreshShare` - Command to request the refreshing of shares.
/// * `RespondRefreshShare` - Command to respond to a share refresh request.
///
/// # Examples
///
/// Issuing a `StartListening` command:
///
/// ```ignore
/// let command = Command::StartListening {
///     addr: "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
///     sender: /* oneshot sender */,
/// };
/// ```
#[derive(Debug)]
pub enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    StartProviding {
        key: String,
        sender: oneshot::Sender<()>,
    },
    GetProviders {
        key: String,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    GetAllProviders {
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    RequestShare {
        key: String,
        peer: PeerId,
        sender: PeerId,
        sender_chan: oneshot::Sender<Result<(u8, Vec<u8>), Box<dyn Error + Send>>>,
    },
    RespondShare {
        share: (u8, Vec<u8>),
        success: bool,
        channel: ResponseChannel<Response>,
    },
    RequestRegisterShare {
        share: (u8, Vec<u8>),
        key: String,
        peer: PeerId,
        sender: PeerId,
        sender_chan: oneshot::Sender<Result<bool, Box<dyn Error + Send>>>,
    },
    RespondRegisterShare {
        success: bool,
        channel: ResponseChannel<Response>,
    },
    RequestRefreshShare {
        key: String,
        refresh_key: Vec<Polynomial>,
        peer: PeerId,
        sender: PeerId,
        sender_chan: oneshot::Sender<Result<bool, Box<dyn Error + Send>>>,
    },
    RespondRefreshShare {
        success: bool,
        channel: ResponseChannel<Response>,
    },
}

/// Handles incoming commands for the network event loop.
///
/// This async function processes various network-related commands and performs corresponding actions
/// in the libp2p Swarm. It deals with network operations like listening, dialing, responding to requests, etc.
///
/// # Arguments
///
/// * `eventloop` - The mutable reference to the `EventLoop` managing the network operations.
/// * `command` - The `Command` to be processed.
///
/// # Examples
///
/// Handling a command in the network event loop:
///
/// ```ignore
/// command_handler(&mut eventloop, command).await;
/// ```
pub async fn command_handler(eventloop: &mut EventLoop, command: Command) {
    match command {
        Command::StartListening { addr, sender } => {
            let _ = match eventloop.swarm.listen_on(addr) {
                Ok(_) => sender.send(Ok(())),
                Err(e) => sender.send(Err(Box::new(e))),
            };
        }
        Command::Dial {
            peer_id,
            peer_addr,
            sender,
        } => {
            if let hash_map::Entry::Vacant(e) = eventloop.pending_dial.entry(peer_id) {
                eventloop
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, peer_addr.clone());
                match eventloop.swarm.dial(peer_addr.with(Protocol::P2p(peer_id))) {
                    Ok(()) => {
                        e.insert(sender);
                    }
                    Err(e) => {
                        let _ = sender.send(Err(Box::new(e)));
                    }
                }
            } else {
                todo!("Already dialing peer.");
            }

            // refresh the routing table
            let _ = eventloop.swarm.behaviour_mut().kademlia.bootstrap();
        }
        Command::StartProviding { key, sender } => {
            let query_id = eventloop
                .swarm
                .behaviour_mut()
                .kademlia
                .start_providing(key.into_bytes().into())
                .expect("No store error.");
            eventloop.pending_start_providing.insert(query_id, sender);
        }
        Command::GetProviders { key, sender } => {
            let query_id = eventloop
                .swarm
                .behaviour_mut()
                .kademlia
                .get_providers(key.into_bytes().into());
            eventloop.pending_get_providers.insert(query_id, sender);
        }
        Command::GetAllProviders { sender } => {
            // build a list of all peers in the routing table
            let peers: Vec<PeerId> = eventloop
                .swarm
                .behaviour_mut()
                .gossipsub
                .all_peers()
                .map(|p| p.0.clone())
                .collect();
            debug!("Found {} peers", peers.len());
            debug!("Peers: {:?}", peers);
            let set: HashSet<PeerId> = peers.into_iter().collect();
            sender.send(set).expect("Receiver not to be dropped.");
            debug!("Completed get all providers");
        }
        Command::RequestShare {
            key,
            peer,
            sender,
            sender_chan,
        } => {
            let request_id = eventloop
                .swarm
                .behaviour_mut()
                .request_response
                .send_request(
                    &peer,
                    Request::GetShare(GetShareRequest {
                        key,
                        peer: peer.into(),
                        sender: sender.into(),
                    }),
                );
            eventloop
                .pending_request_share
                .insert(request_id, sender_chan);
        }
        Command::RespondShare {
            share,
            success,
            channel,
        } => {
            eventloop
                .swarm
                .behaviour_mut()
                .request_response
                .send_response(
                    channel,
                    Response::GetShare(GetShareResponse { share, success }),
                )
                .expect("Connection to peer to be still open.");
        }
        Command::RequestRegisterShare {
            share,
            key,
            peer,
            sender,
            sender_chan,
        } => {
            debug!("Sending request to register share {}.", key);
            let request_id = eventloop
                .swarm
                .behaviour_mut()
                .request_response
                .send_request(
                    &peer,
                    Request::RegisterShare(RegisterShareRequest {
                        share,
                        key,
                        peer: peer.into(),
                        sender: sender.into(),
                    }),
                );
            eventloop
                .pending_register_share
                .insert(request_id, sender_chan);
            debug!("Sent request to register share");
        }
        Command::RespondRegisterShare { success, channel } => {
            eventloop
                .swarm
                .behaviour_mut()
                .request_response
                .send_response(
                    channel,
                    Response::RegisterShare(RegisterShareResponse { success }),
                )
                .expect("Connection to peer should still be open.");
        }
        Command::RequestRefreshShare {
            key,
            refresh_key,
            peer,
            sender,
            sender_chan,
        } => {
            debug!("Sending request to refresh shares {}.", key);
            let request_id = eventloop
                .swarm
                .behaviour_mut()
                .request_response
                .send_request(
                    &peer,
                    Request::RefreshShare(RefreshShareRequest {
                        key,
                        refresh_key,
                        peer: peer.into(),
                        sender: sender.into(),
                    }),
                );
            eventloop
                .pending_refresh_share
                .insert(request_id, sender_chan);
            debug!("Sent request to refresh shares");
        }
        Command::RespondRefreshShare { success, channel } => {
            eventloop
                .swarm
                .behaviour_mut()
                .request_response
                .send_response(
                    channel,
                    Response::RefreshShares(RefreshShareResponse { success }),
                )
                .expect("Connection to peer to be still open.");
        }
    }
}
