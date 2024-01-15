use crate::{Error, Value, Config, SettingsDB, RpcApi, TBPubTransaction};
use crate::{json, json_to_string, hex_encode, hex_decode, get_bitcoin_rpc, send_transaction};
use crate::{TBPUB, FLAG_DID, FLAG_HASH};

use serde::{Serialize, Deserialize};

enum ArgumentType {
    String,
    Number,
    Bool
}

pub enum RequestMethod {
    BroadcastDID,
    BroadcastHash,
    GetInfo,
    Help
}

impl RequestMethod {
    fn from_string(method: &str) -> Option<RequestMethod> {
        match method {
            "broadcastdid" => Some(RequestMethod::BroadcastDID),
            "broadcasthash" => Some(RequestMethod::BroadcastHash),
            "getinfo" => Some(RequestMethod::GetInfo),
            "help" => Some(RequestMethod::Help),
            "?" => Some(RequestMethod::Help),
            _ => None
        }
    }

    fn argument_types(&self) -> Vec<(&str, ArgumentType)> {
        match *self {
            RequestMethod::BroadcastDID => vec![
                ("did", ArgumentType::String),
                ("price", ArgumentType::Number),
                ("check_mempool", ArgumentType::Bool)
            ],
            RequestMethod::BroadcastHash => vec![
                ("hash", ArgumentType::String),
                ("price", ArgumentType::Number),
                ("check_mempool", ArgumentType::Bool)
            ],
            RequestMethod::GetInfo => vec![],
            RequestMethod::Help => vec![("method", ArgumentType::String)],
        }
    }

    fn help(&self) -> JsonResponse {
        JsonResponse::help(format!("Help Message"))
    }
////        match *self {
////            RequestMethod::AddPage => JsonResponse::help("
////addpage ( name n_value data )

////Adds a page to the list of unpublished pages.

////Arguments:
////1. name              (string) The name of the page
////2. n_value           (numeric) The number of satoshis to publish this page under
////3. data              (string) A hex string containing upto 991 bytes of data as the content of the page

////Result:
////{                             (json object)
////  \"name\" : \"str\",             (alphanumeric string) The name of the page
////  \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
////  \"data\" : \"hex\",             (string) The data of the page
////}

////Examples:
////> lipcli addpage mynewpage 600 ae84532332efabfc".to_owned()),
////            RequestMethod::ListUnpublished => JsonResponse::help("
////listunpublished

////Returns an array of all unpublished pages

////Result:
////[                               (json array)
////  {                             (json object)
////    \"name\" : \"str\",             (alphanumeric string) The name of the page
////    \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
////    \"data\" : \"hex\",             (string) The data of the page
////  }
////]

////Examples:
////> lipcli listunpublished".to_owned()),
////            RequestMethod::ListPages => JsonResponse::help("
////listpages

////Returns an array of all pages that are 
////valid and stored in the Library

////Result:
////{                             (json object)
////  \"block_height\" : n,         (numeric) The block height the page was included in
////  \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
////  \"data\" : \"hex\",             (string) The data of the page
////}

////Examples:
////> lipcli listpages".to_owned()),
////            RequestMethod::GetPage => JsonResponse::help("
////getpage ( name )

////Gets a page from the list of unpublished pages 
////by name if it exists

////Arguments:
////1. name              (string) The name of the page

////Result:
////{                             (json object)
////  \"name\" : \"str\",             (alphanumeric string) The name of the page
////  \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
////  \"data\" : \"hex\",             (string) The data of the page
////}

////Examples:
////> lipcli getpage mypage".to_owned()),

////            RequestMethod::RemovePage => JsonResponse::help("
////removepage ( name )

////Removes a page from the list of unpublished pages

////Arguments:
////1. name              (string) The name of the page

////Result:
////n                               (bool) whether or not the page was removed

////Examples:
////> lipcli removepage myoldpage".to_owned()),
////            RequestMethod::Publish => JsonResponse::help("
////publish ( publish_fee )

////Publishes all unpublished pages

////Arguments:
////2. publish_fee           (numeric) The fee to publish the all the unpublished pages with, If 0 it will choose the minimum amount to successfully publish the book.

////Result:
////hex                             (string) The Txid of the transaction that broadcast the book of all unpublished pages

////Examples:
////> lipcli publish 100".to_owned()),
////        RequestMethod::Help => JsonResponse::help("
////== Commands  ==
////addpage \"name\" n_value \"data\"
////getpage \"name\"
////removepage \"name\"
////listunpublished
////listpages
////publish publish_fee".to_owned())

////        }
////    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRequest{
    pub method: String,
    pub args: Vec<String>,
}

impl JsonRequest {
    pub fn verify_request(&self) -> Result<(RequestMethod, Value), JsonResponse> {
        let mut result: Value = json!(null);
        let request_method = match RequestMethod::from_string(self.method.as_str()) {
            Some(r) => r,
            None => return Err(JsonResponse::error(String::new()+"Unknown Method("+&self.method+")"))
        };
        let arg_types = request_method.argument_types();
        if self.args.len() != arg_types.len() {return Err(request_method.help());}
        for ((arg_name, arg_type), arg) in arg_types.iter().zip(self.args.iter()) {
            match arg_type {
                ArgumentType::String => {
                    result[arg_name] = json!(arg);
                    if !result[arg_name].is_string() {return Err(JsonResponse::error(format!(
                            "Argument({}) is not a String", arg_name)));}
                },
                ArgumentType::Number => {
                    if !arg.chars().all(char::is_numeric) {
                        return Err(JsonResponse::error(format!(
                            "Argument({}) is not Numeric", arg_name)));
                    }
                    result[arg_name] = json!(arg.parse::<u64>().unwrap());
                },
                ArgumentType::Bool => {
                    result[arg_name] = json!(match arg.as_str() {
                        "true" => true,
                        "false" => false,
                        _ => return Err(JsonResponse::error(format!(
                            "Argument({}) is not a Boolean", arg_name)))
                    });
                },
            }
        }
        Ok((request_method, result))
    }

    pub fn handel_request(&self, config: &Config) -> Result<JsonResponse, Error> {
        let (method, args) = match self.verify_request() {
            Ok(value) => value,
            Err(response) => return Ok(response)
        };
        let rpc = get_bitcoin_rpc(&config)?;

        match method {
            RequestMethod::BroadcastHash => {
                let price = args["price"].as_u64().unwrap();
                let hash = args["hash"].as_str().unwrap();

                match hex_decode(&hash) {
                    Err(_) => return Ok(JsonResponse::error(format!(
                                "Argument({}) is not a valid Hex String", "hash"))),
                    Ok(bytes) => {
                        if bytes.len() != 20 {return Ok(JsonResponse::error(format!(
                                "Argument({}) must be 20 bytes long", "hash")))}
                    }
                }

                let txids = rpc.get_raw_mempool()?;
                for txid in txids {
                    let tx = match rpc.get_raw_transaction(&txid, None) {
                        Ok(tx) => tx,
                        Err(_) => continue
                    };

                    if TBPubTransaction::from_transaction(&tx).is_none() {continue;}

                    return Ok(JsonResponse::error(format!("TBPUB Transaction found in mempool with txid({})", txid)));
                }

                let output_script = format!("{}{}{}", TBPUB, hex_encode([FLAG_HASH]), hex_encode(hash));
                let txid = match send_transaction(&rpc, output_script, price) {
                    Err(e) => return Ok(JsonResponse::error(e.to_string())),
                    Ok(val) => val
                };
                let mut result: Value = json!(null);
                result["txid"] = json!(txid);
                Ok(JsonResponse::success(json_to_string(&result)?))
            },
            RequestMethod::BroadcastDID => {
                let price = args["price"].as_u64().unwrap();
                let did = args["did"].as_str().unwrap();
                //TODO: check if valid did
                let output_script = format!("{}{}{}", TBPUB, hex_encode([FLAG_DID]), hex_encode(did.as_bytes()));
                let txid = match send_transaction(&rpc, output_script, price) {
                    Err(e) => return Ok(JsonResponse::error(e.to_string())),
                    Ok(val) => val
                };
                let mut result: Value = json!(null);
                result["txid"] = json!(txid);
                Ok(JsonResponse::success(json_to_string(&result)?))
            },
            RequestMethod::GetInfo => {
                let settings = SettingsDB::new(&config)?;
                let mut result: Value = json!(null);
                result["block_height"] = json!(settings.get("block_height")?.unwrap().parse::<u64>()?);
                result["initial_block_scan"] = json!(settings.get("initial_block_scan")?.unwrap().parse::<u64>()? != 0);
                return Ok(JsonResponse::success(json_to_string(&result)?))
            },
            RequestMethod::Help => {
                let method_name = args["method"].as_str().unwrap();
                let request_method = match RequestMethod::from_string(&method_name) {
                    Some(r) => r,
                    None => return Ok(JsonResponse::error(format!("Unknown Method({})", method_name)))
                };
                Ok(request_method.help())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonResponse{
    pub status: u8,
    pub message: String,
}

impl JsonResponse {
    pub fn error(message: String) -> JsonResponse {
        JsonResponse{status: 0, message: message}
    }
    pub fn success(message: String) -> JsonResponse {
        JsonResponse{status: 1, message: message}
    }
    pub fn help(message: String) -> JsonResponse {
        JsonResponse{status: 2, message: message}
    }
}
