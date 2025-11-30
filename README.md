# Zelana v2.0

**The High-Frequency, Privacy-Enabled L2 Rollup on Solana.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![SP1](https://img.shields.io/badge/zkVM-SP1-blue)](https://succinct.xyz)

Zelana is a general-purpose ZK Rollup. It replaces the slow HTTP/TCP standards of traditional rollups with a custom **Encrypted UDP Protocol (Zephyr)**, enabling sub-50ms transaction latency while maintaining cryptographic integrity via **SP1 Zero-Knowledge Proofs**.

---

## Workspace Architecture

This repository is a **Rust Monorepo** managed via Cargo Workspaces. It contains the entire L2 stack, from the client SDK to the ZK Prover.

### Libraries (The Foundation)

| Crate             | Path                 | Description                                                                                                                 |
| ----------------- | -------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **`zelana-core`** | `crates/zelana-core` | **Shared Types:** Defines the binary format for `L2Transaction`, `AccountId`, and crypto primitives. The "Source of Truth." |
| **`zelana-net`**  | `crates/zelana-net`  | **Wire Protocol:** Implements the **Zephyr Protocol**—an encrypted, fire-and-forget UDP layer with X25519 handshakes.       |
| **`zelana-sdk`**  | `crates/zelana-sdk`  | **Client Kit:** The developer library for building wallets. Handles signing and connection pooling.                |

### Services (The Runtime)

| Service          | Path                 | Description                                                                                                                |
| ---------------- | -------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| **`sequencer`**  | `services/sequencer` | **The Node:** Listens on UDP port 9000, orders transactions, executes SVM logic, and persists state to RocksDB.            |
| **`sp1-prover`** | `guests/sp1-prover`  | **The Judge:** A Rust program that runs inside the RISC-V zkVM to cryptographically prove the Sequencer's execution trace. |

---

## Quick Start

Follow these steps to spin up a local Zelana L2 node and transact against it.

### 1. Prerequisites

* **Rust:** Install from [https://rustup.rs/](https://rustup.rs/)
* **SP1 Toolchain:** (Required only for ZK proving)

```bash
curl -L https://sp1.succinct.xyz | bash
sp1up
```

### 2. Build the Stack

Compiles all services and libraries in release mode.

```bash
cargo build --release
```

### 3. Run the Sequencer (Terminal 1)

This starts the L2 Node on `0.0.0.0:9000` (UDP).

```bash
RUST_LOG=info cargo run -p zelana-sequencer --release
```

You should see:

```
Zelana Sequencer v2.0 Starting...
```

### 4. Run the Demo Client (Terminal 2)

Runs a script that generates a wallet, connects to the node, and sends 5 transactions.

```bash
RUST_LOG=info cargo run -p zelana-sdk --example demo
```

You should see:

```
CLIENT: Secure Session Established!
```

Followed by transaction logs.

---

## Testing & Verification

We enforce correctness at multiple layers.

### Unit Tests (Cryptography)

Verify X25519 handshake + ChaCha20 encryption.

```bash
cargo test -p zelana-net
```

### Integration Tests (Networking)

Spin up a temporary sequencer and test a full client-server cycle.

```bash
cargo test -p zelana-sequencer --test network_integration
```

### Benchmarks (Throughput)

Measure raw TPS on your hardware.

```bash
cargo run -p zelana-sdk --example bench_throughput --release
```

---

## How It Works

1. **Identity:** User generates a Dual-Key Wallet (Signer + Privacy).
2. **Transport:** User sends a `ClientHello` UDP packet.
3. **Handshake:** Sequencer responds; both derive ephemeral session keys.
4. **Transaction:** User signs a Transfer, encrypts it, sends via UDP.
5. **Execution:** Sequencer decrypts, verifies, applies state in RocksDB.
6. **Settlement:** Sequencer batches transactions → SP1 Prover → L1 proof.

---

## Contributing

**Format:**

```bash
cargo fmt
```

**Lint:**

```bash
cargo clippy --workspace
```

**Test:**

```bash
cargo test --workspace
```

---

## Repository Structure

We use a Flat Monorepo structure:

* Shared logic → `crates/`
* Executable binaries → `services/` and `guests/`
