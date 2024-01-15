use crate::Transaction;
use crate::hex_encode;
use crate::{MINIMUM_TBPUB_TX_PRICE, TBPUB, OP_RETURN};

pub fn did_from_bytes(data: &[u8]) -> Option<&str> {
    let did = match std::str::from_utf8(data) {
        Err(_) => return None,
        Ok(s) => s
    };
    let mut split = did.split(":");
    if split.next() == Some("did") && split.next() == Some("dht") {
        let suffix = split.next();
        if suffix.is_some() && suffix.unwrap().len() == 52 {
            return suffix;
        }
    }
    None
}

#[non_exhaustive]
#[derive(Debug)]
pub struct TBPubTransaction {
    pub price: u64,
    pub data: String,
    pub is_hash: bool
}

impl TBPubTransaction {
    pub fn from_transaction(tx: &Transaction) -> Option<TBPubTransaction> {
        let mut result: Option<TBPubTransaction> = None;
        for output in &tx.output {
            let output_script = output.script_pubkey.to_bytes();
            if output_script.len() > 8 {
                if output_script[0] == OP_RETURN && hex_encode(&output_script[2..7]) == TBPUB {
                    if result.is_some() {return None;}
                    if output.value < MINIMUM_TBPUB_TX_PRICE {return None;}
                    let (data, is_hash) = match output_script[7] {
                        0x00 if output_script.len() == 28 => (output_script[8..28].to_vec(), true),
                        0x01 if did_from_bytes(&output_script[8..]).is_some() => (output_script[8..].to_vec(), false),
                        _ => return None
                    };
                    result = Some(TBPubTransaction{price: output.value, data: hex_encode(data), is_hash: is_hash});
                }
            }
        }
        result
    }
}
