use tokio::time::{sleep, Duration};
use zelana_sdk::{TransactionData, ZelanaClient, ZelanaWallet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //Setup Logging to see what happens
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    println!("CLIENT: Started");

    //Create a fresh Wallet (Identity)
    let wallet = ZelanaWallet::new_random();
    let my_id = wallet.account_id();
    println!("CLIENT: Identity created: {}", my_id.to_hex());

    //Connect to the Sequencer (Handshake happens here)
    println!("CLIENT: Connecting to Sequencer...");
    let mut client = ZelanaClient::connect("127.0.0.1:9000").await?;
    println!("CLIENT: Secure Session Established!");

    //Simulating ->
    //Send a stream of transactions (Simulating HFT)
    for i in 1..=5 {
        println!("CLIENT: Sending Tx #{}...", i);

        //Construct the Intent
        let tx_data = TransactionData {
            from: my_id,
            to: my_id, // Sending to self for testing
            amount: i * 10,
            nonce: i,
            chain_id: 1,
        };

        //Sign it (Ed25519)
        let signed_tx = wallet.sign_transaction(tx_data);

        //Encrypt & Broadcast (UDP)
        client.send_transaction(signed_tx).await?;

        sleep(Duration::from_millis(500)).await;
    }

    println!("👤 CLIENT: Finished sending transactions.");
    Ok(())
}
