use {
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_commitment_config::CommitmentConfig,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::{Keypair, Signer},
    solana_pubkey::Pubkey,
    solana_transaction::Transaction,
    std::{env, str::FromStr},
    wincode_derive::SchemaWrite,
};

// Define the struct manually to avoid circular deps with the bridge crate
#[derive(SchemaWrite)]
struct DepositParams {
    amount: u64,
    nonce: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Config
    let rpc_url = "http://127.0.0.1:8899";
    // We default to the ID you likely deployed. Change if different!
    let bridge_id_str = env::var("BRIDGE_PROGRAM_ID")
        .unwrap_or_else(|_| "DouWDzYTAxi5c3ui695xqozJuP9SpAutDcTbyQnkAguo".to_string());
    let program_id = Pubkey::from_str(&bridge_id_str)?;

    // 2. Setup User (The Depositor)
    let payer = Keypair::new();
    let rpc = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    println!("Airdropping SOL to  {}...", payer.pubkey());
    let sig = rpc.request_airdrop(&payer.pubkey(), 2_000_000_000).await?; // 2 SOL
    while !rpc.confirm_transaction(&sig).await? {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 3. Derive Bridge PDAs
    let (config_pda, _) = Pubkey::find_program_address(&[b"config"], &program_id);
    let (vault_pda, _) =
        Pubkey::find_program_address(&[b"vault", config_pda.as_ref()], &program_id);

    // Receipt PDA (Unique per deposit)
    let nonce: u64 = 101; // Arbitrary nonce for this test
    let nonce_le = nonce.to_le_bytes();
    let (receipt_pda, _) = Pubkey::find_program_address(
        &[
            b"receipt",
            config_pda.as_ref(),
            payer.pubkey().as_ref(),
            &nonce_le,
        ],
        &program_id,
    );

    // 4. Construct Instruction
    let amount = 1_000_000_000; // 1 SOL (1e9 lamports)
    let params = DepositParams { amount, nonce };

    // Discriminator: Assuming 'Deposit' is the 2nd instruction (index 1).
    // If you used the order: [Init, Deposit, Withdraw...], then it is 1.
    // If you used [Init, Withdraw, Deposit...], check your enum!
    let mut data = vec![1];
    data.extend(wincode::serialize(&params)?);

    let system_id = Pubkey::from_str("11111111111111111111111111111111")?;
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(config_pda, false),
        AccountMeta::new(vault_pda, false),
        AccountMeta::new(receipt_pda, false),
        AccountMeta::new_readonly(system_id, false),
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    // 5. Send
    let latest_blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        latest_blockhash,
    );

    println!("Sending Deposit of 1 SOL...");
    let sig = rpc.send_and_confirm_transaction(&tx).await?;
    println!("Deposit Confirmed! Sig: {}", sig);

    Ok(())
}
