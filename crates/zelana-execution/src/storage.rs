use {
    anyhow::Result,
    wincode::{SchemaRead, SchemaWrite},
    zelana_core::AccountId,
};

/// minimal state of user in l2
#[derive(Debug, Clone, Default, PartialEq, SchemaRead, SchemaWrite)]
pub struct AccountState {
    pub balance: u64,
    pub nonce: u64,
}

/// decoupling logic from the db
pub trait StateStore {
    /// Retrieve an account. Returns Default if not found.
    fn get_account(&self, id: &AccountId) -> Result<AccountState>;

    /// Update an account's state.
    fn set_account(&mut self, id: AccountId, state: AccountState) -> Result<()>;
}
