use clap::{crate_version, Parser};

use futures::prelude::*;
use libp2p::PeerId;
use libp2p::{core::Multiaddr, multiaddr::Protocol};
use rand::seq::IteratorRandom;
use rand::RngCore;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::time::Duration;
use tokio::{spawn, time};
use tracing::debug;
use tracing::error;
use tracing_subscriber::EnvFilter;

use shard::constants::DEFAULT_REFRESH_SECONDS;
use shard::event::Event;
use shard::network;
use shard::protocol::Request;
use shard::provider::{
    dao, execute_get_share, execute_refresh_share, execute_register_share, refresh_loop,
};
use shard::sss::combine_shares;
use shard::sss::generate_refresh_key;
use shard::sss::split_secret;

#[derive(Debug, Parser)]
#[command(name = "shard")]
#[command(author = "lodge <jay.logelin@gmail.com>")]
#[command(version = crate_version!())]
#[command(
    about = "SHARD - SHARD Holds And Refreshes Data",
    long_about = "SHARD threshold network allows users to split secrets into shares, distribute them to share providers, and recombine them at a threshold to rebuild the secret. A node will provide shares to the shard, and refresh them automatically at a specified interval. It works by generating a new refresh key and then updating the shares across the network. The provider node persists all shares to a database, and will use the database on restart. Note that the database is in-memory by default, but can be set to a file-based database using the --db-path flag. Shares can only be retrieved or re-registered by the same client that registers the share with the network, identified by the client's peer ID, which is derived from their public key. Shares are automatically refreshed without changing the secret itself between share providers, enhancing the overall security of the network over time. The refresh interval is set using the --refresh-interval flag, and is set to 30 minutes by default"
)]
enum CliArgument {
    /// Run a share provider node that provides shares to shard users, and refresh them automatically at a specified interval, set using the --refresh-interval flag.
    Provide {
        /// use embedded database for persistence
        /// otherwise use memory database
        #[clap(long, short)]
        db_path: Option<String>,

        /// Share refresh interval in seconds
        // #[clap(long, short, default_value_t = 60)]
        #[clap(long, short)]
        refresh_interval: Option<u64>,
    },
    /// Combine shares from the network to rebuild a secret.
    Combine {
        /// key of the share to get.
        #[clap(long, short)]
        key: String,

        /// Share threshold, if none is provided, uses the number of shares
        #[clap(long, short)]
        threshold: Option<usize>,

        /// Verbose mode displays the shares
        #[clap(long, short)]
        verbose: bool,
    },
    /// Split a secret into shares and propagate them across the network.
    Split {
        /// Share threshold.
        #[clap(long, short)]
        threshold: usize,

        /// Number of shares to generate.
        #[clap(long, short)]
        shares: usize,

        /// key to use to register shares for the secret
        #[clap(long, short)]
        key: Option<String>,

        /// Secret to split.
        #[clap(long)]
        secret: String,

        /// Verbose mode displays the shares
        #[clap(long, short)]
        verbose: bool,
    },

    /// Get the list of share providers for a secret.
    Ls {
        /// key of the secret.
        #[clap(long, short)]
        key: String,
    },

    /// Refresh the shares
    Refresh {
        /// key of the secret.
        #[clap(long, short)]
        key: String,

        /// Share threshold.
        #[clap(long, short)]
        threshold: usize,

        /// Key size.
        #[clap(long, short)]
        size: usize,
    },
}

#[derive(Parser, Debug)]
#[clap(name = "shard Threshold Network")]
struct Opt {
    /// Fixed value to generate deterministic peer ID.
    #[clap(long, short)]
    secret_key_seed: Option<u8>,

    /// Address of a peer to connect to.
    #[clap(long, short)]
    peer: Option<Multiaddr>,

    /// Address to listen on.
    #[clap(long, short)]
    listen_address: Option<Multiaddr>,

    /// Subcommand to run.
    #[clap(subcommand)]
    argument: CliArgument,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sender = get_sender();
    debug!("sender ID: {}", sender);

    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let opt = Opt::parse();

    let (mut network_client, mut network_events, network_event_loop, local_peer_id) =
        network::new(opt.secret_key_seed).await?;

    // Spawn the network task for it to run in the background.
    spawn(network_event_loop.run());

    // In case a listen address was provided use it, otherwise listen on any
    // address.
    match opt.listen_address {
        Some(addr) => network_client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),
        None => network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    // In case the user provided an address of a peer on the CLI, dial it.
    if let Some(addr) = opt.peer {
        debug!("Dialing peer at {}.", addr);
        let Some(Protocol::P2p(peer_id)) = addr.iter().last() else {
            return Err("Expect peer multiaddr to contain peer ID.".into());
        };
        network_client
            .dial(peer_id, addr)
            .await
            .expect("Dial to succeed");
    }

    debug!("Waiting for network to be ready...");

    match opt.argument {
        // Providing a share.
        CliArgument::Provide {
            db_path,
            refresh_interval,
        } => {
            // check if the db_path is set, if so use sled, otherwise use HashMap
            let dao = dao(db_path).unwrap();

            // check if refresh is set, if not use a default of 30 minutes
            let refresh = refresh_interval.unwrap_or(DEFAULT_REFRESH_SECONDS);
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
                            execute_register_share(
                                &req.key,
                                &sender,
                                req.share,
                                req.threshold,
                                channel,
                                &dao,
                                &mut network_client,
                            )
                            .await?;
                        }
                        Request::GetShare(req) => {
                            let sender = PeerId::from_bytes(&req.sender).unwrap();
                            execute_get_share(
                                &req.key,
                                &sender,
                                channel,
                                &dao,
                                &mut network_client,
                            )
                            .await?;
                        }
                        Request::RefreshShare(req) => {
                            let sender = PeerId::from_bytes(&req.sender).unwrap();
                            execute_refresh_share(
                                &req.key,
                                &sender,
                                &req.refresh_key,
                                Some(channel),
                                &dao,
                                &mut network_client,
                            )
                            .await?;
                        }
                    },
                    e => debug!("unhandled client event: {e:?}"),
                }
            }
        }

        // Locating and getting a share.
        CliArgument::Combine {
            key,
            threshold,
            verbose,
        } => {
            // sleep for a bit to give the network time to bootstrap
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            debug!("Looking for providers of share {}...", key);
            // Locate all nodes providing the share.
            let providers = network_client.get_providers(key.clone()).await;
            if providers.is_empty() {
                return Err(format!("Could not find provider for share {key}.").into());
            }

            debug!("Found {} providers for share {}.", providers.len(), key);
            // get the threshold number of shares, if threshold is None, use the number of providers
            let threshold = threshold.unwrap_or_else(|| providers.len());

            // Request a share from each node.
            let requests = providers.into_iter().map(|p| {
                let mut network_client = network_client.clone();
                let name = key.clone();
                async move { network_client.request_share(p, name, sender).await }.boxed()
            });

            debug!("Requesting share from providers.");

            let rng = &mut rand::thread_rng();
            let sample = requests.choose_multiple(rng, threshold);

            let shares = futures::future::join_all(sample).await;
            let u: Vec<&(u8, Vec<u8>)> = shares
                .iter()
                .map(|r| {
                    if let Err(e) = r {
                        error!("Error: {:?}", e);
                    }
                    r.as_ref().unwrap()
                })
                .collect();

            // create a shares map for combining
            let shares_map: HashMap<u8, Vec<u8>> = u.iter().map(|(k, v)| (*k, v.clone())).collect();
            shares.iter().for_each(|r| {
                if let Err(e) = r {
                    error!("Error: {:?}", e);
                }
                debug!("Received r: {:?}", r.as_ref().unwrap());
            });

            let secret = combine_shares(&shares_map);
            debug!("Received shares: {:?}", &shares);

            // if the debug flag is set, print the shares
            if verbose {
                println!("🐛 shares: ");
                let mut items: Vec<_> = shares_map.iter().collect();
                items.sort_by(|a, b| a.1.cmp(b.1));

                // Now items is sorted by key, and you can iterate over it to get the values in order
                for (_, value) in items {
                    println!("  {}", hex::encode(value));
                }
            }

            println!(
                "🔑 secret: {:#?}",
                hex::encode(secret.unwrap())
            );

         
        }

        // Splitting a secret.
        CliArgument::Split {
            threshold,
            shares,
            secret,
            key,
            verbose,
        } => {
            // sleep for a bit to give the network time to bootstrap
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // if key is None assign a random key
            let key = key.unwrap_or_else(|| {
                let mut rng = rand::thread_rng();
                let mut key = [0u8; 32];
                rng.fill_bytes(&mut key);
                hex::encode(key)
            });

            let split_shares = split_secret(secret.as_bytes(), threshold, shares)?;
            debug!("Shares: {:?}", split_shares);
            // Locate all nodes providing the share.
            let providers = network_client.get_all_providers().await;
            if providers.is_empty() {
                return Err(format!("Could not find providers.").into());
            }
            // check that there are the correct number of providers
            if providers.len() < shares {
                return Err(format!(
                    "Not enough providers to accomodate shares. Wait for more providers to join"
                )
                .into());
            }

            debug!("*** Found {} providers.", providers.len());

            // select shares number of providers
            let rng = &mut rand::thread_rng();
            let providers_sample = providers.into_iter().choose_multiple(rng, shares);

            // make sure to only send shares to only shares number of providers
            let requests = providers_sample
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    let mut network_client = network_client.clone();
                    let share_id = (i + 1) as u8;
                    let share = split_shares.get(&share_id).ok_or("Share not found");
                    let k = &key;
                    async move {
                        network_client
                            .request_register_share(
                                (share_id, share.unwrap().to_vec()),
                                k.to_string(),
                                threshold as u64,
                                p,
                                sender,
                            )
                            .await
                    }
                    .boxed()
                });

            // Await all of the requests and ensure they all succee
            futures::future::join_all(requests)
                .await
                .iter()
                .for_each(|r| {
                    if let Err(e) = r {
                        error!("Error: {:?}", e);
                    }
                });

            if verbose {
                println!("🐛 shares: ");
                let mut items: Vec<_> = split_shares.iter().collect();
                items.sort_by(|a, b| a.1.cmp(b.1));

                // Now items is sorted by key, and you can iterate over it to get the values in order
                for (_, value) in items {
                    println!("  {}", hex::encode(value));
                }
            }

            println!("✂️  Secret has been split and distributed across network.");
            println!("    key: {:#?}", key);
            println!("    threshold: {:#?}", threshold);
            println!("    providers: {:#?}", providers_sample)
        }
        CliArgument::Ls { key } => {
            let providers = network_client.get_providers(key.clone()).await;
            if providers.is_empty() {
                return Err(format!("Could not find provider for share {key}.").into());
            }

            // println!("Found {} providers for share {}.", providers.len(), key);
            println!("✂️  Share Providers: {:#?}", providers);
        }
        CliArgument::Refresh {
            key,
            threshold,
            size,
        } => {
            // sleep for a bit to give the network time to bootstrap
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let providers = network_client.get_providers(key.clone()).await;
            if providers.is_empty() {
                return Err(format!("Could not find provider for share {key}.").into());
            }

            debug!("Found {} providers for share {}.", providers.len(), key);

            let refresh_key = generate_refresh_key(threshold, size).unwrap();
            debug!("🔑 Refresh Key: {:#?}", refresh_key);

            let requests = providers.clone().into_iter().map(|p| {
                let k = key.clone();
                let ref_key = refresh_key.clone();
                let mut network_client = network_client.clone();
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

            debug!("Found {} providers for share {}.", providers.len(), key);
            print!(
                "🔄 Refreshed {} shares for key: {:?}",
                providers.len(),
                &key
            );
        }
    }

    Ok(())
}

fn get_sender() -> PeerId {
    // create a deterministic peer id from a fixed value
    let mut bytes = [0u8; 32];
    bytes[0] = 42;
    let id_keys = libp2p::identity::Keypair::ed25519_from_bytes(bytes).unwrap();
    //let id_keys = libp2p::identity::Keypair::generate_ed25519();
    id_keys.public().to_peer_id()
}
