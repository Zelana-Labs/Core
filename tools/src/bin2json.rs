use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use json::{object, JsonValue};

use zelana_core::prover::BatchInput;
use zelana_core::{L2Transaction, SignedTransaction};

/// Convert a wincode-serialized BatchInput (batch.bin) into JSON.
///
/// Examples:
///   bin2json --input batch.bin --output batch.json
///   bin2json --input -         # read from stdin, write to stdout
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input binary file (wincode). Use '-' for stdin.
    #[arg(short, long, default_value = "batch.bin")]
    input: String,

    /// Output JSON file. If omitted or '-', write to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print indentation (number of spaces). Use 0 for compact single-line JSON.
    #[arg(short = 'i', long, default_value_t = 4)]
    indent: usize,
}

fn signed_tx_to_json(signed: &SignedTransaction) -> JsonValue {
    let d = &signed.data;
    object!{
        "data" => object!{
            "from" => hex::encode(&d.from),
            "to" => hex::encode(&d.to),
            "amount" => d.amount,
            "nonce" => d.nonce,
            "chain_id" => d.chain_id,
        },
        "signature" => hex::encode(&signed.signature),
        "signer_pubkey" => hex::encode(&signed.signer_pubkey),
    }
}

fn l2tx_to_json(tx: &L2Transaction) -> JsonValue {
    match tx {
        L2Transaction::Transfer(signed) => object!{ "Transfer" => signed_tx_to_json(signed) },
        other => JsonValue::String(format!("{:?}", other)),
    }
}

fn read_all_input(input: &str) -> anyhow::Result<Vec<u8>> {
    if input == "-" {
        let mut buf = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .context("reading from stdin")?;
        Ok(buf)
    } else {
        let mut f = File::open(input).with_context(|| format!("open {}", input))?;
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).context("read input file")?;
        Ok(bytes)
    }
}

fn write_output(output: &Option<PathBuf>, s: &str) -> anyhow::Result<()> {
    match output {
        Some(path) if path.as_os_str() != "-" => {
            std::fs::write(path, s).with_context(|| format!("write to {}", path.display()))?;
            Ok(())
        }
        _ => {
            // Write to stdout
            let mut stdout = io::stdout();
            stdout
                .write_all(s.as_bytes())
                .context("write to stdout")?;
            Ok(())
        }
    }
}

fn batch_to_json_string(batch: &BatchInput, indent: usize) -> String {
    let mut root = object!{
        "pre_state_root" => hex::encode(&batch.pre_state_root),
        "transactions" => JsonValue::new_array(),
        "witness_accounts" => JsonValue::new_object(),
    };

    for tx in &batch.transactions {
        root["transactions"].push(l2tx_to_json(tx)).ok();
    }

    for (aid, acct) in batch.witness_accounts.iter() {
        root["witness_accounts"][hex::encode(aid)] = object!{
            "balance" => acct.balance,
            "nonce" => acct.nonce,
        };
    }
    let ident_u16 = u16::try_from(indent).expect("error while converthing to u16");
    if indent == 0 {
        json::stringify(root)
    } else {
        json::stringify_pretty(root, ident_u16)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Read input bytes (file or stdin)
    let bytes = read_all_input(&args.input)?;

    // Deserialize using wincode (SchemaRead derived on BatchInput)
    let batch: BatchInput = wincode::deserialize(&bytes)
        .map_err(|e| anyhow::anyhow!("wincode deserialize error: {}", e))?;

    // Convert to JSON string
    let out = batch_to_json_string(&batch, args.indent);

    // Write to file or stdout
    write_output(&args.output, &out)?;

    println!(
        "Converted input '{}' -> {}",
        args.input,
        args.output
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "stdout".into())
    );

    Ok(())
}