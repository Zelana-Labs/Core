## Zelana v2.0 Architecture Deep Dive

This document details the engineering decisions, performance physics, and trust models behind Zelana v2.0.

---

## The Physics of 50ms Latency

Zelana claims "sub-50ms transaction latency." This refers to **Soft Confirmation**—the time from a user sending a transaction to receiving a cryptographic acknowledgment from the Sequencer.

Here is the latency budget breakdown compared to a standard Rollup:

| Step | Standard Rollup (HTTP/TCP) | Zelana (Zephyr UDP) | Savings |
| :--- | :--- | :--- | :--- |
| **Transport** | TCP Handshake (SYN/ACK) + TLS + HTTP Headers (3-4 RTT) | Persistent UDP Session (1 RTT) | **~100ms** |
| **Protocol** | JSON-RPC Parsing (Verbose, CPU heavy) | Bincode Structs (Compact, Zero-copy) | **~5ms** |
| **Encryption** | TLS (AES-GCM/RSA) | ChaCha20-Poly1305 (Stream Cipher) | **~1ms** |
| **Execution** | SVM | Native Rust + RocksDB | **< 1ms** |

**The Math:**
* **Network RTT:** ~20ms (Regional)
* **Decrypt & Parse:** < 1ms
* **Execute & Commit:** < 1ms
* **Total Round Trip:** **~22ms**

This architecture eliminates Head-of-Line (HoL) blocking, making it viable for users as well as HFTS etc,.

---

## The Hybrid Trust Model

Zelana implements a "Dual-Lane" architecture to serve two distinct user groups without compromise.

### Lane 1: The Fast Exit (Attested)
* **For:** Market Makers, Arbitrageurs.
* **Mechanism:** The Sequencer instantly signs a `withdraw_attested` transaction on L1.
* **Speed:** Instant (Limited only by Solana block time).
* **Trust:** Relies on the Sequencer's authority key (Optimistic).

### Lane 2: The Trustless Rollup (Proven)
* **For:** Retail Users, Long-term Holders.
* **Mechanism:** Transactions are batched, executed, and the State Root is updated on L1 via a **ZK Proof**.
* **Speed:** Batched (e.g., every 10s or 100 txs).
* **Trust:** **Trustless**. Security is guaranteed by the SP1 ZK Circuit and Solana L1. Users can verify their funds exist in the State Root even if the Sequencer disappears.

---

## Component Stack

### 1. Zelana Net (The Wire)
A custom application-layer protocol built on UDP.
* **Handshake:** X25519 Diffie-Hellman (Perfect Forward Secrecy).
* **Framing:** 1-byte Header + 12-byte Nonce + Encrypted Payload.
* **Replay Protection:** XOR-based Nonce counters (WireGuard style).

### 2. Zelana Core (The Logic)
The shared "Source of Truth" library.
* **Identity:** `AccountId = SHA256(SignerPK || PrivacyPK)`.
* **Serialization:** Canonical `bincode` ensures the Rust Host and RISC-V Guest produce identical binary layouts.

### 3. SP1 Prover (The Judge)
A Zero-Knowledge implementation of the Execution Engine.
* **Input:** `PreStateRoot` + `Batch`.
* **Process:** Re-executes the exact same Rust code (`zelana-execution`) used by the Sequencer.
* **Output:** `PostStateRoot` + `Groth16 Proof`.

### 4. Solana Bridge (The Settlement)
The L1 Smart Contract.
* **Vault:** Holds 100% of the collateral (TVL).
* **Registry:** Stores the current valid Merkle Root of L2.
* **Verifier:** Verifies SP1 proofs to allow State Root updates.