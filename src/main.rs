mod error;
use crate::error::Error;
mod config;
use crate::config::Config;
mod database;
use crate::database::SettingsDB;

//Import Crates
use bitcoincore_rpc::{Auth, Client, RpcApi};

//Import std
use std::time::Duration;
use std::thread::sleep;

//CONSENSUS Variables
const MIN_BLOCK_HEIGHT: u32 = 823913;

fn main() -> Result<(), Error> {
    let config = Config::new()?;
    dbg!(&config);
    let rpc = Client::new(&config.rpcurl, Auth::UserPass((&config.rpcuser).to_owned(), (&config.rpcpassword).to_owned()))?;
    while rpc.get_blockchain_info()?.verification_progress < 0.999 {
        println!("Bitcoin Sync Progress: {}", rpc.get_blockchain_info()?.verification_progress);
        sleep(Duration::from_millis(1000));
    }
    println!("Bitcoin Synced");

    let settings = SettingsDB::new(&config)?;

    let block_height = match settings.get("block_height")? {
        Some(bh) => bh.parse()?,
        None => {
            settings.set("block_height", &MIN_BLOCK_HEIGHT.to_string())?;
            MIN_BLOCK_HEIGHT
        }
    };
    println!("block_height: {}", block_height);


    Ok(())
}
