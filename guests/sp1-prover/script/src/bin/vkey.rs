use sp1_sdk::{ProverClient,HashableKey};

const GUEST_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main(){
    sp1_sdk::utils::setup_logger();

    let client = ProverClient::new();
    let ( pk , vk) = client.setup(GUEST_ELF);
    
    println!("--");
    println!("VKey Hash (Bytes): {:?}", vk.bytes32());
    println!("VKey Hash (Hex):   {}", hex::encode(vk.bytes32()));
    println!("--");

}