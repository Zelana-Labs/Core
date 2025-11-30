use crate::{KIND_APP_DATA, KIND_CLIENT_HELLO, KIND_SERVER_HELLO};
use anyhow::{bail, Result};

#[derive(Debug)]
pub enum Packet<'a> {
    ClientHello {
        public_key: &'a [u8; 32],
    },
    ServerHello {
        public_key: &'a [u8; 32],
    },
    AppData {
        nonce: &'a [u8; 12],
        ciphertext: &'a [u8],
    },
}

impl<'a> Packet<'a> {
    /// Parses a raw UDP frame.
    pub fn parse(buf: &'a [u8]) -> Result<Self> {
        if buf.is_empty() {
            bail!("Empty packet");
        }

        match buf[0] {
            KIND_CLIENT_HELLO => {
                if buf.len() < 33 {
                    bail!("Malformed ClientHello");
                }
                let pk = array_ref(buf, 1);
                Ok(Packet::ClientHello { public_key: pk })
            }
            KIND_SERVER_HELLO => {
                if buf.len() < 33 {
                    bail!("Malformed ServerHello");
                }
                let pk = array_ref(buf, 1);
                Ok(Packet::ServerHello { public_key: pk })
            }
            KIND_APP_DATA => {
                if buf.len() < 13 {
                    bail!("Malformed AppData (Header too small)");
                }
                let nonce = array_ref_12(buf, 1);
                let ciphertext = &buf[13..];
                Ok(Packet::AppData { nonce, ciphertext })
            }
            _ => bail!("Unknown packet kind: {}", buf[0]),
        }
    }
}

// Helpers to safely cast slices to arrays
fn array_ref(slice: &[u8], offset: usize) -> &[u8; 32] {
    let ptr = slice[offset..].as_ptr() as *const [u8; 32];
    unsafe { &*ptr }
}

fn array_ref_12(slice: &[u8], offset: usize) -> &[u8; 12] {
    let ptr = slice[offset..].as_ptr() as *const [u8; 12];
    unsafe { &*ptr }
}
