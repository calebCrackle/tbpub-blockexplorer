use serde::{Serialize, Deserialize};
use crate::Error;
use crate::hash;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct DataBlob {
    pub data: Vec<u8>
}

impl DataBlob {
    pub fn new(data: Vec<u8>) -> Result<DataBlob, Error> {
        if data.len() > 1000 {return Err(Error::MaxVectorSize(1000))}
        Ok(DataBlob{data: data})
    }

    pub fn from_page(page: UnpublishedPage) -> DataBlob {
        let mut result = DataBlob{data: vec![0]};
        result.data.append(&mut page.n_value.to_le_bytes().to_vec());
        result.data.append(&mut page.data.clone());
        result
    }

    pub fn new_branch(data: &Vec<DataBlob>) -> Result<DataBlob, Error> {
        if data.len() > 49 {return Err(Error::MaxVectorSize(49));}
        let mut result = DataBlob{data: vec![1]};
        for blob in data {
            result.data.append(&mut hash(&blob.data));
        }
        Ok(result)
    }

    pub fn hash(&self) -> Vec<u8> {
        hash(&self.data)
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    pub block_height: u64,
    pub n_value: i64,
    pub data: Vec<u8>,
}

impl Page {
    pub fn new(block_height: u64, n_value: i64, data: Vec<u8>) -> Result<Page, Error> {
        if data.len() > 991 {return Err(Error::MaxVectorSize(991));}
        Ok(Page{block_height: block_height, n_value: n_value, data: data})
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
pub struct UnpublishedPage {
    pub name: String,
    pub n_value: i64,
    pub data: Vec<u8>,
}

impl UnpublishedPage {
    pub fn new(name: String, n_value: i64, data: Vec<u8>) -> Result<UnpublishedPage, Error> {
        if data.len() > 991 {return Err(Error::MaxVectorSize(991));}
        Ok(UnpublishedPage{name: name, n_value: n_value, data: data})
    }
}
