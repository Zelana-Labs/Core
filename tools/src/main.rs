use std::fs::File;
use std::io::Read;
use anyhow::Context;

use zelana_core::prover::BatchInput;

fn main() -> anyhow::Result<()> {
    let mut f = File::open("batch.bin").context("open batch.bin")?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).context("read batch.bin")?;

    // Deserialize using wincode (uses SchemaRead derive already present on the type)
    let batch: BatchInput = wincode::deserialize(&bytes)
        .map_err(|e| anyhow::anyhow!("wincode deserialize error: {}", e))?;

    // Write JSON (pretty)
    let out = File::create("batch.json").context("create batch.json")?;
    serde_json::to_writer_pretty(out, &batch).context("write json")?;

    println!("Wrote batch.json");
    Ok(())
}