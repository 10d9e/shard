# SHARD - SHARD Holds And Refreshes Data 🔑

[![build-badge](https://github.com/jlogelin/shard/actions/workflows/build.yml/badge.svg)](https://nightly.link/jlogelin/shard/workflows/build/main/binaries.zip)
[![document-badge](https://github.com/jlogelin/shard/actions/workflows/doc.yml/badge.svg)](https://jlogelin.github.io/shard)
[![license-badge](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

SHARD is a decentralized network node built to facilitate a secure and robust threshold network using Shamir's Secret Sharing (SSS) and proactive secret refreshing. It allows users to split secrets into shares, distribute them across the network, combine them to reconstruct the original secret safely. Nodes non-interactively refresh these shares to harden the overall security of the network. It has a robust p2p network [design](https://github.com/jlogelin/shard#design), checkout the testnet examples [here](https://github.com/jlogelin/shard#example-5-node-local-network) and [here](https://github.com/jlogelin/shard#example-10-node-docker-network).

## Getting Started

To use shard, follow these steps:

1. **Installation**: Compile and install shard from the source code.
2. **Running a Node**: Start an shard node using the `provide` command. (provider nodes automatically perform regular shard refreshes)
3. **Splitting a Secret**: Use the `split` command to divide a secret into shares.
4. **Combining Shares**: Reconstruct the secret using the `combine` command with the required number of shares.
5. **Refreshing Shares**: Regularly refresh shares using the `refresh` command to maintain security.

## Usage

All commands, except for `provide` are for clients to interface with the network.

```bash
❯ shard --help
SHARD threshold network allows users to split secrets into shares, distribute them to share providers, and recombine them at a threshold to rebuild the secret. A node will provide shares to the shard, and refresh them automatically at a specified interval. It works by generating a new refresh key and then updating the shares across the network. The provider node persists all shares to a database, and will use the database on restart. Note that the database is in-memory by default, but can be set to a file-based database using the --db-path flag. Shares can only be retrieved or re-registered by the same client that registers the share with the network, identified by the client's peer ID, which is derived from their public key. Shares are automatically refreshed without changing the secret itself between share providers, enhancing the overall security of the network over time. The refresh interval is set using the --refresh-interval flag, and is set to 30 minutes by default

Usage: shard [OPTIONS] <COMMAND>

Commands:
  provide  Run a share provider node that provides shares to shard users, and refresh them automatically at a specified interval, set using the --refresh-interval flag
  combine  Combine shares from the network to rebuild a secret
  split    Split a secret into shares and propagate them across the network
  ls       Get the list of share providers for a secret
  refresh  Refresh the shares
  help     Print this message or the help of the given subcommand(s)

Options:
  -s, --secret-key-seed <SECRET_KEY_SEED>
          Fixed value to generate deterministic peer ID

  -p, --peer <PEER>
          Address of a peer to connect to

  -l, --listen-address <LISTEN_ADDRESS>
          Address to listen on

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Proactive share refresh
 
Every node on the network implements the proactive share refresh mechanism at a set interval. It provides functionalities to refresh the shares of a secret, without changing the secret itself. This is achieved by generating a new polynomial with a zero constant term ("the secret") and different higher-degree coefficients, Then, the new polynomial is evaluated at each point `(x, y)` in the existing shares, and the new value is added to the old share to get the refreshed share.

Proactive secret refreshing is an extension to SSS, enhancing its security. The shares of the secret are automatically refreshed without changing the secret itself. The refresh interval (in seconds) is set using the `--refresh-interval` flag in `provide` mode, and is set to 30 minutes by default.

### Example 5-node local network
Make sure you have the latest version of Rust installed. To build the node, open your terminal and navigate to the project directory where 'shard' is located, and then run the following command:

**0. Install rust**

Make sure that you have 🦀 [rust](https://www.rust-lang.org/tools/install) installed.

**1. Build the Node**

```bash
cargo build --release
```

This command compiles the 'shard' project in release mode, ensuring optimal performance.

**2. Start the Bootstrapper Node**

The bootstrapper node is essential for the network to operate. It acts as the initial point of contact for other nodes. To start the bootstrapper node, use the following command:

```bash
./target/release/shard --listen-address /ip4/127.0.0.1/tcp/40837 --secret-key-seed 1 provide --refresh-interval 10
```

This command starts 'shard' as a bootstrapper node, listening on address '/ip4/127.0.0.1/tcp/40837' and using the secret key seed '1'. ~Note: Here we are overriding the default `--refresh-interval` to run every 10 seconds, from the default 30 minutes, to demonstrate provider refreshing~

**3. Start Provider Nodes**

You'll need multiple provider nodes to participate in the network. (To run locally we will just use an in-memory database, however, in a real world scenario, we would use the `db-path` option to define the path for share persistence.) To start a provider node, open multiple terminals (4 in this case) and run the following command in 4 separate terminals:

```bash
./target/release/shard --peer provide
```

Each command starts a provider node, and they all connect to the bootstrapper node specified by the `--peer` argument.

**4. Start a Client Node**

To interact with the network, you'll need a client node. Open another terminal and run the following command:

```bash
❯ ./target/release/shard split --threshold 3 --shares 5 --secret butterbeer --key test --verbose

🐛 [debug] shares: 
  6720c971ea73b326ea88
  61461f77eb862dafeb24
  6413a2726487fcec64de
  6eb9c5786785436a6737
  6d8aae7be9710ca0e961
✂️  Secret has been split and distributed across network.
    key: "test"
    threshold: 3
    providers: {
    PeerId(
        "12D3KooWRbMntkocaizeD29DyhwZ5oujjRWJY8uTaUb2LkhrPF6W",
    ),
    PeerId(
        "12D3KooWEVAbvQvN3iH68DQW3wVDzHEwNyfLo7deW1jEQBGY1rkd",
    ),
    PeerId(
        "12D3KooWLP5k1NLj4F5KahMZk253Ndsu6jjNAfkeAFTjNcMAPJys",
    ),
    PeerId(
        "12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X",
    ),
    PeerId(
        "12D3KooWLbyBNppB3nLez7kTeD9c1JeYivDwUVUYoJeHgQHWdKP9",
    ),
}
```

This command splits a secret into 5 shares with a threshold of 2 and broadcasts them to the network. You can optionally view the shares using the `--verbose` flag.

**5. Query the Network for Shares**

To query the network for all nodes hosting shares for a specific key (in this case, 'test'), use the following command:

```bash
❯ ./target/release/shard ls --key test
✂️  Share Providers: {
    PeerId(
        "12D3KooWEVAbvQvN3iH68DQW3wVDzHEwNyfLo7deW1jEQBGY1rkd",
    ),
    PeerId(
        "12D3KooWRbMntkocaizeD29DyhwZ5oujjRWJY8uTaUb2LkhrPF6W",
    ),
    PeerId(
        "12D3KooWLbyBNppB3nLez7kTeD9c1JeYivDwUVUYoJeHgQHWdKP9",
    ),
    PeerId(
        "12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X",
    ),
    PeerId(
        "12D3KooWLP5k1NLj4F5KahMZk253Ndsu6jjNAfkeAFTjNcMAPJys",
    ),
}
```

This command retrieves information about nodes hosting shares for the specified key.

**6. Refresh Shares**

The refresh command can be run as many times as you want, and it doesn't require client interaction. ~Note: that this is an interactive version of refreshing. Each node performs it's own refresh operations that propagates through the network automatically~ Use the following command to refresh shares:

```bash
❯ ./target/release/shard refresh --key test --threshold 3 --size 10
🔄 Refreshed 5 shares for key: "test"
```

This command refreshes the shares for the 'test' key with a threshold of 2 and a size of 10.

**7. Reassemble the Secret**

To reassemble the secret, you'll need to randomly select 2 providers corresponding to the threshold. Use the following command:

```bash
❯ ./target/release/shard combine --key test --verbose
🐛 shares: 
  32a48be4c9fb1c698580
  3e0d5de4d92bcb127064
  6edca27475a2b51e9096
  9043ef808c5a8bfdc9a0
  cc3bc6103003228adcb6
🔑 secret: "butterbeer"

Attempting to reassemble below the threshold results in an error:

```bash
❯ ./target/release/shard combine --key test --verbose --threshold 2
🐛 shares: 
  425ecc0bba2487cd7405
  c96f907814d92f7d6429
🔑 secret: "Error: Unable to combine shares at threshold"
```

This command combines the shares from the selected providers to reassemble the secret. You can use the `--verbose` flag to view share bytes during the process.

Each node logs output as it interacts with the network:

```bash
🚀 Registered share for key: "test".
🔄 Refreshed share for key: "test"
🔄 Refreshed share for key: "test"
🔄 Refreshed share for key: "test"
🔄 Refreshed share for key: "test"
💡 Sent share for key: "test".
```

#### Example 10-node Docker network

A docker-based testnet is provided that includes 10 nodes and a client container for testing commands.

### 1. Start the shard test network

```bash
docker-compose up --build
```

### 2. Execute the commands within the testnet

#### From the host operating system

```bash
docker exec -it client shard --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X split --threshold 7 --shares 10 --secret butterbeer --key test
docker exec -it client shard --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X ls --key test
docker exec -it client shard --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X refresh --key test --threshold 7 --size 10
docker exec -it client shard --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X combine --key test
```

#### Interactively within the `client` container

```bash
❯ docker exec -it client /bin/bash 
root@71b4dce52922:/app# shard --help
SHARD threshold network allows users to split ...
```

Observe the node participants log statements with each command:

```bash
provider_2      | 🚀 Registered share for key: "test".
bootstrapper_1  | 🚀 Registered share for key: "test".
provider_5      | 🚀 Registered share for key: "test".
provider_9      | 🚀 Registered share for key: "test".
provider_3      | 🚀 Registered share for key: "test".
provider_8      | 🚀 Registered share for key: "test".
provider_6      | 🚀 Registered share for key: "test".
provider_7      | 🚀 Registered share for key: "test".
provider_4      | 🚀 Registered share for key: "test".
provider_1      | 🚀 Registered share for key: "test".
provider_2      | 💡 Sent share for key: "test".
provider_7      | 💡 Sent share for key: "test".
provider_9      | 💡 Sent share for key: "test".
provider_3      | 💡 Sent share for key: "test".
provider_4      | 💡 Sent share for key: "test".
provider_8      | 💡 Sent share for key: "test".
bootstrapper_1  | 💡 Sent share for key: "test".
provider_3      | 🔄 Refreshed share for key: "test"
provider_7      | 🔄 Refreshed share for key: "test"
provider_6      | 🔄 Refreshed share for key: "test"
bootstrapper_1  | 🔄 Refreshed share for key: "test"
provider_8      | 🔄 Refreshed share for key: "test"
provider_4      | 🔄 Refreshed share for key: "test"
provider_2      | 🔄 Refreshed share for key: "test"
provider_9      | 🔄 Refreshed share for key: "test"
provider_1      | 🔄 Refreshed share for key: "test"
provider_5      | 🔄 Refreshed share for key: "test"
provider_1      | 💡 Sent share for key: "test".
bootstrapper_1  | 💡 Sent share for key: "test".
provider_7      | 💡 Sent share for key: "test".
provider_6      | 💡 Sent share for key: "test".
provider_5      | 💡 Sent share for key: "test".
provider_9      | 💡 Sent share for key: "test".
provider_8      | 💡 Sent share for key: "test".
```

These instructions provide a step-by-step guide for interacting with 'shard' and its various functionalities. Make sure to adjust the IP addresses, ports, and keys as needed for your specific use case.

## shard command-line options

shard provides various functionalities as part of its command-line interface:

### 1. `provide`

Run shard as a share provider. This mode enables the node to store and manage secret shares.

```bash
shard provide [OPTIONS]
```

### 2. `combine`

Combine shares to reconstruct the original secret. This command requires specifying the key associated with the shares and the threshold number.

```bash
shard combine --key <KEY> --threshold <THRESHOLD>
```

### 3. `split`

Split a secret into shares and register them with the network. This command allows specifying the threshold, total number of shares, and the secret to split.

```bash
shard split --threshold <THRESHOLD> --shares <SHARES> --secret <SECRET>
```

### 4. `ls`

List the providers for a specific share. This command helps in identifying all the nodes that hold a share of a particular secret.

```bash
shard ls
```

### 5. `refresh`

Refresh the shares to enhance their security. This command requires the key associated with the shares, the share threshold, and the key size.

```bash
shard refresh --key <KEY> --threshold <THRESHOLD> --size <SIZE>
```

## Design

### Description

In the 'shard' application, network peers function in dual roles: as share providers or as share retrievers. Share providers use `libp2p-kad` to advertise available shares on a Distributed Hash Table (DHT), making them discoverable to others in the network. Conversely, share retrievers use the same system to find and acquire specific shares from any peer in the network.

## Operational Flow

Here's an overview of how 'shard' manages share distribution:

- **Share Providers**: Consider two nodes, A and B, assuming the roles of share providers. Node A holds share FA, while Node B holds share FB. They broadcast their availability as share providers on the DHT via `libp2p-kad`. This step is crucial as it makes their shares discoverable to other network peers.

- **Share Retrievers**: Take Node C as an example of a share retriever. Its goal is to obtain either share FA or FB. It leverages `libp2p-kad` for locating the providers of these shares on the DHT, even without a direct connection to them. Once the provider is identified, Node C initiates a connection and requests the share using `libp2p-request-response`.

- **Role of DHT in Connectivity**: The DHT is a pivotal element in 'shard' for share distribution. It serves as a repository and discovery service for information about share providers. The interconnected nature of nodes through the DHT enhances the efficacy of share discovery and acquisition.

## Architectural Features

'shard' is characterized by several key architectural features:

- **User-Friendly Async/Await Interface**: The application is equipped with a streamlined and replicable async/await interface. This design provides a smooth interaction with the network layer for users. The `Client` module is specifically designed to encompass all necessary network communication functionalities.

- **Optimized Network Management**: 'shard' employs a singular task-driven approach for its network layer. This strategic design choice promotes efficient network communication, circumventing the need for complex synchronization or locking mechanisms.


### libp2p as the under-pinning p2p framework

Using libp2p for building the 'shard' node offers several advantages due to the design and features of libp2p. Here's a detailed explanation of the benefits:

1. **Modularity**: libp2p is designed with modularity in mind. It's composed of a set of pluggable modules which can be mixed and matched according to specific requirements. This means you can choose only the components you need for 'shard', potentially reducing the complexity and increasing the efficiency of your network.

2. **Interoperability**: One of the key strengths of libp2p is its language-agnostic nature, supporting multiple programming languages. This cross-language support facilitates interoperability, allowing 'shard' nodes written in different languages to communicate seamlessly.

3. **Decentralization and Scalability**: libp2p is built for decentralized systems, which aligns well with the architecture of 'shard'. It handles peer discovery and routing in a decentralized manner, making it easier to scale the network and add more nodes without central coordination.

4. **Transport Agnosticism**: The library is transport-agnostic, meaning it can work over various transport protocols like TCP, UDP, WebSockets, etc. This flexibility allows 'shard' to operate in different network environments and conditions, enhancing its adaptability and resilience.

5. **NAT Traversal**: libp2p offers robust NAT traversal capabilities, which is crucial for peer-to-peer networks like 'shard' that may operate in environments with various network constraints.

6. **Security**: libp2p has a strong focus on security. It supports encrypted communications and allows you to integrate various cryptographic protocols. This means that data transfer within the 'shard' can be made secure, which is especially important when handling sensitive operations like secret sharing.

7. **Stream Multiplexing**: The library supports multiplexing multiple data streams over a single connection. This feature can optimize network resources and improve the efficiency of data transmission in 'shard'.

8. **Peer Routing and Discovery**: libp2p simplifies the process of peer discovery and routing. In a network like 'shard', where nodes need to find and connect with each other efficiently, these features of libp2p can significantly enhance performance and reliability.

9. **Customization and Extensibility**: Due to its modular design, libp2p can be easily extended with custom protocols and features. This allows for the creation of tailored solutions that fit the specific requirements of 'shard'.

The choice of libp2p for building 'shard' provides a solid foundation for creating a secure, efficient, and scalable decentralized network, with the flexibility to adapt to various use cases and environments.

## Theory

### Shamir's Secret Sharing (SSS)

Shamir's Secret Sharing is a cryptographic technique developed by Adi Shamir. It enables a secret to be divided into multiple shares, distributed to participants, such that only a specified number of shares (threshold) can reconstruct the original secret. This approach ensures that the secret is secure even if some shares are compromised.

#### Mathematical Foundation

The secret sharing is based on polynomial interpolation in finite fields. The process involves:

1. Choosing a random polynomial of degree `t-1`, where `t` is the threshold.
2. Setting the constant term of the polynomial as the secret.
3. Generating shares as points on the polynomial.

The secret can be reconstructed by interpolating the polynomial using any `t` shares.

### Proactive Secret Sharing

Proactive secret sharing extends the security of SSS by periodically refreshing shares without altering the secret. It mitigates the risk of long-term attacks and ensures the durability of the secret's confidentiality.

#### Mathematical Foundation

1. **Polynomial Construction**:
   The core idea of SSS is based on constructing a polynomial of degree `t-1` (where `t` is the threshold number of shares needed to reconstruct the secret) over a finite field (typically GF(2^8) or GF(256) for 256 distinct values).

   The polynomial is defined as:
   $f(x) = a_0 + a_1 \cdot x + a_2 \cdot x^2 + \cdots + a_{t-1} \cdot x^{t-1} \$

   where:
   - $\( a_0 \)$ is the secret to be shared.
   - $\( a_1, a_2, \ldots, a_{t-1} \)$ are coefficients chosen randomly from the finite field, except for $\( a_0 \)$.

2. **Share Generation**:
   Shares are generated by evaluating this polynomial at different points. Each share is a tuple $\( (x, f(x)) \)$, where $\( x \)$ is a non-zero element of the field, and $\( f(x) \)$ is the polynomial's value at $\( x \)$.

3. **Secret Reconstruction**:
   The secret $\( a_0 \)$ (the constant term of the polynomial) can be reconstructed using any $\( t \)$ shares by interpolating the polynomial using Lagrange interpolation.

   The Lagrange interpolation formula in a finite field is:
   
   $$f(0) = \sum_{j=1}^{t} y_j \cdot \prod_{\substack{1 \leq m \leq t \\ m \neq j}} \frac{x_m}{x_m - x_j} \$$

   where \( (x_j, y_j) \) are the shares used for reconstruction.

### Proactive Secret Refreshing

Proactive secret sharing involves periodically updating the shares without changing the original secret. This is achieved by:

1. **Creating a New Polynomial**:
   A new polynomial is generated a zero term constant (the secret) but with different random coefficients for the other terms.

2. **Refreshing Shares**:
   The existing shares are updated by adding values derived from the new polynomial. The process ensures that the shares remain valid for the original secret but are different from the previous shares.

### Example: Using SSS and Proactive Refresh in shard

1. **Splitting a Secret (SSS)**:
   Suppose you have a secret number `123`, and you want to split it into shares with a threshold of `3`. You would create a polynomial of degree `2` (threshold - 1) with `123` as the constant term. Random coefficients are chosen for the other terms. Shares are then generated by evaluating this polynomial at different points.

2. **Refreshing Shares**:
   After some time, to refresh these shares, you generate a new polynomial with `123` as the constant term again but with new random coefficients for the other terms. The shares are updated by adding values from this new polynomial, thus refreshing them without altering the original secret.
