mod bitcoin_rpc;
use crate::bitcoin_rpc::get_bitcoin_rpc;
mod config;
use crate::config::Config;
mod database;
use crate::database::{DataDB, UnpublishedPagesDB, RepublisherBlobsDB, LibraryDB};
mod structs;
use crate::structs::{DataBlob, UnpublishedPage, Page};
mod error;
use crate::error::Error;
mod hash;
use crate::hash::hash;
mod json;
use crate::json::{get_string, get_i64, get_vec_u8};
mod mainline;
use crate::mainline::{MainlineRPC};
mod cli;
use crate::cli::JsonRequest;
mod validation;
use crate::validation::{encode_pages, validate_utxo};
mod system;
use crate::system::{spawn_thread};

use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde_json::{Value, json};

use std::{thread, time::Duration};
use std::io::{Read, Write};
use std::net::TcpListener;

use serde_json::from_str as from_json;
use serde_json::to_string as to_json;
use hex::decode as hex_decode;
use hex::encode as hex_encode;

//#[tokio::main]
fn main() -> Result<(), Error> {
    let mut config = Config::new()?;
    let data = DataDB::new(&config)?;
    if config.blockheight > 0 {
        data.set("blockheight", &config.blockheight.to_string())?;
    } else {
        let blockheight = data.get("blockheight")?;
        if blockheight.is_some() {
            config.blockheight = blockheight.unwrap().parse().unwrap();
        } else { 
            data.set("blockheight", &818100.to_string())?;
        }
    }
    println!("[INFO] Config: {:?}", &config);

    println!("Start LIPNODE Listener!");
    spawn_thread(|config| -> Result<(), Error> {
        let listener = TcpListener::bind(&config.cliurl.clone())?;
        for income in listener.incoming() {
            let mut stream = income?; 
            spawn_thread(move|config| -> Result<(), Error> {
                let mut data = String::new();
                stream.read_to_string(&mut data)?;
                let request: JsonRequest = from_json(&data)?;
                let mainlinerpc = MainlineRPC::new(&config)?;
                let response = request.handel_request(&config, &mainlinerpc)?;
                stream.write(to_json(&response)?.as_bytes())?;
                Ok(())
            }, config.clone());
        }
        Ok(())
    }, config.clone());

    let watchonly_rpc = get_bitcoin_rpc(&config, true)?;
    let mainlinerpc = MainlineRPC::new(&config)?;
    let republisher_blobs = RepublisherBlobsDB::new(&config)?;
    let library = LibraryDB::new(&config)?;

    let mut daemon_time = 1;
    loop {
        if (daemon_time % 1*30) == 0 {
            let blobs = republisher_blobs.get_blobs()?;
            for blob in blobs {
                if mainlinerpc.get_blob(&hex_encode(blob.hash())).is_none() {
                    println!("[INFO] Republished Blob: {}", &hex_encode(blob.hash()));
                    mainlinerpc.add_blob(blob)?;
                }
            }
        }

        let block_height = watchonly_rpc.get_block_count()?;
        let unspent = watchonly_rpc.list_unspent(Some(0), None, None, Some(true), None)?;
        let checked_block_height = data.get("blockheight")?.unwrap().parse::<u64>()?;
        for utxo in &unspent {
            if checked_block_height < (block_height-utxo.confirmations as u64) {
                match validate_utxo(&config, utxo, &watchonly_rpc, &mainlinerpc)? {
                    Some((pages, blobs)) => {
                        for page in pages {
                            println!("[INFO] Adding Page To Library From Txid: {}", &utxo.txid.to_string());
                            library.add_page(page)?;
                        }
                        for blob in blobs {
                            println!("[INFO] Adding Blob To Republisher: {}", &hex_encode(blob.hash()));
                            republisher_blobs.add_blob(blob)?;
                        }
                    },
                    None => break
                }
            } else {break;}
        }
        if unspent.len() > 0 && checked_block_height < block_height-(unspent[0].confirmations as u64) {data.set("blockheight", &(block_height-unspent[0].confirmations as u64).to_string())?;}

        thread::sleep(Duration::from_secs(1));   
        daemon_time += 1;
    }
}
