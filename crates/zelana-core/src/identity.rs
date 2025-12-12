use {
    sha2::{Digest, Sha256},
    std::fmt,
    wincode::{SchemaRead, SchemaWrite},
};

/// The canonical identifier for a user on L2 (32 bytes).
/// Derived from H(SignerPK || PrivacyPK)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, SchemaWrite, SchemaRead)]

pub struct AccountId(pub [u8; 32]);

impl AccountId {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl AsRef<[u8]> for AccountId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AccountId({})", self.to_hex())
    }
}

/// Helper struct to hold a user's full keypair set.
#[derive(Clone, Debug)]
pub struct IdentityKeys {
    pub signer_pk: [u8; 32],  // Ed25519 Public Key
    pub privacy_pk: [u8; 32], // X25519 Public Key (for encryption)
}

impl IdentityKeys {
    /// Deterministically derives the L2 Account ID.
    /// Formula: SHA256( signer_pk_bytes || privacy_pk_bytes )
    pub fn derive_id(&self) -> AccountId {
        let mut hasher = Sha256::new();
        hasher.update(self.signer_pk);
        hasher.update(self.privacy_pk);
        AccountId(hasher.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_id_derivation() {
        let signer = [1u8; 32];
        let privacy = [2u8; 32];
        let keys = IdentityKeys {
            signer_pk: signer,
            privacy_pk: privacy,
        };

        let id = keys.derive_id();
        // If this hash changes, we broke the protocol.
        assert_eq!(
            hex::encode(id.0),
            "f818afd37a6dc3bc92fb44731011277006db4efa6e9023cd7468c02335d22a4d"
        );
    }
}
