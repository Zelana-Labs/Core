use log::info;
use zelana_core::SignedTransaction;

pub struct TransactionExecutor {
    //RocksDB handle here
}

impl TransactionExecutor {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn process(&self, tx: SignedTransaction) -> anyhow::Result<()> {
        // SVM Execution
        // 1. Load Account
        // 2. Check Balance
        // 3. Update State

        info!(
            "EXECUTE: {} -> {} | Amount: {} | Nonce: {}",
            tx.data.from.to_hex(),
            tx.data.to.to_hex(),
            tx.data.amount,
            tx.data.nonce
        );

        Ok(())
    }
}
