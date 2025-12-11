use {
    anyhow::Context,
    clap::Parser,
    sp1_sdk::{ProverClient, SP1Stdin},
    std::{
        fs::File,
        io::{BufReader, Read},
    },
    zelana_core::prover::BatchInput,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "batch.bin")]
    input: String,

    #[arg(long, default_value_t = false)]
    network: bool,
}

const GUEST_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup
    sp1_sdk::utils::setup_logger();
    let args = Args::parse();

    println!("Starting Zelana Prover...");

    let file = File::open(&args.input)
        .with_context(|| format!("Failed to open input file: {}", args.input))?;
    let mut reader = BufReader::new(file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    let batch: BatchInput = wincode::deserialize(&bytes)
        .map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))?;

    println!("Batch contains {} transactions.", batch.transactions.len());

    let mut stdin = SP1Stdin::new();
    stdin.write(&bytes);

    //Initialize Prover
    let client = ProverClient::new();
    let (pk, vk) = client.setup(GUEST_ELF);

    //Execute (Fast Mode - No Proof) first to check logic
    println!("Simulating execution...");
    let (_, report) = client
        .execute(GUEST_ELF, &stdin.clone())
        .run()
        .expect("Execution failed inside ZKVM. Check your Guest logic!");

    println!(
        "Simulation Successful! Cycles: {}",
        report.total_instruction_count()
    );

    println!("Generating Proof...");
    let proof = client.prove(&pk, &stdin).run().expect("Proving failed");

    println!("Proof Generated!");
    proof.save("proof-with-io.bin")?;

    Ok(())
}
