pub mod memory;
pub mod processor;
pub mod storage;

pub use {
    memory::ZkMemStore,
    processor::BatchExecutor,
    storage::{AccountState, StateStore},
};

#[cfg(test)]
mod tests {
    use {
        super::*,
        std::collections::HashMap,
        zelana_core::{identity::AccountId, SignedTransaction, TransactionData},
    };

    //Mock Store (In-Memory)
    struct MockStore {
        accounts: HashMap<AccountId, AccountState>,
    }

    impl StateStore for MockStore {
        fn get_account(&self, id: &AccountId) -> anyhow::Result<AccountState> {
            Ok(self.accounts.get(id).cloned().unwrap_or_default())
        }
        fn set_account(&mut self, id: AccountId, state: AccountState) -> anyhow::Result<()> {
            self.accounts.insert(id, state);
            Ok(())
        }
    }

    #[test]
    fn test_transfer_execution() {
        let mut store = MockStore {
            accounts: HashMap::new(),
        };
        let alice = AccountId([1u8; 32]);
        let bob = AccountId([2u8; 32]);

        // Setup: Give Alice money
        store
            .set_account(
                alice,
                AccountState {
                    balance: 100,
                    nonce: 0,
                },
            )
            .unwrap();

        // Action: Alice sends 50 to Bob
        let tx_data = TransactionData {
            from: alice,
            to: bob,
            amount: 50,
            nonce: 0,
            chain_id: 1,
        };
        // NOTE: In this test we mock signature, as Executor assumes sig verified earlier
        let signed = SignedTransaction {
            data: tx_data,
            signature: vec![],
            signer_pubkey: [0u8; 32],
        };

        let mut executor = BatchExecutor::new(&mut store);
        executor
            .execute(&zelana_core::L2Transaction::Transfer(signed))
            .unwrap();

        // Verify Alice
        let alice_state = store.get_account(&alice).unwrap();
        assert_eq!(alice_state.balance, 50);
        assert_eq!(alice_state.nonce, 1);

        // Verify Bob
        let bob_state = store.get_account(&bob).unwrap();
        assert_eq!(bob_state.balance, 50);
    }
}
