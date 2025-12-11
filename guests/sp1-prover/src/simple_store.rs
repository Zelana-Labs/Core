use {
    anyhow::Result,
    blake3::Hasher,
    std::collections::HashMap,
    zelana_core::{AccountData, AccountId},
    zelana_execution::{AccountState, StateStore},
};

/// A lightweight, verifiable state store for the ZKVM.
pub struct ZkMemStore {
    accounts: HashMap<AccountId, AccountState>,
}

impl ZkMemStore {
    /// Initialize from the witness data provided by the Sequencer.
    pub fn new(witness: HashMap<AccountId, AccountData>) -> Self {
        let mut accounts = HashMap::new();
        for (id, data) in witness {
            accounts.insert(
                id,
                AccountState {
                    balance: data.balance,
                    nonce: data.nonce,
                },
            );
        }
        Self { accounts }
    }

    /// Computes the cryptographic commitment (Root) of the current state.
    /// This proves that the state is what we say it is.
    pub fn compute_root(&self) -> [u8; 32] {
        // 1. Collect all entries
        let mut entries: Vec<(&AccountId, &AccountState)> = self.accounts.iter().collect();

        // 2. Sort by ID ensures determinism (essential for Merkle consistency)
        entries.sort_by_key(|(id, _)| id.0);

        // 3. Hash them all together
        let mut hasher = Hasher::new();
        for (id, state) in entries {
            hasher.update(&id.0);
            hasher.update(&state.balance.to_le_bytes());
            hasher.update(&state.nonce.to_le_bytes());
        }

        hasher.finalize().into()
    }
}

// Implement the Trait so BatchExecutor can use it
impl StateStore for ZkMemStore {
    fn get_account(&self, id: &AccountId) -> Result<AccountState> {
        Ok(self.accounts.get(id).cloned().unwrap_or_default())
    }

    fn set_account(&mut self, id: AccountId, state: AccountState) -> Result<()> {
        self.accounts.insert(id, state);
        Ok(())
    }
}
