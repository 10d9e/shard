use crate::event::Event;
use crate::{
    client::Client,
    constants::DEFAULT_REFRESH_SECONDS,
    protocol::{Request, Response},
    repository::{HashMapShareEntryDao, ShareEntry, ShareEntryDaoTrait, SledShareEntryDao},
    sss::{generate_refresh_key, refresh_share, Polynomial},
};
use futures::future::FutureExt;
use futures::prelude::*;
use libp2p::request_response::ResponseChannel;
use libp2p::PeerId;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    spawn,
    time::{self, Interval},
};
use tracing::{debug, error};

/// Checks if the given `PeerId` is the owner of the `ShareEntry`.
///
/// # Arguments
/// * `entry` - A reference to the `ShareEntry` to check.
/// * `sender_id` - The `PeerId` to validate against the owner in the `ShareEntry`.
///
/// # Returns
/// Returns `true` if `sender_id` matches the owner in the `ShareEntry`, otherwise `false`.
pub fn check_share_owner(entry: &ShareEntry, sender_id: &PeerId) -> bool {
    PeerId::from_bytes(&entry.sender).unwrap() == *sender_id
}

/// Executes the share refresh logic asynchronously.
///
/// This function retrieves the specified `ShareEntry` from the database, refreshes its share,
/// and then updates the entry in the database. If a response channel is provided, it sends a
/// response back to the network client.
///
/// # Arguments
/// * `key` - The key identifying the `ShareEntry` to refresh.
/// * `sender` - The `PeerId` of the sender requesting the refresh.
/// * `refresh_key` - A slice of `Polynomial` used for refreshing the share.
/// * `channel` - An optional `ResponseChannel<Response>` for sending responses.
/// * `dao` - A shared and mutable reference to the data access object (DAO) trait object.
/// * `network_client` - A mutable reference to the network client for responding to requests.
///
/// # Returns
/// Returns a `Result<(), Box<dyn std::error::Error>>`, indicating success or failure.
pub async fn execute_refresh_share(
    key: &str,
    sender: &PeerId,
    refresh_key: &[Polynomial],
    channel: Option<ResponseChannel<Response>>,
    dao: &Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>,
    network_client: &mut Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut share_entry: ShareEntry = dao
        .lock()
        .unwrap()
        .get(key)
        .unwrap()
        .ok_or("Share not found")?;

    //let sender = PeerId::from_bytes(&sender).unwrap();
    debug!("-- Sender: {:#?}.", sender);

    // check that the peer requesting the share is the owner
    // only if the channel is not None
    if channel.is_some() {
        if !check_share_owner(&share_entry, sender) {
            println!(
                "⚠️ Share not owned by sender {:?}, actual owner: {:?}",
                sender,
                PeerId::from_bytes(&share_entry.sender).unwrap()
            );

            network_client
                .respond_refresh_shares(false, channel.unwrap())
                .await;

            return Ok(());
        }
    }

    debug!("-- share before refresh: {:?}", share_entry.share);
    let _ = refresh_share(
        (&mut share_entry.share.0, &mut share_entry.share.1),
        refresh_key,
    );
    dao.lock().unwrap().insert(key, &share_entry)?;
    debug!("-- share after refresh:  {:?}", share_entry.share);

    let test = dao
        .lock()
        .unwrap()
        .get(&key)
        .unwrap()
        .ok_or("Share not found")?;
    debug!("-- test share from dao: {:?}", test.share);

    if channel.is_some() {
        network_client
            .respond_refresh_shares(true, channel.unwrap())
            .await;
    }
    println!("🔄 Refreshed share for key: {:?}", key);
    Ok(())
}

/// Executes the share registration logic asynchronously.
///
/// This function checks for the existence of a share in the database and registers a new
/// share if it doesn't exist or if the sender is the owner. It then sends a response back
/// to the network client.
///
/// # Arguments
/// * `key` - The key identifying the share to register.
/// * `sender` - The `PeerId` of the sender requesting the registration.
/// * `share` - A tuple containing the share identifier and data.
/// * `threshold` - The threshold value for the share.
/// * `channel` - The `ResponseChannel<Response>` for sending responses.
/// * `dao` - A shared and mutable reference to the DAO trait object.
/// * `network_client` - A mutable reference to the network client.
///
/// # Returns
/// Returns a `Result<(), Box<dyn std::error::Error>>`, indicating success or failure.
pub async fn execute_register_share(
    key: &str,
    sender: &PeerId,
    share: (u8, Vec<u8>),
    threshold: u64,
    channel: ResponseChannel<Response>,
    dao: &Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>,
    network_client: &mut Client,
) -> Result<(), Box<dyn std::error::Error>> {
    // check if the share already exists and if so, check that the peer requesting the share is the owner
    if let Some(share_entry) = dao.lock().unwrap().get(key)? {
        debug!("Retrieved Entry: {:?}", share_entry);
        debug!("-- Sender: {:#?}.", sender);

        // check that the peer requesting the share is the owner
        if !check_share_owner(&share_entry, &sender) {
            println!(
                "⚠️ Share exists, not owned by sender {:?}, actual owner: {:?}",
                sender, share_entry.sender
            );
            network_client.respond_register_share(false, channel).await;
            return Ok(());
        }
    }

    network_client.start_providing(key.to_string()).await;
    debug!("-- Sender: {:#?}.", sender);
    dao.lock().unwrap().insert(
        key,
        &ShareEntry {
            share,
            sender: sender.to_bytes(),
            threshold,
        },
    )?;
    network_client.respond_register_share(true, channel).await;
    println!("🚀 Registered share for key: {:?}.", key);

    Ok(())
}

/// Executes the logic to retrieve and send a share asynchronously.
///
/// This function retrieves a `ShareEntry` from the database and sends it back to the requester
/// via the network client, if the requester is the owner of the share.
///
/// # Arguments
/// * `key` - The key identifying the share to retrieve.
/// * `sender` - The `PeerId` of the sender requesting the share.
/// * `channel` - The `ResponseChannel<Response>` for sending the share.
/// * `dao` - A shared and mutable reference to the DAO trait object.
/// * `network_client` - A mutable reference to the network client.
///
/// # Returns
/// Returns a `Result<(), Box<dyn std::error::Error>>`, indicating success or failure.
pub async fn execute_get_share(
    key: &str,
    sender: &PeerId,
    channel: ResponseChannel<Response>,
    dao: &Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>,
    network_client: &mut Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let share_entry = dao
        .lock()
        .unwrap()
        .get(&key)
        .unwrap()
        .ok_or("Share not found")?;

    debug!("-- Sender: {:#?}.", sender);

    // check that the peer requesting the share is the owner
    if !check_share_owner(&share_entry, &sender) {
        println!(
            "⚠️ Share not owned by sender {:?}, actual owner: {:?}",
            sender, share_entry.sender
        );
        network_client
            .respond_share((0u8, vec![]), false, channel)
            .await;
        return Ok(());
    }
    network_client
        .respond_share(share_entry.share.clone(), true, channel)
        .await;
    println!("💡 Sent share for key: {:?}.", key);

    Ok(())
}

/// Creates and returns a DAO instance based on the specified database path.
///
/// If a path is provided, a Sled database DAO is created; otherwise, an in-memory HashMap
/// DAO is used. This allows flexibility in choosing the underlying storage mechanism.
///
/// # Arguments
/// * `db_path` - An optional string slice representing the path to the database.
///
/// # Returns
/// Returns a `Result<Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>>`, encapsulating the DAO in a
/// thread-safe, reference-counted pointer, or an error if the database cannot be initialized.
pub fn dao(
    db_path: Option<String>,
) -> Result<Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>, Box<dyn std::error::Error>> {
    // check if the db_path is set, if so use sled, otherwise use HashMap
    let dao: Arc<Mutex<Box<dyn ShareEntryDaoTrait>>> = if db_path.is_some() {
        debug!("Using Sled DB");
        Arc::new(Mutex::new(Box::new(SledShareEntryDao::new(
            &db_path.unwrap(),
        )?)))
    } else {
        debug!("Using HashMap DB");
        Arc::new(Mutex::new(Box::new(HashMapShareEntryDao {
            map: Mutex::new(HashMap::new()),
        })))
    };
    Ok(dao)
}

/// Runs the main event loop asynchronously.
///
/// This function initializes the DAO and starts a periodic refresh task. It also listens for
/// incoming network events and handles them appropriately.
///
/// # Arguments
/// * `db_path` - An optional string slice for the database path.
/// * `refresh` - An optional duration in seconds for the refresh interval.
/// * `local_peer_id` - The `PeerId` of the local node.
/// * `network_client` - A mutable reference to the network client.
/// * `network_events` - A stream of network events to listen to.
pub async fn run_loop(
    db_path: Option<String>,
    refresh: Option<u64>,
    local_peer_id: PeerId,
    network_client: &mut Client,
    mut network_events: impl Stream<Item = Event> + Unpin,
) {
    // check if the db_path is set, if so use sled, otherwise use HashMap
    let dao: Arc<Mutex<Box<dyn ShareEntryDaoTrait>>> = dao(db_path).unwrap();

    // check if refresh is set, if not use a default of 30 minutes
    let refresh = refresh.unwrap_or(DEFAULT_REFRESH_SECONDS);
    debug!("Using refresh_seconds: {}", refresh);

    // spawn a refresh task to run every refresh_seconds seconds
    let dao_clone = Arc::clone(&dao);
    let mut network_client_clone = network_client.clone();
    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(refresh));
        refresh_loop(
            &mut interval,
            dao_clone,
            &mut network_client_clone,
            local_peer_id,
        )
        .await;
    });

    loop {
        match network_events.next().await {
            // Reply with the content of the file on incoming requests.
            Some(Event::InboundRequest { request, channel }) => match request {
                Request::RegisterShare(req) => {
                    let sender = PeerId::from_bytes(&req.sender).unwrap();
                    let _ = execute_register_share(
                        &req.key,
                        &sender,
                        req.share,
                        req.threshold,
                        channel,
                        &dao,
                        network_client,
                    )
                    .await;
                }
                Request::GetShare(req) => {
                    let sender = PeerId::from_bytes(&req.sender).unwrap();
                    let _ =
                        execute_get_share(&req.key, &sender, channel, &dao, network_client).await;
                }
                Request::RefreshShare(req) => {
                    let sender = PeerId::from_bytes(&req.sender).unwrap();
                    let _ = execute_refresh_share(
                        &req.key,
                        &sender,
                        &req.refresh_key,
                        Some(channel),
                        &dao,
                        network_client,
                    )
                    .await;
                }
            },
            e => debug!("unhandled client event: {e:?}"),
        }
    }
}

/// Periodically refreshes shares in a separate asynchronous task.
///
/// This function iterates over all shares in the database at regular intervals and refreshes
/// them. It also communicates with other peers in the network to synchronize the refreshed shares.
///
/// # Arguments
/// * `interval` - A mutable reference to a time interval generator.
/// * `dao_clone` - A cloned reference to the DAO, wrapped in an Arc and Mutex.
/// * `network_client_clone` - A cloned mutable reference to the network client.
/// * `local_peer_id` - The `PeerId` of the local node.
pub async fn refresh_loop(
    interval: &mut Interval,
    dao_clone: Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>,
    network_client_clone: &mut Client,
    local_peer_id: PeerId,
) {
    loop {
        interval.tick().await;
        debug!("Starting refresh.");

        // get all the shares
        let shares = dao_clone.lock().unwrap().get_all().unwrap();
        debug!("shares: {:?}", shares);

        // iterate over the shares and refresh them
        for (key, share_entry) in shares.iter() {
            debug!("key: {:?}", key);
            debug!("share_entry: {:?}", share_entry);
            let sender = PeerId::from_bytes(&share_entry.sender).unwrap();
            debug!("sender: {:?}", sender);

            // determine the threshold from the share
            let secret_len = share_entry.share.1.len();
            // generate a new refresh key
            let refresh_key =
                generate_refresh_key(share_entry.threshold as usize, secret_len).unwrap();
            debug!("🔑 Refresh Key: {:#?}", refresh_key);

            // get the providers for the share
            let providers = network_client_clone.get_providers(key.clone()).await;
            if providers.is_empty() {
                error!("Could not find provider for share {key}.");
                continue;
            }

            debug!("Found {} providers for share {}.", providers.len(), key);

            // refresh the share locally
            let _ = execute_refresh_share(
                key,
                &local_peer_id,
                &refresh_key,
                None,
                &dao_clone,
                &mut network_client_clone.clone(),
            )
            .await;

            // remove local_peer_id from providers
            let providers = providers
                .into_iter()
                .filter(|p| p != &local_peer_id)
                .collect::<Vec<_>>();

            let requests = providers.clone().into_iter().map(|p| {
                let k = key.clone();
                let ref_key = refresh_key.clone();
                let mut network_client = network_client_clone.clone();
                debug!("🔄 Refreshing share for key: {:?} to peer {:?}", &k, p);
                async move {
                    network_client
                        .request_refresh_shares(k, ref_key, p, sender)
                        .await
                }
                .boxed()
            });

            // Await all of the requests and ensure they all succeed
            futures::future::join_all(requests).await;

            // println!("Found {} providers for share {}.", providers.len(), key);
            debug!(
                "🔄 Refreshed {} shares for key: {:?}",
                providers.len(),
                &key
            );
        }
    }
}
