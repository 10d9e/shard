# MPCNet 🔑

[![build-badge](https://github.com/jlogelin/mpcnet/actions/workflows/build.yml/badge.svg)](https://nightly.link/jlogelin/mpcnet/workflows/build/main/binaries.zip)
[![document-badge](https://github.com/jlogelin/mpcnet/actions/workflows/doc.yml/badge.svg)](https://jlogelin.github.io/mpcnet)
[![license-badge](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

MPCNet is a decentralized application built to facilitate secure and robust threshold networking using Shamir's Secret Sharing (SSS) and proactive secret refreshing. It allows users to split secrets into shares, distribute them across a network, combine them to reconstruct the original secret, and refresh these shares to maintain security over time.

## Getting Started

To use MPCNet, follow these steps:

1. **Installation**: Compile and install MPCNet from the source code.
2. **Running a Node**: Start an MPCNet node using the `provide` command.
3. **Splitting a Secret**: Use the `split` command to divide a secret into shares.
4. **Combining Shares**: Reconstruct the secret using the `combine` command with the required number of shares.
5. **Refreshing Shares**: Regularly refresh shares using the `refresh` command to maintain security.

### Example 5 node network
Make sure you have the latest version of Rust installed. To build the node, open your terminal and navigate to the project directory where 'mpcnet' is located, and then run the following command:

**1. Build the Node**

```bash
cargo build --release
```

This command compiles the 'mpcnet' project in release mode, ensuring optimal performance.

**2. Start the Bootstrapper Node**

The bootstrapper node is essential for the network to operate. It acts as the initial point of contact for other nodes. To start the bootstrapper node, use the following command:

```bash
./target/release/mpcnet --listen-address /ip4/127.0.0.1/tcp/40837 --secret-key-seed 1 provide
```

This command starts 'mpcnet' as a bootstrapper node, listening on address '/ip4/127.0.0.1/tcp/40837' and using the secret key seed '1'.

**3. Start Provider Nodes**

You'll need multiple provider nodes to participate in the network. (To run locally we will just use an in-memory database, however, in a real world scenario, we would use the `db-path` option to define the path for share persistence.) To start a provider node, open multiple terminals (as many as needed) and run the following command in each terminal:

```bash
./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X provide
```

Each command starts a provider node, and they all connect to the bootstrapper node specified by the `--peer` argument.

**4. Start a Client Node**

To interact with the network, you'll need a client node. Open another terminal and run the following command:

```bash
❯ ./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X split --threshold 2 --shares 5 --secret butterbeer --key test --debug

🐛 [debug] shares: 
  6720c971ea73b326ea88
  61461f77eb862dafeb24
  6413a2726487fcec64de
  6eb9c5786785436a6737
  6d8aae7be9710ca0e961
✂️  Secret has been split and distributed across network.
    key: "test"
    threshold: 2
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

This command splits a secret into 5 shares with a threshold of 2 and broadcasts them to the network. You can optionally view the shares using the `--debug` flag.

**5. Query the Network for Shares**

To query the network for all nodes hosting shares for a specific key (in this case, 'test'), use the following command:

```bash
❯ ./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X ls --key test
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

The refresh command can be run as many times as you want, and it doesn't require client interaction. Use the following command to refresh shares:

```bash
❯ ./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X refresh --key test --threshold 2 --size 10
🔄 Refreshed 5 shares for key: "test"
```

This command refreshes the shares for the 'test' key with a threshold of 2 and a size of 10.

**7. Reassemble the Secret**

To reassemble the secret, you'll need to randomly select 2 providers corresponding to the threshold. Use the following command:

```bash
❯ ./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X combine --key test --threshold 2 --debug
🐛 [debug] shares: 
  9f236113dd50807ae895
  c2478ca20d6c3c9b1e2f
🔑 secret: "butterbeer"
```

This command combines the shares from the selected providers to reassemble the secret. You can use the `--debug` flag to view share bytes during the process.

Each node logs output as it interacts with the network:

```bash
🚀 Registered share for key: "test".
🔄 Refreshed share for key: "test"
🔄 Refreshed share for key: "test"
🔄 Refreshed share for key: "test"
🔄 Refreshed share for key: "test"
💡 Sent share for key: "test".
```

These instructions provide a step-by-step guide for interacting with 'mpcnet' and its various functionalities. Make sure to adjust the IP addresses, ports, and keys as needed for your specific use case.

## MPCNet command-line options

MPCNet provides various functionalities as part of its command-line interface:

### 1. `provide`

Run MPCNet as a share provider. This mode enables the node to store and manage secret shares.

```bash
mpcnet provide [OPTIONS]
```

### 2. `combine`

Combine shares to reconstruct the original secret. This command requires specifying the key associated with the shares and the threshold number.

```bash
mpcnet combine --key <KEY> --threshold <THRESHOLD>
```

### 3. `split`

Split a secret into shares and register them with the network. This command allows specifying the threshold, total number of shares, and the secret to split.

```bash
mpcnet split --threshold <THRESHOLD> --shares <SHARES> --secret <SECRET>
```

### 4. `ls`

List the providers for a specific share. This command helps in identifying all the nodes that hold a share of a particular secret.

```bash
mpcnet ls
```

### 5. `refresh`

Refresh the shares to enhance their security. This command requires the key associated with the shares, the share threshold, and the key size.

```bash
mpcnet refresh --key <KEY> --threshold <THRESHOLD> --size <SIZE>
```

#### Docker

A docker-based testnet is provided that includes 10 nodes and a client container for testing commands.

### 1. Start the mpcnet test network

```bash
docker-compose up --build
```

### 2. Execute the commands within the testnet

#### From the host operating system

```bash
docker exec -it client mpcnet --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X split --threshold 7 --shares 10 --secret butterbeer --key test
docker exec -it client mpcnet --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X ls --key test
docker exec -it client mpcnet --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X refresh --key test --threshold 7 --size 10
docker exec -it client mpcnet --peer /ip4/10.5.0.5/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X combine --key test --threshold 7

```

#### Interactively within the `client` container

```bash
❯ docker exec -it client /bin/bash 
root@71b4dce52922:/app# mpcnet --help
Usage: mpcnet [OPTIONS] <COMMAND>

Commands:
  provide  Run as a share provider
  combine  Combine shares to rebuild the secret
  split    Split a secret into shares and register them with the network
  ls       Get the list of providers for a share
  refresh  Refresh the shares
  help     Print this message or the help of the given subcommand(s)

Options:
  -s, --secret-key-seed <SECRET_KEY_SEED>  Fixed value to generate deterministic peer ID
  -p, --peer <PEER>                        Address of a peer to connect to
  -l, --listen-address <LISTEN_ADDRESS>    Address to listen on
  -h, --help                               Print help
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

## Design

### Description

In the 'mpcnet' application, network peers function in dual roles: as share providers or as share retrievers. Share providers use `libp2p-kad` to advertise available shares on a Distributed Hash Table (DHT), making them discoverable to others in the network. Conversely, share retrievers use the same system to find and acquire specific shares from any peer in the network.

## Operational Flow

Here's an overview of how 'mpcnet' manages share distribution:

- **Share Providers**: Consider two nodes, A and B, assuming the roles of share providers. Node A holds share FA, while Node B holds share FB. They broadcast their availability as share providers on the DHT via `libp2p-kad`. This step is crucial as it makes their shares discoverable to other network peers.

- **Share Retrievers**: Take Node C as an example of a share retriever. Its goal is to obtain either share FA or FB. It leverages `libp2p-kad` for locating the providers of these shares on the DHT, even without a direct connection to them. Once the provider is identified, Node C initiates a connection and requests the share using `libp2p-request-response`.

- **Role of DHT in Connectivity**: The DHT is a pivotal element in 'mpcnet' for share distribution. It serves as a repository and discovery service for information about share providers. The interconnected nature of nodes through the DHT enhances the efficacy of share discovery and acquisition.

## Architectural Features

'mpcnet' is characterized by several key architectural features:

- **User-Friendly Async/Await Interface**: The application is equipped with a streamlined and replicable async/await interface. This design provides a smooth interaction with the network layer for users. The `Client` module is specifically designed to encompass all necessary network communication functionalities.

- **Optimized Network Management**: 'mpcnet' employs a singular task-driven approach for its network layer. This strategic design choice promotes efficient network communication, circumventing the need for complex synchronization or locking mechanisms.


### libp2p as the under-pinning p2p framework

Using libp2p for building the 'mpcnet' node offers several advantages due to the design and features of libp2p. Here's a detailed explanation of the benefits:

1. **Modularity**: libp2p is designed with modularity in mind. It's composed of a set of pluggable modules which can be mixed and matched according to specific requirements. This means you can choose only the components you need for 'mpcnet', potentially reducing the complexity and increasing the efficiency of your network.

2. **Interoperability**: One of the key strengths of libp2p is its language-agnostic nature, supporting multiple programming languages. This cross-language support facilitates interoperability, allowing 'mpcnet' nodes written in different languages to communicate seamlessly.

3. **Decentralization and Scalability**: libp2p is built for decentralized systems, which aligns well with the architecture of 'mpcnet'. It handles peer discovery and routing in a decentralized manner, making it easier to scale the network and add more nodes without central coordination.

4. **Transport Agnosticism**: The library is transport-agnostic, meaning it can work over various transport protocols like TCP, UDP, WebSockets, etc. This flexibility allows 'mpcnet' to operate in different network environments and conditions, enhancing its adaptability and resilience.

5. **NAT Traversal**: libp2p offers robust NAT traversal capabilities, which is crucial for peer-to-peer networks like 'mpcnet' that may operate in environments with various network constraints.

6. **Security**: libp2p has a strong focus on security. It supports encrypted communications and allows you to integrate various cryptographic protocols. This means that data transfer within the 'mpcnet' can be made secure, which is especially important when handling sensitive operations like secret sharing.

7. **Stream Multiplexing**: The library supports multiplexing multiple data streams over a single connection. This feature can optimize network resources and improve the efficiency of data transmission in 'mpcnet'.

8. **Peer Routing and Discovery**: libp2p simplifies the process of peer discovery and routing. In a network like 'mpcnet', where nodes need to find and connect with each other efficiently, these features of libp2p can significantly enhance performance and reliability.

9. **Customization and Extensibility**: Due to its modular design, libp2p can be easily extended with custom protocols and features. This allows for the creation of tailored solutions that fit the specific requirements of 'mpcnet'.

The choice of libp2p for building 'mpcnet' provides a solid foundation for creating a secure, efficient, and scalable decentralized network, with the flexibility to adapt to various use cases and environments.

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

### Example: Using SSS and Proactive Refresh in MPCNet

1. **Splitting a Secret (SSS)**:
   Suppose you have a secret number `123`, and you want to split it into shares with a threshold of `3`. You would create a polynomial of degree `2` (threshold - 1) with `123` as the constant term. Random coefficients are chosen for the other terms. Shares are then generated by evaluating this polynomial at different points.

2. **Refreshing Shares**:
   After some time, to refresh these shares, you generate a new polynomial with `123` as the constant term again but with new random coefficients for the other terms. The shares are updated by adding values from this new polynomial, thus refreshing them without altering the original secret.
