# Zelana Core

**The shared type definitions, cryptographic primitives, and identity logic for the Zelana L2 Protocol.**

`zelana-core` is the foundational library for the entire Zelana stack. It acts as the "Source of Truth" for data structures, ensuring that the **Client SDK**, **Sequencer**, and **ZK Prover** all speak the exact same language.

## Role & Responsibilities

This crate is a pure library (no network I/O, minimal dependencies) designed to be imported by every other component in the workspace.

* **Identity Management:** Defines the **Dual-Key Identity** system (Signer + Privacy keys) and deterministic `AccountId` derivation.
* **Transaction Types:** Defines the binary structure of `L2Transaction`, `SignedTransaction`, and `DepositEvent`.
* **Serialization:** Enforces `bincode` configuration for consistent serialization across Rust (Sequencer) and ZK (SP1) environments.
* **Cryptography:** Centralizes hash functions (BLAKE3) and signature verification (Ed25519) to ensure protocol-wide consistency.

## Key Components

### 1. Account Identity

Zelana uses a **Dual-Key Account Abstraction** to enable future privacy features (like encrypted mempools) without breaking the address format.

* **Signer Key (Ed25519):** Used to authorize transactions (signatures).
* **Privacy Key (X25519):** Used to encrypt transaction data between the User and Sequencer.
* **Account ID:** A 32-byte hash derived from both keys.

  * `AccountId = SHA256(SignerPK || PrivacyPK)`

### 2. Transaction Model

The protocol defines a unified enum for all state transitions.

```rust
pub enum L2Transaction {
    /// A user-initiated transfer sent via the high-speed UDP layer.
    Transfer(SignedTransaction),
    
    /// A deposit event bridged from Solana L1 (trusts the Bridge program).
    Deposit(DepositEvent),
    
    /// A withdrawal request to move funds back to L1.
    Withdraw(WithdrawRequest),
}
```

### 3. ZK-Friendly Design

All structures in this crate are designed to be compatible with the SP1 zkVM.

We explicitly include the signer_pubkey in SignedTransaction to allow the ZK circuit to verify signatures without needing a complex reverse-hash lookup.

We use bincode for serialization because it is compact and efficient inside the zkVM.

## 🛠 Usage

This crate is a dependency for:

* **zelana-sdk:** To generate wallets and sign transactions.
* **zelana-sequencer:** To decode and execute transactions.
* **guests/sp1-prover:** To re-execute transaction logic inside the proof.

## Installation

In your Cargo.toml (if using inside the workspace):

```toml
[dependencies]
zelana-core = { path = "../zelana-core" }
```

## 🧪 Testing

This crate includes unit tests for:

* **Deterministic ID Derivation:** Ensuring an account ID generated today matches one generated in the future.
* **Serialization Stability:** Ensuring binary formats don't drift.

Run tests with:

```bash
cargo test -p zelana-core
```
