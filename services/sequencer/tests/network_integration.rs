use {
    std::sync::Once,
    tokio::time::{sleep, Duration},
    zelana_sdk::{TransactionData, ZelanaClient, ZelanaWallet},
};

static INIT: Once = Once::new();

fn setup_logs() {
    INIT.call_once(|| {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
        env_logger::init();
    });
}

#[tokio::test]
async fn test_e2e_handshake_and_tx() {
    setup_logs();

    tokio::spawn(async move {
        use {
            x25519_dalek::PublicKey,
            zelana_net::{protocol::Packet, EphemeralKeyPair, KIND_SERVER_HELLO},
        };

        let socket = tokio::net::UdpSocket::bind("127.0.0.1:9001").await.unwrap();
        let mut buf = [0u8; 1500];

        loop {
            let (len, peer) = socket.recv_from(&mut buf).await.unwrap();
            let data = &buf[..len];

            if let Ok(Packet::ClientHello { public_key }) = Packet::parse(data) {
                // Generate keys
                let server_keys = EphemeralKeyPair::generate();
                let server_pub = *server_keys.pk.as_bytes();

                // Compute shared secret (ECDH)
                let client_pub = PublicKey::from(*public_key);
                let shared = server_keys.sk.diffie_hellman(&client_pub);
                let _shared_secret = shared.to_bytes();

                // Reply: ServerHello
                let mut resp = Vec::with_capacity(33);
                resp.push(KIND_SERVER_HELLO);
                resp.extend_from_slice(&server_pub);

                socket.send_to(&resp, peer).await.unwrap();
            }
        }
    });

    // Give server time to bind
    sleep(Duration::from_millis(100)).await;

    // 2. Connect Client
    let result = ZelanaClient::connect("127.0.0.1:9001").await;
    assert!(result.is_ok(), "Handshake failed: {:?}", result.err());

    let mut client = result.unwrap();

    // 3. Send Transaction
    let wallet = ZelanaWallet::new_random();
    let tx = wallet.sign_transaction(TransactionData {
        from: wallet.account_id(),
        to: wallet.account_id(),
        amount: 500,
        nonce: 1,
        chain_id: 1,
    });

    let send_result = client.send_transaction(tx).await;
    assert!(send_result.is_ok(), "Tx send failed");
}
