# Zelana SDK

**The official client library for building wallets, trading bots, and applications on the Zelana L2 Rollup.**

`zelana-sdk` provides a high-level, async Rust interface for interacting with the Zelana protocol. It abstracts away the low-level cryptographic handshakes (`zelana-net`) and serialization details (`zelana-core`), allowing developers to focus on application logic.

## Features

* **Dual-Key Wallet Management:** Automatically manages the **Signer Key** (Ed25519) for authorization and the **Privacy Key** (X25519) for encryption.
* **Auto-Handshake:** Performs the Diffie-Hellman Key Exchange with the Sequencer transparently upon connection.
* **Fire-and-Forget Networking:** Uses the **Zephyr Protocol** (Encrypted UDP) for ultra-low latency transaction submission.
* **Type-Safe:** Re-exports all core protocol types (`L2Transaction`, `AccountId`) to ensure your application is always compatible with the node.

## Installation

Add this to your `Cargo.toml`. If you are in the workspace, use the path; otherwise, point to the git repo.

```toml
[dependencies]
zelana-sdk = { path = "../zelana-sdk" }
tokio = { version = "1.48.0" , features = ["full"]}
```

## Usage Guide

### 1. Creating a Wallet (Identity)

The `ZelanaWallet` manages your keys. It can generate fresh keys or recover from a seed.

```rust
use zelana_sdk::ZelanaWallet;

// Generate a random wallet
let wallet = ZelanaWallet::new_random();

// Get your L2 Account ID (Safe to share)
let my_id = wallet.account_id();
println!("My L2 Address: {}", my_id.to_hex());

// Get public keys (for sharing with others to receive encrypted data)
let keys = wallet.public_keys();
```

### 2. Connecting to the Sequencer

The `ZelanaClient` handles the network connection. When you call connect, it performs the cryptographic handshake immediately.

```rust
use zelana_sdk::ZelanaClient;

// Connect to a local or remote sequencer
// This awaits the ServerHello and derives session keys
let mut client = ZelanaClient::connect("127.0.0.1:9000").await?;
```

### 3. Sending a Transaction

Sign a transaction with your wallet, then broadcast it with the client.

```rust
use zelana_sdk::{TransactionData, AccountId};

// 1. Define the transaction intent
let tx_data = TransactionData {
    from: wallet.account_id(),
    to: AccountId([0u8; 32]), // Recipient ID
    amount: 1000,
    nonce: 1,      // Must increment per tx
    chain_id: 1,   // 1 = Mainnet, 2 = Devnet
};

// 2. Sign it (Attaches your Public Key for ZK verification)
let signed_tx = wallet.sign_transaction(tx_data);

// 3. Send it (Encrypts -> UDP Broadcast)
client.send_transaction(signed_tx).await?;
```

## Architecture

This SDK is a wrapper around two lower-level crates:

* **zelana-core:** Provides the data structures (`SignedTransaction`, `IdentityKeys`) and serialization logic.
* **zelana-net:** Provides the `SessionKeys` and UDP frame parsing for the Zephyr protocol.

## Examples

Run the included demo to see the full flow in action:

```bash
# Ensure the sequencer is running first!
cargo run -p zelana-sequencer

# In a separate terminal:
cargo run -p zelana-sdk --example demo
```
