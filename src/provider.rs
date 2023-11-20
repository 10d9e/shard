use crate::{
    client::Client,
    protocol::Response,
    repository::{ShareEntry, ShareEntryDaoTrait, SledShareEntryDao, HashMapShareEntryDao},
    sss::{generate_refresh_key, refresh_share, Polynomial},
};
use futures::prelude::*;
use libp2p::request_response::ResponseChannel;
use libp2p::PeerId;
use std::{sync::{Arc, Mutex}, collections::HashMap};
use tokio::time::Interval;
use tracing::{debug, error};

pub fn check_share_owner(entry: &ShareEntry, sender_id: &PeerId) -> bool {
    PeerId::from_bytes(&entry.sender).unwrap() == *sender_id
}

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

    println!("-- share before refresh: {:?}", share_entry.share);
    let _ = refresh_share(
        (&mut share_entry.share.0, &mut share_entry.share.1),
        refresh_key,
    );
    dao.lock().unwrap().insert(key, &share_entry)?;
    println!("-- share after refresh:  {:?}", share_entry.share);

    let test = dao
        .lock()
        .unwrap()
        .get(&key)
        .unwrap()
        .ok_or("Share not found")?;
    println!("-- test share from dao: {:?}", test.share);

    if channel.is_some() {
        network_client
            .respond_refresh_shares(true, channel.unwrap())
            .await;
    }
    println!("🔄 Refreshed share for key: {:?}", key);
    Ok(())
}

pub async fn execute_register_share(
    key: &str,
    sender: &PeerId,
    share: (u8, Vec<u8>),
    channel: ResponseChannel<Response>,
    dao: &Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>,
    network_client: &mut Client,
) -> Result<(), Box<dyn std::error::Error>> {
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
            share: share,
            sender: sender.to_bytes(),
        },
    )?;
    network_client.respond_register_share(true, channel).await;
    println!("🚀 Registered share for key: {:?}.", key);

    Ok(())
}

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

pub fn dao(db_path: Option<String>) -> Result<Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>, Box<dyn std::error::Error>> {
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

pub async fn refresh_loop(
    interval: &mut Interval,
    dao_clone: Arc<Mutex<Box<dyn ShareEntryDaoTrait>>>,
    network_client_clone: &mut Client,
    local_peer_id: PeerId,
) {
    loop {
        interval.tick().await;
        // Your task here
        println!("Starting refresh.");

        // get all the shares
        let shares = dao_clone.lock().unwrap().get_all().unwrap();
        debug!("shares: {:?}", shares);

        // iterate over the shares and refresh them
        for (key, share_entry) in shares.iter() {
            debug!("key: {:?}", key);
            debug!("share_entry: {:?}", share_entry);
            let sender = PeerId::from_bytes(&share_entry.sender).unwrap();
            debug!("sender: {:?}", sender);

            let secret_len = share_entry.share.1.len();
            // generate a new refresh key
            let refresh_key = generate_refresh_key(2, secret_len).unwrap();
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
