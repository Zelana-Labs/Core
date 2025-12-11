use {
    crate::{AccountId, L2Transaction},
    std::collections::HashMap,
    wincode_derive::{SchemaRead, SchemaWrite},
};

/// The input fed into the SP1 ZKVM.
#[derive(SchemaRead, SchemaWrite, Debug, Clone)]
pub struct BatchInput {
    /// The Merkle Root of the state BEFORE this batch.
    pub pre_state_root: [u8; 32],

    /// The ordered list of transactions to execute.
    pub transactions: Vec<L2Transaction>,

    /// The subset of the State Database needed to execute these transactions.
    /// future->Merkle Proofs. For MVP, it's the raw accounts).
    pub witness_accounts: HashMap<AccountId, AccountData>,
}

/// Helper struct for passing account state across the boundary.
#[derive(SchemaRead, SchemaWrite, Debug, Clone)]
pub struct AccountData {
    pub balance: u64,
    pub nonce: u64,
}
