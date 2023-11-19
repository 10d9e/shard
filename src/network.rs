use crate::client::Client;
use crate::event::{Event, EventLoop};
use crate::protocol::{Request, Response};

use futures::channel::mpsc;
use futures::prelude::*;

use libp2p::gossipsub::IdentTopic;
use libp2p::request_response::ProtocolSupport;
use libp2p::{
    gossipsub, identify, identity, kad, noise, request_response, swarm::NetworkBehaviour, tcp,
    yamux, StreamProtocol,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::Duration;
use std::vec;
use tracing::debug;

/// Represents the combined network behaviour for the libp2p Swarm.
///
/// This struct encapsulates various libp2p behaviours like Kademlia, Gossipsub, etc.
/// to enable different functionalities in the peer-to-peer network.
///
/// # Fields
///
/// * `request_response` - Handles request-response communication using CBOR serialization.
/// * `kademlia` - Kademlia distributed hash table behaviour for peer discovery and content routing.
/// * `identify` - Protocol for identifying other peers on the network.
/// * `gossipsub` - Gossipsub protocol for pub/sub messaging.
///
/// # Examples
///
/// Instantiating `Behaviour`:
///
/// ```ignore
/// let behaviour = Behaviour {
///     request_response: /* request_response behaviour */,
///     kademlia: /* kademlia behaviour */,
///     identify: /* identify behaviour */,
///     gossipsub: /* gossipsub behaviour */,
/// };
/// ```
#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub request_response: request_response::cbor::Behaviour<Request, Response>,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
}

/// Creates a new libp2p Swarm instance with specified behaviours and returns a `Client` for network operations.
///
/// This function sets up a new libp2p Swarm, configuring various behaviours like Kademlia, Gossipsub, etc.
/// It also prepares channels for command and event handling in the network.
///
/// # Arguments
///
/// * `secret_key_seed` - An optional seed for deterministic key generation. If `None`, random keys are generated.
///
/// # Returns
///
/// A `Result` containing a tuple of `Client`, an event stream, and `EventLoop`, or an error.
///
/// # Errors
///
/// Returns an error if there is a failure in setting up the Swarm or any of its behaviours.
///
/// # Examples
///
/// Creating a new client and event loop:
///
/// ```ignore
/// let (client, event_stream, event_loop) = new(Some(42)).await?;
/// ```
pub async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = Event>, EventLoop), Box<dyn Error>> {
    // Create a public/private key pair, either random or based on a seed.
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = id_keys.public().to_peer_id();
    debug!("Peer ID: {}", peer_id);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // To content-address message, we can take the hash of message and use it as an ID.
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // Set a custom gossipsub configuration
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                .build()?;
            //.map_err(|msg| tokio::io::Error::new(tokio::io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

            // build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let kademlia = kad::Behaviour::new(
                peer_id,
                kad::store::MemoryStore::new(key.public().to_peer_id()),
            );
            let request_response = request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/mpcnet/reqres/1.0.0"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            );

            let identify = identify::Behaviour::new(identify::Config::new(
                "/mpcnet/id/1.0.0".to_string(),
                key.public(),
            ));

            Ok(Behaviour {
                kademlia,
                request_response,
                identify,
                gossipsub,
            })
        })?
        .build();

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    // Create a Gossipsub topic
    let topic = IdentTopic::new("/mpcnet/pubsub/1.0.0".to_string());
    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    let (command_sender, command_receiver) = mpsc::channel(0);
    let (event_sender, event_receiver) = mpsc::channel(0);

    Ok((
        Client {
            sender: command_sender,
        },
        event_receiver,
        EventLoop::new(swarm, command_receiver, event_sender),
    ))
}
