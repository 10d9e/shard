# MPCNet: Decentralized Threshold Network using Shamir's Secret Sharing

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

You'll need multiple provider nodes to participate in the network. To start a provider node, open multiple terminals (as many as needed) and run the following command in each terminal:

```bash
./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X provide
```

Each command starts a provider node, and they all connect to the bootstrapper node specified by the `--peer` argument.

**4. Start a Client Node**

To interact with the network, you'll need a client node. Open another terminal and run the following command:

```bash
./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X split --threshold 2 --shares 5 --secret butterbeer --key test --debug
```

This command splits a secret into 5 shares with a threshold of 2 and broadcasts them to the network. You can optionally view the shares using the `--debug` flag.

**5. Query the Network for Shares**

To query the network for all nodes hosting shares for a specific key (in this case, 'test'), use the following command:

```bash
./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X ls --key test
```

This command retrieves information about nodes hosting shares for the specified key.

**6. Refresh Shares**

The refresh command can be run as many times as you want, and it doesn't require client interaction. Use the following command to refresh shares:

```bash
./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X refresh --key test --threshold 2 --size 10
```

This command refreshes the shares for the 'test' key with a threshold of 2 and a size of 10.

**7. Reassemble the Secret**

To reassemble the secret, you'll need to randomly select 2 providers corresponding to the threshold. Use the following command:

```bash
./target/release/mpcnet --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X combine --key test --threshold 2 --debug
```

This command combines the shares from the selected providers to reassemble the secret. You can use the `--debug` flag to view additional information during the process.

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
   \[ f(x) = a_0 + a_1 \cdot x + a_2 \cdot x^2 + \cdots + a_{t-1} \cdot x^{t-1} \]

   where:
   - \( a_0 \) is the secret to be shared.
   - \( a_1, a_2, \ldots, a_{t-1} \) are coefficients chosen randomly from the finite field, except for \( a_0 \).

2. **Share Generation**:
   Shares are generated by evaluating this polynomial at different points. Each share is a tuple \( (x, f(x)) \), where \( x \) is a non-zero element of the field, and \( f(x) \) is the polynomial's value at \( x \).

3. **Secret Reconstruction**:
   The secret \( a_0 \) (the constant term of the polynomial) can be reconstructed using any \( t \) shares by interpolating the polynomial using Lagrange interpolation.

   The Lagrange interpolation formula in a finite field is:
   \[ f(0) = \sum_{j=1}^{t} y_j \cdot \prod_{\substack{1 \leq m \leq t \\ m \neq j}} \frac{x_m}{x_m - x_j} \]

   where \( (x_j, y_j) \) are the shares used for reconstruction.

### Proactive Secret Refreshing

Proactive secret sharing involves periodically updating the shares without changing the original secret. This is achieved by:

1. **Creating a New Polynomial**:
   A new polynomial is generated with the same constant term (the secret) but with different random coefficients for the other terms.

2. **Refreshing Shares**:
   The existing shares are updated by adding values derived from the new polynomial. The process ensures that the shares remain valid for the original secret but are different from the previous shares.

### Example: Using SSS and Proactive Refresh in MPCNet

1. **Splitting a Secret (SSS)**:
   Suppose you have a secret number `123`, and you want to split it into shares with a threshold of `3`. You would create a polynomial of degree `2` (threshold - 1) with `123` as the constant term. Random coefficients are chosen for the other terms. Shares are then generated by evaluating this polynomial at different points.

2. **Refreshing Shares**:
   After some time, to refresh these shares, you generate a new polynomial with `123` as the constant term again but with new random coefficients for the other terms. The shares are updated by adding values from this new polynomial, thus refreshing them without altering the original secret.
