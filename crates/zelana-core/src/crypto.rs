use blake3::Hasher as Blake3Hasher;
use wincode::{serialize, SchemaWrite};

/// The standard hash function for the Protocol.
/// Currently BLAKE3 (Fastest on CPU, SP1-friendly).
pub fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Blake3Hasher::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn to_bytes<T>(value: &T) -> Vec<u8>
where
    T: SchemaWrite<Src = T>,
{
    serialize(value).expect("Serialization should never fail for core types")
}
