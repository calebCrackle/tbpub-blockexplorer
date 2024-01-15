mod bitcoin_rpc;
use crate::bitcoin_rpc::{get_bitcoin_rpc, send_transaction};
mod config;
use crate::config::Config;
mod database;
use crate::database::{SettingsDB, HashesDB, RootDIDsDB};
mod tbpub_transaction;
use crate::tbpub_transaction::{TBPubTransaction};
mod error;
use crate::error::{Error};
mod cli;
use crate::cli::JsonRequest;
mod system;
use crate::system::{spawn_thread};

use bitcoin::Transaction;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;

use serde_json::from_str as json_from_str;
use serde_json::to_string as json_to_string;
use hex::decode as hex_decode;
use hex::encode as hex_encode;

const MINIMUM_TBPUB_TX_PRICE: u64 = 10000;
const MINIMUM_BLOCK_HEIGHT: u64 = 825478;
const TBPUB: &str = "5442505542";
const FLAG_HASH: u8 = 0x00;
const FLAG_DID: u8 = 0x01;
const OP_RETURN: u8 = 0x6a;

//#[tokio::main]
fn main() -> Result<(), Error> {
    let config = Config::new()?;
    println!("[INFO] Config: {:?}", &config);
    let settings = SettingsDB::new(&config)?;
    let mut block_height = match settings.get("block_height")? {
        Some(bh) => bh.parse::<u64>()?,
        None => {
            settings.set("block_height", &MINIMUM_BLOCK_HEIGHT.to_string())?; 
            MINIMUM_BLOCK_HEIGHT
        }
    };
    let ibs = match settings.get("initial_block_scan")? {
        Some(ibs) => ibs.parse::<u64>()? != 0,
        None => {
            settings.set("initial_block_scan", "1")?; 
            true
        }
    };
    println!("[INFO] Block Height: {}", block_height);
    println!("[INFO] Initial Block Scan: {}", ibs);

    spawn_thread(|config| -> Result<(), Error> {
        let listener = TcpListener::bind(&config.cliurl.clone())?;
        for income in listener.incoming() {
            let mut stream = income?; 
            spawn_thread(move|config| -> Result<(), Error> {
                let mut data = String::new();
                stream.read_to_string(&mut data)?;
                let request: JsonRequest = json_from_str(&data)?;
                let response = request.handel_request(&config)?;
                stream.write(json_to_string(&response)?.as_bytes())?;
                Ok(())
            }, config.clone());
        }
        Ok(())
    }, config.clone());
    println!("Started LIPNODE Listener!");

    let rpc = get_bitcoin_rpc(&config)?;
    let top_block = rpc.get_block_count()?;
    println!("[INFO] Top Block: {}", top_block);
    let hashes = HashesDB::new(&config)?;
    let rootdids = RootDIDsDB::new(&config)?;

    loop {
        //Scan for blocks
        if block_height <= top_block {
            println!("[INFO] Checking block {}", block_height);
            let block_hash = rpc.get_block_hash(block_height)?;
            println!("[INFO] Block {} has hash {}", block_height, block_hash);
            let block = rpc.get_block(&block_hash)?;
            let mut tbpub_tx: Option<TBPubTransaction> = None;
            for tx in block.txdata {
                match TBPubTransaction::from_transaction(&tx) {
                    Some(tmp_tx) => {
                        match tbpub_tx {
                            Some(stored_tx) if stored_tx.price < tmp_tx.price => tbpub_tx = Some(tmp_tx),
                            None => tbpub_tx = Some(tmp_tx),
                            _ => continue
                        }
                    },
                    None => continue
                };
            }
            if tbpub_tx.is_some() {
                let top_tbpub_tx = tbpub_tx.unwrap();
                if top_tbpub_tx.is_hash {
                    hashes.add(&top_tbpub_tx.data, block_height, top_tbpub_tx.price)?;
                } else {
                    rootdids.add(&top_tbpub_tx.data, block_height, top_tbpub_tx.price)?;
                }
            }
            block_height += 1;
            settings.set("block_height", &block_height.to_string())?;
        } else {
            //Waiting for next block and Initial Block Scan is finished.
            settings.set("initial_block_scan", "0")?;
        }
    }
}
