use std::sync::{Arc, Mutex};

use crate::{
    client::Client,
    protocol::Response,
    repository::{ShareEntry, ShareEntryDaoTrait},
    sss::{refresh_share, Polynomial},
};
use libp2p::request_response::ResponseChannel;
use libp2p::PeerId;
use tracing::debug;

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
    //Request::RefreshShare(req) => {
    //debug!("-- Request: {:#?}.", req);

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
                sender, PeerId::from_bytes(&share_entry.sender).unwrap()
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
