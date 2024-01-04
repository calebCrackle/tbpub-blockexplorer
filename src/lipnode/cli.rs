use crate::{LibraryDB, MainlineRPC, Error, Value, UnpublishedPage, Config, UnpublishedPagesDB, RpcApi};
use crate::{hex_decode, to_json, get_string, get_i64, get_vec_u8, encode_pages, get_bitcoin_rpc};
use crate::json;

use serde::{Serialize, Deserialize};

enum ArgumentType {
    StringAlphanumeric,
    StringHex(Box<dyn Fn(&Vec<u8>) -> bool>, String),
    Number(Box<dyn Fn(i64, &Config) -> bool>, String),
}

pub enum RequestMethod {
    AddPage,
    ListUnpublished,
    ListPages,
    GetPage,
    RemovePage,
    Publish,
    Help
}

impl RequestMethod {
    fn from_string(method: &str) -> Option<RequestMethod> {
        match method {
            "addpage" => Some(RequestMethod::AddPage),
            "listunpublished" => Some(RequestMethod::ListUnpublished),
            "listpages" => Some(RequestMethod::ListPages),
            "getpage" => Some(RequestMethod::GetPage),
            "removepage" => Some(RequestMethod::RemovePage),
            "publish" => Some(RequestMethod::Publish),
            "help" => Some(RequestMethod::Help),
            "?" => Some(RequestMethod::Help),
            _ => None
        }
    }

    fn argument_types(&self, config: &Config) -> Vec<(&str, ArgumentType)> {
        match *self {
            RequestMethod::AddPage => vec![
                ("name", ArgumentType::StringAlphanumeric), 
                ("n_value", ArgumentType::Number(Box::new(|val, config| {val >= config.min_page_n_value}), format!("n_value must be at least {}", config.min_page_n_value))), 
                ("data", ArgumentType::StringHex(Box::new(|val| {val.len() < 992}), "data must be less then 992 bytes".to_owned()))
            ],
            RequestMethod::ListUnpublished => vec![],
            RequestMethod::ListPages => vec![],
            RequestMethod::GetPage => vec![("name", ArgumentType::StringAlphanumeric)],
            RequestMethod::RemovePage => vec![("name", ArgumentType::StringAlphanumeric)],
            RequestMethod::Publish => vec![
                ("publisher_fee", ArgumentType::Number(Box::new(|_, _| {true}), "".to_string()))
            ],
            RequestMethod::Help => vec![("method", ArgumentType::StringAlphanumeric)],
        }
    }

    fn help(&self) -> JsonResponse {
        match *self {
            RequestMethod::AddPage => JsonResponse::help("
addpage ( name n_value data )

Adds a page to the list of unpublished pages.

Arguments:
1. name              (string) The name of the page
2. n_value           (numeric) The number of satoshis to publish this page under
3. data              (string) A hex string containing upto 991 bytes of data as the content of the page

Result:
{                             (json object)
  \"name\" : \"str\",             (alphanumeric string) The name of the page
  \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
  \"data\" : \"hex\",             (string) The data of the page
}

Examples:
> lipcli addpage mynewpage 600 ae84532332efabfc".to_owned()),
            RequestMethod::ListUnpublished => JsonResponse::help("
listunpublished

Returns an array of all unpublished pages

Result:
[                               (json array)
  {                             (json object)
    \"name\" : \"str\",             (alphanumeric string) The name of the page
    \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
    \"data\" : \"hex\",             (string) The data of the page
  }
]

Examples:
> lipcli listunpublished".to_owned()),
            RequestMethod::ListPages => JsonResponse::help("
listpages

Returns an array of all pages that are 
valid and stored in the Library

Result:
{                             (json object)
  \"block_height\" : n,         (numeric) The block height the page was included in
  \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
  \"data\" : \"hex\",             (string) The data of the page
}

Examples:
> lipcli listpages".to_owned()),
            RequestMethod::GetPage => JsonResponse::help("
getpage ( name )

Gets a page from the list of unpublished pages 
by name if it exists

Arguments:
1. name              (string) The name of the page

Result:
{                             (json object)
  \"name\" : \"str\",             (alphanumeric string) The name of the page
  \"n_value\" : n,              (numeric) The number of satoshis to publish this page under
  \"data\" : \"hex\",             (string) The data of the page
}

Examples:
> lipcli getpage mypage".to_owned()),

            RequestMethod::RemovePage => JsonResponse::help("
removepage ( name )

Removes a page from the list of unpublished pages

Arguments:
1. name              (string) The name of the page

Result:
n                               (bool) whether or not the page was removed

Examples:
> lipcli removepage myoldpage".to_owned()),
            RequestMethod::Publish => JsonResponse::help("
publish ( publish_fee )

Publishes all unpublished pages

Arguments:
2. publish_fee           (numeric) The fee to publish the all the unpublished pages with, If 0 it will choose the minimum amount to successfully publish the book.

Result:
hex                             (string) The Txid of the transaction that broadcast the book of all unpublished pages

Examples:
> lipcli publish 100".to_owned()),
        RequestMethod::Help => JsonResponse::help("
== Commands  ==
addpage \"name\" n_value \"data\"
getpage \"name\"
removepage \"name\"
listunpublished
listpages
publish publish_fee".to_owned())

        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRequest{
    pub method: String,
    pub args: Vec<String>,
}

impl JsonRequest {
    pub fn verify_request(&self, config: &Config) -> Result<(RequestMethod, Value), JsonResponse> {
        let mut result: Value = json!(null);
        let request_method = match RequestMethod::from_string(self.method.as_str()) {
            Some(r) => r,
            None => return Err(JsonResponse::error(String::new()+"Unknown Method("+&self.method+")"))
        };
        let arg_types = request_method.argument_types(config);
        if self.args.len() != arg_types.len() {return Err(request_method.help());}
        for ((arg_name, arg_type), arg) in arg_types.iter().zip(self.args.iter()) {
            match arg_type {
                ArgumentType::StringAlphanumeric => {
                    if !arg.chars().all(char::is_alphanumeric) {
                        return Err(JsonResponse::error(String::new()+"Argument("+arg_name+") is not Alphanumeric"));
                    }
                    result[arg_name] = json!(*arg);
                },
                ArgumentType::StringHex(constraint, errormsg) => {
                    let parsed_hex = match hex_decode(&arg) {
                        Ok(response) => response,
                        Err(_) => return Err(JsonResponse::error(String::new()+"Argument("+arg_name+") is not a valid Hex String")),
                    };
                    if !constraint(&parsed_hex) {return Err(JsonResponse::error(errormsg.to_string()));}
                    result[arg_name] = json!(parsed_hex);
                },
                ArgumentType::Number(constraint, errormsg) => {
                    if !arg.chars().all(char::is_numeric) {
                        return Err(JsonResponse::error(String::new()+"Argument("+arg_name+") is not Numeric"));
                    }
                    let number = arg.parse().unwrap();
                    if !constraint(number, config) {return Err(JsonResponse::error(errormsg.to_string()));}
                    result[arg_name] = json!(number);
                },
            }
        }
        Ok((request_method, result))
    }

    pub fn handel_request(&self, config: &Config, mainlinerpc: &MainlineRPC) -> Result<JsonResponse, Error> {
 
        let (method, args) = match self.verify_request(config) {
            Ok(value) => value,
            Err(response) => return Ok(response)
        };
        match method {
            RequestMethod::AddPage => {
                let unpublished_pages = UnpublishedPagesDB::new(&config)?;
                let pagename = get_string(&args["name"])?;
                if unpublished_pages.get_page(&pagename)?.is_some() {return Ok(JsonResponse::error(String::new()+"Page("+&pagename+") already exists"));}
                let page = unpublished_pages.add_page(UnpublishedPage::new(
                    get_string(&args["name"])?,
                    get_i64(&args["n_value"])?,
                    get_vec_u8(&args["data"])?)?)?;
                
                Ok(JsonResponse::success(to_json(&page)?))
            },
            RequestMethod::ListUnpublished => {
                let unpublished_pages = UnpublishedPagesDB::new(&config)?;
                Ok(JsonResponse::success(to_json(&unpublished_pages.get_pages()?)?))
            },
            RequestMethod::ListPages => {
                let library = LibraryDB::new(&config)?;
                Ok(JsonResponse::success(to_json(&library.get_pages()?)?))
            },
            RequestMethod::GetPage => {
                let unpublished_pages = UnpublishedPagesDB::new(&config)?;
                Ok(JsonResponse::success(to_json(&unpublished_pages.get_page(&get_string(&args["name"])?)?)?))
            },
            RequestMethod::RemovePage => {
                let unpublished_pages = UnpublishedPagesDB::new(&config)?;
                let pagename = get_string(&args["name"])?;
                match unpublished_pages.remove_page(&pagename)? {
                    Some(()) => Ok(JsonResponse::success(1.to_string())),
                    None => Ok(JsonResponse::success(0.to_string())),
                }
            },
            RequestMethod::Publish => {
                let unpublished_pages = UnpublishedPagesDB::new(&config)?;
                let rpc = get_bitcoin_rpc(&config, false)?;
                let pages = unpublished_pages.get_pages()?;
                if pages.len() < 1 {return Ok(JsonResponse::error("Must have at least one page submited".to_string()));}

                let pub_fee = get_i64(&args["publisher_fee"])?;
                let (master_hash, total, blobs) = encode_pages(pages)?;
                println!("total {}", total);
                let fee = if pub_fee > 0 {
                    let sat_pub_fee = pub_fee as f64 /100000000.0;
                    if sat_pub_fee < total {
                        return Ok(JsonResponse::error("publisher_fee(".to_owned()+&sat_pub_fee.to_string()+") must be greater then thin minimum fee("+&total.to_string()+") determined by the pages"))
                    }
                    sat_pub_fee
                } else {
                    total   
                };
                let mut args: Vec<Value> = vec![];
                let mut outputs: Value = json!(null);
                outputs[&config.address] = json!(fee);
                outputs["data"] = json!(master_hash);
                args.push(outputs);
                let tx: Value = match rpc.call("send", &args) {
                    Err(e) => return Ok(JsonResponse::error(e.to_string())),
                    Ok(val) => val
                };
                for blob in blobs {
                    match mainlinerpc.add_blob(blob) {
                        Ok(_) => (),
                        Err(e) => return Ok(JsonResponse::error(e.to_string())) }; }
                unpublished_pages.remove_pages()?;
                Ok(JsonResponse::success(get_string(&tx["txid"])?))
            },
            RequestMethod::Help => {
                let method_name = get_string(&args["method"])?;
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
