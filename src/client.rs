use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use libp2p::{core::Multiaddr, request_response::ResponseChannel, PeerId};

use std::collections::HashSet;
use std::error::Error;

use crate::command::Command;
use crate::protocol::Response;
use crate::sss::Polynomial;

/// Represents a client in the network capable of issuing commands.
///
/// This struct provides an interface to interact with the network by sending various commands
/// like starting to listen, dialing peers, providing and getting shares, etc.
///
/// # Fields
///
/// * `sender` - A channel sender used to send commands to the network event loop.
///
/// # Examples
///
/// Creating a new `Client`:
///
/// ```rust
/// use futures::channel::mpsc;
/// use mpcnet::client::Client;
/// use mpcnet::command::Command;
///
/// let (sender, receiver) = mpsc::channel::<Command>(10);
/// let client = Client { sender };
/// ```
#[derive(Clone)]
pub struct Client {
    pub sender: mpsc::Sender<Command>,
}

impl Client {
    /// Listen for incoming connections on the given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The multiaddress to listen on.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// client.start_listening("/ip4/0.0.0.0/tcp/0".parse()?).await?;
    /// ```
    pub async fn start_listening(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Dial the given peer at the given address.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The `PeerId` of the peer to dial.
    /// * `peer_addr` - The multiaddress of the peer.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// client.dial(peer_id, peer_addr).await?;
    /// ```
    pub async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Advertise the local node as the provider of the given key on the DHT.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to start providing on the DHT.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// client.start_providing("my_key".to_string()).await;
    /// ```
    pub async fn start_providing(&mut self, key: String) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartProviding { key, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }

    /// Find the providers for the given key on the DHT.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for which to find providers.
    ///
    /// # Returns
    ///
    /// A set of `PeerId` representing the providers of the key.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let providers = client.get_providers("my_key".to_string()).await;
    /// ```
    pub async fn get_providers(&mut self, key: String) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetProviders { key, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Find all providers on the DHT.
    ///
    /// # Returns
    ///
    /// A set of `PeerId` representing all the providers in the DHT.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let all_providers = client.get_all_providers().await;
    /// ```
    pub async fn get_all_providers(&mut self) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetAllProviders { sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Request the content of the given share from the given peer.
    ///
    /// # Arguments
    ///
    /// * `peer` - The `PeerId` of the peer from whom to request the share.
    /// * `key` - The key of the share to request.
    /// * `sender` - The `PeerId` of the sender making the request.
    ///
    /// # Returns
    ///
    /// The requested share data upon success.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let share_content = client.request_share(peer_id, "my_key".to_string(), sender_id).await?;
    /// ```
    pub async fn request_share(
        &mut self,
        peer: PeerId,
        key: String,
        sender: PeerId,
    ) -> Result<(u8, Vec<u8>), Box<dyn Error + Send>> {
        let (sender_chan, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestShare {
                key,
                peer,
                sender,
                sender_chan,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    /// Respond with the provided share content to the given request.
    ///
    /// # Arguments
    ///
    /// * `share` - The share to respond with.
    /// * `success` - Whether the response is successful.
    /// * `channel` - The response channel to send the response.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// client.respond_share((1, vec![1, 2, 3]), true, response_channel).await;
    /// ```
    pub async fn respond_share(
        &mut self,
        share: (u8, Vec<u8>),
        success: bool,
        channel: ResponseChannel<Response>,
    ) {
        self.sender
            .send(Command::RespondShare {
                share,
                success,
                channel,
            })
            .await
            .expect("Command receiver not to be dropped.");
    }

    /// Request registration of the given share.
    ///
    /// # Arguments
    ///
    /// * `share` - The share to register.
    /// * `key` - The key associated with the share.
    /// * `peer` - The `PeerId` of the peer to register the share with.
    /// * `sender` - The `PeerId` of the sender making the request.
    ///
    /// # Returns
    ///
    /// `true` if the share was successfully registered.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let result = client.request_register_share((1, vec![1, 2, 3]), "my_key".to_string(), peer_id, sender_id).await?;
    /// ```
    pub async fn request_register_share(
        &mut self,
        share: (u8, Vec<u8>),
        key: String,
        threshold: u64,
        peer: PeerId,
        sender: PeerId,
    ) -> Result<bool, Box<dyn Error + Send>> {
        let (sender_chan, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestRegisterShare {
                share,
                key,
                peer,
                threshold,
                sender,
                sender_chan,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    /// Respond to a register share request.
    ///
    /// # Arguments
    ///
    /// * `success` - Whether the registration was successful.
    /// * `channel` - The response channel to send the response.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// client.respond_register_share(true, response_channel).await;
    /// ```
    pub async fn respond_register_share(
        &mut self,
        success: bool,
        channel: ResponseChannel<Response>,
    ) {
        self.sender
            .send(Command::RespondRegisterShare { success, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }

    /// Request the refreshing of shares.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the shares to refresh.
    /// * `refresh_key` - A list of polynomials for the refreshing process.
    /// * `peer` - The `PeerId` of the peer to refresh the shares with.
    /// * `sender` - The `PeerId` of the sender making the request.
    ///
    /// # Returns
    ///
    /// `true` if the shares were successfully refreshed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let result = client.request_refresh_shares("my_key".to_string(), vec![Polynomial::new(2, gf256::new(5))], peer_id, sender_id).await?;
    /// ```
    pub async fn request_refresh_shares(
        &mut self,
        key: String,
        refresh_key: Vec<Polynomial>,
        peer: PeerId,
        sender: PeerId,
    ) -> Result<bool, Box<dyn Error + Send>> {
        let (sender_chan, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestRefreshShare {
                key,
                refresh_key,
                peer,
                sender,
                sender_chan,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    /// Respond to a refresh shares request.
    ///
    /// # Arguments
    ///
    /// * `success` - Whether the refresh was successful.
    /// * `channel` - The response channel to send the response.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// client.respond_refresh_shares(true, response_channel).await;
    /// ```
    pub async fn respond_refresh_shares(
        &mut self,
        success: bool,
        channel: ResponseChannel<Response>,
    ) {
        self.sender
            .send(Command::RespondRefreshShare { success, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }
}
