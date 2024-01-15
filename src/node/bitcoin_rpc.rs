use crate::{Config, Error, Value, Auth, Client, RpcApi};
use crate::{hex_encode, hex_decode, json};
use crate::OP_RETURN;

pub fn get_bitcoin_rpc(config: &Config) -> Result<Client, Error> {
    let walletless_rpc = Client::new(&config.rpcurl, Auth::UserPass((&config.rpcuser).to_owned(), (&config.rpcpassword).to_owned()))?;
    match walletless_rpc.load_wallet(&config.wallet) {
        Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(e))) if e.code == -35 => {
            Ok(()) //-35 == Wallet Already Loaded
        }
        Err(e) => Err(e),
        Ok(_) => Ok(())
    }?;
    Ok(Client::new(&(config.rpcurl.clone()+"/wallet/"+&config.wallet), Auth::UserPass((&config.rpcuser).to_owned(), (&config.rpcpassword).to_owned()))?)
}

pub fn send_transaction(rpc: &Client, output_script: String, price: u64) -> Result<String, Error> {
    let mut outputs: Value = json!(null);
    outputs["data"] = json!(output_script);
    let inputs: Vec<Value> = vec![];
    let crt_args: Vec<Value> = vec![json!(inputs), json!(outputs)];
    let mut raw_tx: String = rpc.call("createrawtransaction", &crt_args)?;

    //Modify raw hex string to bypass safety constraints set by Bitcoin Core
    //OP_RETURN appears at the start of the script. Get the position of the first
    //OP_RETURN and there for the start of the script. That Index-1 is a length byte
    //for the script. And just behind that Index-9...Index-1 are the 8 bytes for the
    //amount this script is paid.
    //...0000000000000000 00 6a...
    let position = hex_decode(&raw_tx)?.iter().position(|&r| r == OP_RETURN).unwrap();
    raw_tx = raw_tx[..(position-9)*2].to_owned()+
        &hex_encode((price as i64).to_le_bytes())+
        &raw_tx[(position-1)*2..];

    rpc.call::<Value>("decoderawtransaction", &vec![json!(raw_tx)])?;

    let frt_args: Vec<Value> = vec![json!(raw_tx)];
    let funded_tx: Value = rpc.call("fundrawtransaction", &frt_args)?;

    let srt_args: Vec<Value> = vec![funded_tx["hex"].clone()];
    let signed_tx = rpc.call::<Value>("signrawtransactionwithwallet", &srt_args)?;

    if !signed_tx["complete"].as_bool().unwrap() {return Err(Error::CouldNotSignTransaction())}

    let send_rt_args: Vec<Value> = vec![signed_tx["hex"].clone(), json!(0.10), json!(1)];
    Ok(rpc.call::<String>("sendrawtransaction", &send_rt_args)?)
}
