use crate::{Config, Error, Page, DataBlob, UnpublishedPage, Client, RpcApi, MainlineRPC};
use crate::{hex_encode};

pub fn encode_pages(pages: Vec<UnpublishedPage>) -> Result<(String, f64, Vec<DataBlob>), Error> {
    let mut total: i64 = 0;
    let mut encoded: Vec<DataBlob> = vec![];
    for page in pages {
        total += page.n_value;
        encoded.push(DataBlob::from_page(page));
    }
    let mut result: Vec<DataBlob> = encoded.clone();
    while encoded.len() > 49 {
        let chunk = encoded[0..49].to_vec();
        let mut new_encoded = vec![];
        let branch = DataBlob::new_branch(&chunk)?;
        result.push(branch.clone());
        new_encoded.push(branch);
        new_encoded.append(&mut encoded[49..].to_vec());
        encoded = new_encoded;
    }
    let master_branch = DataBlob::new_branch(&encoded)?;
    let hash = &master_branch.hash();
    result.push(master_branch);
    Ok((hex_encode(hash), (total as f64) / 100000000.0, result))
}

#[derive(Debug, PartialEq)]
pub enum VError {
    NotLargestLibraryTx(String),
    MultipleOPReturns(String),
    NoFollowingHash(String),
    NoOPReturns(String),
    NotEnoughSpent(String),
    NotFullBranch(String),
    DataNotFound(String),
    ExpectedPage(String),
    MalformedData(String),
    TooManyBranches(i64),
    PageUnderMin(String)
}

impl VError {
    pub fn to_string(&self) -> String {
        match self {
            VError::NotLargestLibraryTx(txid) => format!("Txid({}): Was not the largest library transaction in the block", txid),
            VError::MultipleOPReturns(txid) => format!("Txid({}): Multiple outputs start with OP_Return", txid),
            VError::NoFollowingHash(txid) => format!("Txid({}): OP_Return not followed by 160 bit hash", txid),
            VError::NoOPReturns(txid) => format!("Txid({}): No outputs starting with OP_Return", txid),
            VError::NotEnoughSpent(txid) => format!("Txid({}): Amount spent on pages exceeded amount destroyed", txid),
            VError::PageUnderMin(txid) => format!("Txid({}): Contains a page with a fee that is under the minimum", txid),
            VError::NotFullBranch(hash) => format!("Hash({}): Found a branch that was not full, ignoring", hash),
            VError::DataNotFound(hash) => format!("Hash({}): No data found in mainline for hash", hash),
            VError::ExpectedPage(hash) => format!("Hash({}): Expected a page", hash),
            VError::MalformedData(hash) => format!("Hash({}): Contains malformed data", hash),
            VError::TooManyBranches(max_b) => format!("Contained more then the {} branch limit", max_b),
        }
    }

    pub fn print(&self) -> () {
        println!("[WARNING] {}", self.to_string());
    }
}

pub fn validate_page(hash: String, mainlinerpc: &MainlineRPC) -> Result<DataBlob, VError>{
    match mainlinerpc.get_blob(&hash) {
        Some(blob) => match blob.data[0] {
            0 => {
                if blob.data.len() < 10 {return Err(VError::MalformedData(hash));}
                Ok(blob)
            },
            _ => return Err(VError::ExpectedPage(hash))
        },
        None => return Err(VError::DataNotFound(hash))
    }
}

pub fn validate_branch(branch_hash: String, branch_index: i64, max_branch_index: i64, mainlinerpc: &MainlineRPC) -> Result<Vec<DataBlob>, VError> {
    match mainlinerpc.get_blob(&branch_hash) {
        Some(blob) => match blob.data[0] {
            1 => {
                if branch_index > max_branch_index {return Err(VError::TooManyBranches(max_branch_index));}
                let hashes = &blob.data[1..].chunks(20).map(|i| i.into()).collect::<Vec<Vec<u8>>>();
                if branch_index > 0 && hashes.len() != 49 {return Err(VError::NotFullBranch(branch_hash));}
                if !(hashes.len() > 0) {return Err(VError::MalformedData(branch_hash));}
                if hashes[0].len() != 20 {return Err(VError::MalformedData(branch_hash));}
                let mut blobs = match validate_branch(hex_encode(&hashes[0]), branch_index+1, max_branch_index, mainlinerpc) {
                    Err(err) => return Err(err),
                    Ok(blobs) => blobs
                };
                blobs.push(blob);
                for hash in &hashes[1..].to_vec() {
                    if hash.len() != 20 {return Err(VError::MalformedData(branch_hash));}
                    let pageblob = match validate_page(hex_encode(hash), mainlinerpc) {
                        Err(err) => return Err(err),
                        Ok(blob) => blob
                    };
                    blobs.push(pageblob);
                }
                Ok(blobs)
            },
            0 => match validate_page(branch_hash, mainlinerpc) {
                Err(err) => return Err(err),
                Ok(blob) => Ok(vec![blob])
            },
            _ => return Err(VError::MalformedData(branch_hash))
        },
        None => return Err(VError::DataNotFound(branch_hash))
    }
}

pub fn validate_utxo(config: &Config, utxo: &bitcoincore_rpc::json::ListUnspentResultEntry, watchonly_rpc: &Client, mainlinerpc: &MainlineRPC) -> Result<Option<(Vec<Page>, Vec<DataBlob>)>, Error> {
    let destroyed = utxo.amount.to_sat() as i64;
    let max_branch_index = (destroyed / config.min_page_n_value) / 49;
    let block_height = watchonly_rpc.get_block_count()?-utxo.confirmations as u64;
    let txinfo = watchonly_rpc.get_raw_transaction_info(&utxo.txid, None)?;
    dbg!(&txinfo);
    if txinfo.blockhash.is_none() {VError::MalformedData(utxo.txid.to_string()).print(); return Ok(None);}
    for tx in watchonly_rpc.get_block(&txinfo.blockhash.unwrap())?.txdata {
        if tx.txid() == utxo.txid {continue;}
        for txout in tx.output {
            match bitcoin::Address::from_script(bitcoin::ScriptBuf::from_hex(&txout.script_pubkey.to_hex_string())?.as_script(), bitcoin::Network::Bitcoin) {
                Ok(adr) => {
                    if adr.to_string() == config.address {
                        if txout.value >= utxo.amount.to_sat() {VError::NotLargestLibraryTx(utxo.txid.to_string()).print(); return Ok(None);}
                    }
                },
                _ => continue
            };
        }
    };
    let mut op_return_found = false;
    let mut hash = String::new();
    for txout in txinfo.vout {
        let hex = txout.script_pub_key.hex;
        if hex.len() > 0 && hex[0] == 0x6a {
            if op_return_found {VError::MultipleOPReturns(utxo.txid.to_string()).print(); return Ok(None);}
            op_return_found = true;
            if hex.len() != 22 {VError::NoFollowingHash(utxo.txid.to_string()).print(); return Ok(None);}
            hash = hex_encode(&hex[2..]);
        }
    }
    if !op_return_found {VError::NoOPReturns(utxo.txid.to_string()).print(); return Ok(None);}
    let blobs = match validate_branch(hash, 0, max_branch_index, mainlinerpc) {
        Err(verr) => {verr.print(); return Ok(None);},
        Ok(blobs) => blobs
    };
    let mut total = 0;
    let mut pages: Vec<Page> = vec![];
    for blob in &blobs {
        if blob.data[0] == 0 {
            let n_value = i64::from_le_bytes((&blob.data[1..9]).try_into().unwrap());
            if n_value < config.min_page_n_value {VError::PageUnderMin(utxo.txid.to_string()).print(); return Ok(None);}
            total += n_value;
            pages.push(Page::new(block_height, n_value, blob.data[9..].to_vec())?)
        }
    }
    if total > destroyed {VError::NotEnoughSpent(utxo.txid.to_string()).print(); return Ok(None);}
    Ok(Some((pages, blobs)))
}
