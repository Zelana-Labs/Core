# Zelana Sequencer

**The high-performance L2 Node and Execution Engine for the Zelana Rollup.**

The `zelana-sequencer` is the "Brain" of the Zelana protocol. Unlike standard rollups that rely on slow HTTP/TCP JSON-RPC, this sequencer implements a custom **UDP-based Event Loop** (via the **Zephyr Protocol**) to process thousands of transactions per second with sub-millisecond network latency.

## Architecture

The service is composed of three main stages:

### 1. Ingress Layer (UDP)

* **Listener:** Binds to `0.0.0.0:9000` (default) and accepts raw UDP frames.
* **Zero-Allocation Parsing:** Uses `zelana-net` to identify packet types (`ClientHello` vs `AppData`) without unnecessary memory allocation.
* **Session Management:** Maintains a thread-safe `DashMap` of active clients and their ephemeral session keys.

### 2. Cryptographic Layer

* **Handshake:** Performs **X25519** Diffie-Hellman key exchange for every new connection to establish Perfect Forward Secrecy (PFS).
* **Decryption:** Decrypts incoming `AppData` payloads using **ChaCha20-Poly1305**.
* **Replay Protection:** Enforces strict nonce ordering to prevent replay attacks.

### 3. Execution Layer

* **Authentication:** Verifies **Ed25519** signatures against the user's `AccountId` (Double-Key Identity).
* **Ordering:** Sequences valid transactions into a deterministic order.
* **Execution:** (Phase 2) Applies state transitions to the **SVM (Solana Virtual Machine)** and persists changes to **RocksDB**.

## Getting Started

### Prerequisites

* **Rust:** Stable toolchain (1.75+)
* **Clang/LLVM:** Required for RocksDB compilation (if enabled).

### Running the Node

By default, the sequencer runs in "Dev Mode" on localhost.

```bash
# Run with info logs to see startup status
RUST_LOG=info cargo run -p zelana-sequencer --release
```

### Expected Output

```
INFO  > Zelana Sequencer v2.0 Starting...
INFO  > Listening on UDP 0.0.0.0:9000
```

## Configuration

Currently, configuration is static for the Alpha release. Future versions will support `config.toml` or `.env` overrides.

| Setting | Default   | Description                                   |
| ------- | --------- | --------------------------------------------- |
| Port    | `9000`    | The UDP port for high-frequency transactions. |
| Host    | `0.0.0.0` | Binds to all network interfaces.              |
| MTU     | `1500`    | Maximum Transmission Unit for UDP frames.     |

## Integration

This service relies on:

* **zelana-core:** For shared transaction types (`L2Transaction`).
* **zelana-net:** For wire protocol and encryption primitives.

It is designed to be consumed by:

* **zelana-sdk:** Clients connect to this sequencer using the SDK.
* **zelana-prover:** (Future) The sequencer will stream execution traces to the prover for ZK proof generation.

## Testing

To verify the sequencer is working, run the end-to-end integration test which spins up a temporary server and connects a client.

```bash
cargo test -p zelana-sequencer --test network_integration
```
