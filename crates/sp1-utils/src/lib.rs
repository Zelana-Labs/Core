use sp1_sdk::SP1Stdin;
use anyhow::Context;
use wincode::SchemaWrite;


/// Extension helpers to write wincode-serialized values into SP1Stdin
pub trait SP1StdinWincode {
    /// Serialize `value` using wincode and write the resulting bytes to the SP1 stdin buffer.
    /// This avoids the serde::Serialize bound that `SP1Stdin::write` imposes.
    fn write_wincode<T>(&mut self, value: &T) -> anyhow::Result<()>
    where 
        T: SchemaWrite<Src = T>;

    /// Write already-serialized bytes to SP1 stdin (thin wrapper around write_slice).
    fn write_wincode_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()>;
}

impl SP1StdinWincode for SP1Stdin {
    fn write_wincode<T>(&mut self, value: &T) -> anyhow::Result<()> 
    where
        T:  SchemaWrite<Src = T>{
        // wincode::serialize will require the generated SchemaWrite impl on T
        // but we don't need to mention any serde trait here.
        let bytes = wincode::serialize(value)
            .map_err(|e| anyhow::anyhow!("wincode serialize error: {}", e))?;
        // SP1Stdin exposes a byte-oriented write (write_slice / similar).
        // Use it so we never touch serde.
        self.write_slice(&bytes);
        Ok(())
    }

    fn write_wincode_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.write_slice(bytes);
        Ok(())
    }
}
