# SP1 Prover (Guest)

**The Zero-Knowledge Circuit for the Zelana Rollup.**

This crate contains the **Guest Program**—the actual Rust logic that runs inside the [SP1 zkVM](https://github.com/succinctlabs/sp1). It acts as the cryptographic "Judge" of the protocol, proving that the Sequencer executed a batch of transactions correctly without modifying the software.

## Concept

* **The Sequencer** runs the Execution Engine on x86/ARM hardware (Fast).
* **The Prover** runs the *exact same* Execution Engine code inside the RISC-V zkVM (Verifiable).

If the output State Root matches between both, the proof is valid.

## Role in Architecture

1. **Input:** The Guest receives a `BatchInput` struct containing:

    * `pre_state_root`: The Merkle Root before execution.
    * `transactions`: A list of `L2Transaction`s.

2. **Logic:**

    * Verifies `Ed25519` signatures for every transaction (checking `signer_pubkey`).
    * Re-executes the transaction logic (Balance checks, Transfers).
    * Updates the internal Merkle Trie representation.

3. **Output:**

    * Commits the `post_state_root` to the public journal.

## Build & Operations

### Prerequisites

* **Rust:** Stable toolchain
* **SP1 CLI:**

```bash
curl -L https://sp1.succinct.xyz | bash
```

### Compilation

Compile the Rust code into a RISC-V ELF binary designed for the zkVM:

```bash
cargo prove build
```

This generates the binary in:

```
elf/riscv32im-succinct-zkvm-elf
```

### Generating Verification Key (GenVK)

To verify proofs on Solana L1, we need the unique hash (VKey) of this program.

```bash
# From the workspace root
cargo run -p sp1-prover --bin gen_vkey
```

## Testing (Mock Prover)

You can run the proving logic locally without generating a heavy ZK proof (fast mode):

```bash
RUST_LOG=info cargo run --release -- --execute
```
