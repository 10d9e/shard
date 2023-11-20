//! # Shamir Secret Sharing with Proactive Share Refresh
//!
//! This library implements a network that supoorts Shamir's Secret Sharing (SSS) and Proactive Share Refreshing mechanisms.
//! It provides functionality to split secrets into shares, distribute them among participants, and
//! proactively refresh these shares over time to enhance security.
//!
//! ## Shamir's Secret Sharing (SSS)
//!
//! Shamir's Secret Sharing is a cryptographic algorithm created by Adi Shamir. It is a form of secret
//! sharing, where a secret is divided into parts, giving each participant its own unique part, with
//! the property that a certain number of these parts are needed to reconstruct the secret.
//!
//! ### The Mathematics Behind SSS
//!
//! The idea of SSS is based on polynomial interpolation in finite fields. Given a secret `S`, the
//! algorithm chooses a random polynomial of degree `t-1` (where `t` is the threshold number of shares
//! needed to reconstruct the secret):
//!
//! ```ignore
//! f(x) = a0 + a1*x + a2*x^2 + ... + a(t-1)*x^(t-1)
//! ```
//!
//! where `a0 = S` (the secret), and `a1, ..., a(t-1)` are randomly chosen coefficients. Each share
//! corresponds to a point `(x, f(x))` on this polynomial. With at least `t` points, the polynomial
//! and hence the secret can be reconstructed using Lagrange interpolation.
//!
//! ### Proactive Secret Sharing
//!
//! Proactive Secret Sharing is an extension to SSS, enhancing its security. Over time, the shares of
//! the secret are refreshed without changing the secret itself. This is achieved by generating a new
//! polynomial with the same constant term (the secret) and different higher-degree coefficients,
//! then updating the shares accordingly.
//!
//! ## Usage in the Code
//!
//! The `sss` module in this library provides functionalities to perform Shamir's Secret Sharing and
//! proactive secret refreshing. It includes creating polynomials, generating shares, and refreshing
//! them.
//!
//! ### Example: Splitting a Secret
//!
//! To split a secret using Shamir's Secret Sharing:
//!
//! ```rust
//! use shard::sss::Polynomial;
//! use gf256::gf256;
//!
//! let secret = gf256::new(123); // Your secret
//! let degree = 2; // Degree of the polynomial (threshold - 1)
//! let poly = Polynomial::new(degree, secret); // Create a polynomial with the secret as the constant term
//!
//! // Generate shares
//! let shares: Vec<(u8, gf256)> = (1..=5).map(|i| (i, poly.evaluate(gf256::new(i)))).collect();
//! // Now `shares` holds 5 shares of the secret
//! ```
//!
//! ### Example: Refreshing Shares
//!
//! To proactively refresh the shares:
//!
//! ```ignore
//! use shard::sss::Polynomial;
//! use gf256::gf256;
//! // Assume `old_shares` is the existing shares
//! let new_poly = Polynomial::new(0, gf256::new(0)); // New polynomial with zero constant term
//!
//! // Refresh shares
//! let refreshed_shares: Vec<(u8, gf256)> = old_shares
//!     .iter()
//!     .map(|&(x, y)| {
//!         let new_y = new_poly.evaluate(gf256::new(x));
//!         (x, y + new_y) // Add the new value to the old share
//!     })
//!     .collect();
//! // Now `refreshed_shares` holds the refreshed shares
//! ```
//!
//! ## Modules
//!
//! - `client`: Defines the network client functionality.
//! - `command`: Contains commands used in network operations.
//! - `event`: Defines various network events.
//! - `network`: Implements network behaviors and utilities.
//! - `protocol`: Defines the network communication protocol.
//! - `repository`: Manages data storage and retrieval.
//! - `sss`: Implements Shamir's Secret Sharing and proactive secret refreshing.
//!
//! [More detailed documentation and examples are provided in each module.]

/// The `client` module defines the network client functionalities, enabling interactions with the
/// network, such as sending and receiving messages, handling requests, and other peer-to-peer
/// communication features.
pub mod client;

/// The `command` module contains definitions of various commands used in network operations. These
/// commands represent different actions that can be performed in the network, such as dialing other
/// peers, starting to listen for connections, and managing secret shares.
pub mod command;

/// The `event` module defines the different types of events that can occur in the network, such as
/// inbound requests or updates in the network state. This module helps in handling asynchronous
/// network events in a structured manner.
pub mod event;

/// The `network` module implements the necessary network behaviors and utilities. It encapsulates
/// the logic for network interactions, including setting up the network, handling peer discovery,
/// and managing communication protocols.
pub mod network;

/// The `protocol` module defines the communication protocols used in the network. It includes the
/// specifications for various request and response formats, ensuring standardized communication
/// across different network nodes.
pub mod protocol;

/// The `repository` module manages data storage and retrieval. It is responsible for persisting
/// important data, like secret shares, and provides interfaces for accessing and updating this data.
pub mod repository;

/// The `sss` (Shamir's Secret Sharing) module is a crucial component of the library. It implements
/// the Shamir's Secret Sharing algorithm and proactive secret sharing. This module provides
/// functionalities to split secrets into shares, distribute them, and proactively refresh these
/// shares to enhance security.
pub mod sss;

/// The `provider` module defines the `Provider` trait, which is used to implement different
/// providers for the network. A provider is responsible for managing the network state, including
/// the secret shares and the peer list.
pub mod provider;

/// The `constants` module defines various constants used in the library.
pub mod constants;
