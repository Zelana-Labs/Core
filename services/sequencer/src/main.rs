mod executor;
mod session;
mod db;

use ed25519_dalek::SigningKey;
use executor::TransactionExecutor;
use log::{debug, error, info, warn};
use session::SessionManager;
use zelana_execution::AccountState;
use std::sync::Arc;
use tokio::net::UdpSocket;
use x25519_dalek::{PublicKey, StaticSecret};
use zelana_core::{IdentityKeys, L2Transaction, SignedTransaction};
use zelana_net::{
    protocol::Packet, EphemeralKeyPair, SessionKeys, KIND_APP_DATA, KIND_CLIENT_HELLO,
    KIND_SERVER_HELLO,
};

const MAX_DATAGRAM_SIZE: usize = 1500; // Standard MTU safe limit

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Sequencer  Starting...");

    //Bind UDP Socket
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:9000").await?);
    info!("Listening on UDP 0.0.0.0:9000");

    //Initialize State
    let sessions = Arc::new(SessionManager::new());
    let executor = TransactionExecutor::new("./data/sequencer_db")?;

    {
        // 1. Re-derive the ID from the same seed [7u8; 64]
        let seed = [7u8; 64];
        let sign_seed: [u8; 32] = seed[0..32].try_into().unwrap();
        let enc_seed: [u8; 32] = seed[32..64].try_into().unwrap();

        let sign_sk = SigningKey::from_bytes(&sign_seed);
        let enc_sk = StaticSecret::from(enc_seed);
        let keys = IdentityKeys {
            signer_pk: sign_sk.verifying_key().to_bytes(),
            privacy_pk: *PublicKey::from(&enc_sk).as_bytes(),
        };
        let whale_id = keys.derive_id();

        // 2. Inject 1 Million tokens
        // We use a new block to drop the 'store' mutable borrow immediately
        let mut store = executor.db.clone(); // Clone the Arc/wrapper
        zelana_execution::StateStore::set_account(
            &mut store, 
            whale_id, 
            AccountState { balance: 1_000_000, nonce: 0 }
        )?;
        info!("Genesis: Funded {} with 1M tokens", whale_id.to_hex());
    }

    let mut buf = [0u8; MAX_DATAGRAM_SIZE];


    loop {
        //Receive Packet
        let (len, peer) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                error!("UDP Receive Error: {}", e);
                continue;
            }
        };

        let packet_data = &buf[..len];

        //Zero-Copy Parse
        match Packet::parse(packet_data) {
            Ok(Packet::ClientHello { public_key }) => {
                debug!("ClientHello from {}", peer);

                //Generate Server Ephemeral Keys
                let server_keys = EphemeralKeyPair::generate();
                let server_pub_bytes = *server_keys.pk.as_bytes();

                //Convert client public key bytes → x25519_dalek::PublicKey
                // public_key: & [u8; 32]
                let client_public = PublicKey::from(*public_key);

                //Derive Session (EphemeralSecret × PublicKey → SharedSecret)
                let shared = server_keys.sk.diffie_hellman(&client_public);
                let shared_secret = shared.to_bytes(); // [u8; 32]

                let session = SessionKeys::derive(shared_secret, public_key, &server_pub_bytes);

                //Store Session
                sessions.insert(peer, session);

                //Send ServerHello
                let mut response = Vec::with_capacity(33);
                response.push(KIND_SERVER_HELLO);
                response.extend_from_slice(&server_pub_bytes);

                if let Err(e) = socket.send_to(&response, peer).await {
                    warn!("Failed to send ServerHello to {}: {}", peer, e);
                }
            }

            Ok(Packet::AppData { nonce, ciphertext }) => {
                //Lookup Session
                let decrypted_opt =
                    sessions.get_mut(&peer, |session| session.keys.decrypt(nonce, ciphertext));

                match decrypted_opt {
                    Some(Ok(plaintext)) => {
                        //Handle Transaction
                        match handle_transaction(&plaintext, &executor).await {
                            Ok(_) => debug!("Tx Executed from {}", peer),
                            Err(e) => warn!("Tx Failed from {}: {}", peer, e),
                        }
                    }
                    Some(Err(e)) => {
                        warn!("Decryption failed for {}: {}", peer, e);
                        // Potential Replay Attack or Bad Key - Drop Session
                    }
                    None => {
                        debug!("Unknown Peer {}, ignoring AppData", peer);
                        // Client sent data but we have no session (Server restarted?)
                        // Ideally send a "Reset" packet here so client reconnects
                    }
                }
            }

            Ok(Packet::ServerHello { .. }) => {
                // Clients send ClientHello, not ServerHello. Ignore.
            }

            Err(e) => {
                warn!("🗑️ Malformed packet from {}: {}", peer, e);
            }
        }
    }
}

/// Decodes and routes the transaction to the executor
async fn handle_transaction(
    plaintext: &[u8],
    executor: &TransactionExecutor,
) -> anyhow::Result<()> {
    //Deserialize
    let tx: L2Transaction = wincode::deserialize(plaintext)?;

    match tx {
        L2Transaction::Transfer(signed_tx) => {
            //Validate Signature (Anti-Spoofing)
            // Even though ZK proves this later, we MUST check it now to protect the Sequencer.
            verify_signature(&signed_tx)?;

            //Execute
            executor.process(signed_tx).await?;
        }
        _ => {
            // Handle Deposits/Withdrawals
        }
    }
    Ok(())
}

fn verify_signature(tx: &SignedTransaction) -> anyhow::Result<()> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let vk = VerifyingKey::from_bytes(&tx.signer_pubkey)?;
    let sig = Signature::from_slice(&tx.signature)?;

    // Re-serialize data to verify (Must match SDK serialization exactly)
    let msg = wincode::serialize(&tx.data)?;

    vk.verify(&msg, &sig)?;
    Ok(())
}
