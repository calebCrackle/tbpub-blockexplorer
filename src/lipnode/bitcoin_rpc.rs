use crate::{Config, Error, Duration, Value, thread, Auth, Client, RpcApi};
use crate::{to_json};

pub fn get_bitcoin_rpc(config: &Config, watchonly: bool) -> Result<Client, Error> {
    let walletless_rpc = Client::new(&config.rpcurl, Auth::UserPass((&config.rpcuser).to_owned(), (&config.rpcpassword).to_owned()))?;
    Ok(if !watchonly {
        match walletless_rpc.load_wallet(&config.wallet) {
            Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(e))) if e.code == -35 => {
                Ok(()) //Wallet Already Loaded
            }
            Err(e) => Err(e),
            Ok(_) => Ok(())
        }?;
        let rpc = Client::new(&(config.rpcurl.clone()+"/wallet/"+&config.wallet), Auth::UserPass((&config.rpcuser).to_owned(), (&config.rpcpassword).to_owned()))?;
        if !rpc.get_wallet_info()?.private_keys_enabled {return Err(Error::WalletWatchOnly((&config.wallet).to_string()));}
        rpc
    } else {
        match walletless_rpc.load_wallet(&config.watchonlywallet) {
            Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(e))) if e.code == -18 => {
                walletless_rpc.create_wallet(&config.watchonlywallet, Some(true), None, None, None)?;
                Ok(()) //Wallet Created
            },
            Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(e))) if e.code == -35 => {
                Ok(()) //Wallet Already Loaded
            }
            Err(e) => Err(e),
            Ok(_) => Ok(())
        }?;
        let watchonly_rpc = Client::new(&(config.rpcurl.clone()+"/wallet/"+&config.watchonlywallet), Auth::UserPass((&config.rpcuser).to_owned(), (&config.rpcpassword).to_owned()))?;
        if watchonly_rpc.get_wallet_info()?.private_keys_enabled {return Err(Error::WalletNotWatchOnly((&config.watchonlywallet).to_string()));}

        let descriptor_json = to_json(&watchonly_rpc.call::<Value>("listdescriptors", &vec![])?)?;
        if !descriptor_json.contains(&config.descriptor) {
            println!("[INFO] Importing watchonly descriptor and rescanning blockchain, This may take a while");
            let thread_config: Config = config.clone();
            let thread_time: u64 = watchonly_rpc.get_block(&watchonly_rpc.get_block_hash(config.blockheight)?)?.header.time as u64;
            thread::spawn(move|| {
                let _ = Client::new(&(thread_config.rpcurl.clone()+"/wallet/"+&thread_config.watchonlywallet), Auth::UserPass((&thread_config.rpcuser).to_owned(), (&thread_config.rpcpassword).to_owned())).unwrap().import_descriptors(bitcoincore_rpc::bitcoincore_rpc_json::ImportDescriptors{ descriptor: (&thread_config.descriptor).to_owned(), timestamp: bitcoincore_rpc::bitcoincore_rpc_json::Timestamp::Time(thread_time), active: None, range: None, next_index: None, internal: None, label: None});
            });
            thread::sleep(Duration::from_secs(1));   
            loop {
                match watchonly_rpc.import_descriptors(bitcoincore_rpc::bitcoincore_rpc_json::ImportDescriptors{ descriptor: (&config.descriptor).to_owned(), timestamp: bitcoincore_rpc::bitcoincore_rpc_json::Timestamp::Now, active: None, range: None, next_index: None, internal: None, label: None}) {
                    Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(e))) if e.code == -4 => Ok(()),
                    Err(e) => Err(e),
                    Ok(_) => break,
                }?;
                thread::sleep(Duration::from_secs(1));   
            }
        }
        watchonly_rpc
    })
}
