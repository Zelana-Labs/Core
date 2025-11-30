use sp1_sdk::{ProverClient, SP1Stdin};
use zelana_core::prover::BatchInput;
use std::fs::File;
use anyhow::Context;
use std::io::{BufReader, Read};
use clap::Parser;

use sp1_utils::SP1StdinWincode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the batch input file
    #[arg(short, long, default_value = "batch.bin")]
    input: String,

    /// Prove locally (CPU) or use the Network (GPU/Prover Network)
    #[arg(long, default_value_t = false)]
    network: bool,

    /// Path to the guest ELF file. If omitted, a few default locations will be tried.
    #[arg(long)]
    guest_elf: Option<String>,
}

fn load_guest_elf_from_candidates(candidates: &[&str]) -> anyhow::Result<Vec<u8>> {
    for p in candidates {
        match std::fs::read(p) {
            Ok(bytes) => {
                eprintln!("Loaded guest ELF from: {}", p);
                return Ok(bytes);
            }
            Err(_) => continue,
        }
    }
    Err(anyhow::anyhow!("no guest ELF found in candidates"))
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup
    sp1_sdk::utils::setup_logger();
    let args = Args::parse();

    println!("Starting Zelana Prover...");
    println!("Reading Input: {}", args.input);

    // Read raw wincode bytes for the batch input
    let file = File::open(&args.input)
        .with_context(|| format!("opening batch input file `{}`", &args.input))?;
    let mut reader = BufReader::new(file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    // let batch: BatchInput = serde_json::from_reader(reader)?;
    let batch: BatchInput = wincode::deserialize(&bytes).map_err(|e| anyhow::anyhow!("deserailize error :{}",e))?; 
    println!("Batch contains {} transactions.", batch.transactions.len());
    
    //Setup SP1 Inputs
    let mut stdin = SP1Stdin::new();
    stdin.write_wincode_bytes(&bytes)?;

    let guest_elf_bytes: Vec<u8> = if let Some(p) = args.guest_elf.as_deref() {
        std::fs::read(p)
            .with_context(|| format!("reading guest ELF from `{}`", p))?
    } else {
        let candidates = [
            "../../../../../target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1-prover",
        ];
        load_guest_elf_from_candidates(&candidates)
            .with_context(|| "trying default guest ELF candidate paths; pass --guest-elf to override")?
    };

    //Initialize Prover
    let client = ProverClient::new();
    let (pk, vk) = client.setup(&guest_elf_bytes);

    //Execute (Fast Mode - No Proof) first to check logic
    println!("Simulating execution (Witness Generation)...");
    let (output, report) = client.execute(&guest_elf_bytes, &stdin.clone())
        .run()
        .expect("Execution failed. Logic bug in Guest?");
    
    println!("Simulation Successful! Cycles: {}", report.total_instruction_count());

    //Generate Proof (Slow Mode)
    println!("Generating Zero-Knowledge Proof...");
    let proof = client.prove(&pk, &stdin).run()
        .expect("Proving failed");

    println!("Proof Generated Successfully!");

    //Save Proof
    proof.save("proof-with-io.bin")?;
    println!("Saved to 'proof-with-io.bin'");

    Ok(())
}