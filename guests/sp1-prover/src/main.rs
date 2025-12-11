#![no_main]
sp1_zkvm::entrypoint!(main);

use {
    zelana_core::prover::BatchInput,
    zelana_execution::{BatchExecutor, ZkMemStore},
};

mod simple_store;

pub fn main() {
    //Read Input (The batch from the Sequencer)

    let raw: Vec<u8> = sp1_zkvm::io::read();
    let input: BatchInput = wincode::deserialize(&raw).expect("failed to deserialize batchinput");

    //rebuild the state map from the witness data provided
    let mut store = ZkMemStore::new(input.witness_accounts);

    //Verify Pre-State Root
    //"Does the witness data match the Root we claimed we started with?"
    let calculated_pre_root = store.compute_root();

    if calculated_pre_root != input.pre_state_root {
        println!("Guest Calc Root: {:?}", calculated_pre_root);
        println!("Input Pre Root:  {:?}", input.pre_state_root);
        panic!("Fraud Detected: Witness data does not match Pre-State Root!");
    }
    //Execution Loop
    //We run the EXACT SAME logic as the Sequencer
    let mut executor = BatchExecutor::new(&mut store);

    for tx in input.transactions {
        // panic if execution fails. In a ZK Rollup, a "Batch" must contain
        // only valid transactions. Invalid ones should be dropped by Sequencer.
        executor
            .execute(&tx)
            .expect("Transaction Execution Failed inside ZK");
    }

    //Compute Post-State Root
    let new_root = store.compute_root();

    //Commit the Result
    //The proof now publicly asserts: "Given Pre-Root X, running these Txs results in Post-Root Y"
    sp1_zkvm::io::commit(&new_root);
}
