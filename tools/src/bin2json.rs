use {
    anyhow::Context,
    clap::Parser,
    serde_json::{json, Value},
    std::{
        fs::File,
        io::{self, Read, Write},
        path::PathBuf,
    },
    zelana_core::{prover::BatchInput, L2Transaction, SignedTransaction},
};

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
    #[arg(short = 'n', long, default_value_t = 4)]
    indent: usize,
}

fn signed_tx_to_json(signed: &SignedTransaction) -> Value {
    let d = &signed.data;
    json! ({
        "data": {
            "from" : hex::encode(d.from),
            "to" : hex::encode(d.to),
            "amount" : d.amount,
            "nonce" : d.nonce,
            "chain_id" : d.chain_id,
        },
        "signature" : hex::encode(&signed.signature),
        "signer_pubkey" : hex::encode(signed.signer_pubkey),
    })
}

fn l2tx_to_json(tx: &L2Transaction) -> Value {
    match tx {
        L2Transaction::Transfer(signed) => json!({
            "Transfer": signed_tx_to_json(signed)
        }),
        other => Value::String(format!("{:?}", other)),
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
            stdout.write_all(s.as_bytes()).context("write to stdout")?;
            Ok(())
        }
    }
}

fn batch_to_json_string(batch: &BatchInput, indent: usize) -> String {
    let mut root = json!({
        "pre_state_root": hex::encode(batch.pre_state_root),
        "transactions": [],
        "witness_accounts": {}
    });

    // Push transactions
    if let Some(arr) = root.get_mut("transactions").and_then(|v| v.as_array_mut()) {
        for tx in &batch.transactions {
            arr.push(l2tx_to_json(tx));
        }
    }

    // Insert witness accounts
    if let Some(map) = root
        .get_mut("witness_accounts")
        .and_then(|v| v.as_object_mut())
    {
        for (aid, acct) in batch.witness_accounts.iter() {
            map.insert(
                hex::encode(aid),
                json!({
                    "balance": acct.balance,
                    "nonce": acct.nonce,
                }),
            );
        }
    }

    if indent == 0 {
        serde_json::to_string(&root).unwrap()
    } else {
        serde_json::to_string_pretty(&root).unwrap()
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
