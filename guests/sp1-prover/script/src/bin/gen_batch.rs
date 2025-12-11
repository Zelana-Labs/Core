use {
    ed25519_dalek::{ed25519::signature::SignerMut, SigningKey},
    std::{collections::HashMap, fs::File, io::Write},
    x25519_dalek::{PublicKey as XPub, StaticSecret},
    zelana_core::{
        prover::{AccountData, BatchInput},
        IdentityKeys, L2Transaction, SignedTransaction, TransactionData,
    },
    zelana_execution::ZkMemStore,
};

fn main() -> anyhow::Result<()> {
    // 1. Setup Identities
    let seed = [7u8; 32]; // Deterministic seed
    let mut sign_key = SigningKey::from_bytes(&seed);
    let enc_key = StaticSecret::from(seed);

    let keys = IdentityKeys {
        signer_pk: sign_key.verifying_key().to_bytes(),
        privacy_pk: *XPub::from(&enc_key).as_bytes(),
    };
    let my_id = keys.derive_id();

    // 2. Create Initial State (Witness)
    let mut witness = HashMap::new();
    witness.insert(
        my_id,
        AccountData {
            balance: 1000,
            nonce: 0,
        },
    );

    // We clone the witness map because ZkMemStore consumes it (or you can clone inside new)
    let store = ZkMemStore::new(witness.clone());
    let pre_root = store.compute_root();
    println!("Calculated Pre-Root: {:?}", pre_root);

    // 3. Create Transactions
    let tx_data = TransactionData {
        from: my_id,
        to: my_id,
        amount: 50,
        nonce: 0,
        chain_id: 1,
    };

    // Sign it
    let msg = wincode::serialize(&tx_data)?;
    let signature = sign_key.try_sign(&msg)?.to_vec();

    let signed = SignedTransaction {
        data: tx_data,
        signature,
        signer_pubkey: keys.signer_pk,
    };

    // 4. Calculate Pre-State Root (Must match what the guest calculates!)
    // We use the same ZkMemStore logic locally to get the expected root
    // (In production, the Sequencer provides this).
    // For this test script, we need to replicate the sorting/hashing logic of ZkMemStore
    // OR just import it if we moved ZkMemStore to a shared lib.
    // For now, let's trust the guest will compute it from the witness map.

    // HACK: To make the Guest pass the "Pre-Root Check", we need to know what the Root IS.
    // The easiest way is to let the Guest run once, panic with "Expected X got Y", and copy Y.
    // OR, we move `simple_store` to `zelana-execution` so we can use it here.

    let input = BatchInput {
        pre_state_root: pre_root,
        transactions: vec![L2Transaction::Transfer(signed)],
        witness_accounts: witness,
    };

    // 5. Save
    let bytes = wincode::serialize(&input)
        .map_err(|e| anyhow::anyhow!("wincode serialize error: {}", e))?;

    let mut file = File::create("batch.bin")?;
    file.write_all(&bytes)?;

    println!("Generated batch.json with 1 tx. (Pre-Root is invalid, expect panic on first run!)");
    Ok(())
}
