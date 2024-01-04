use crate::Error;

pub fn get_string(value: &serde_json::Value) -> Result<String, Error> {
    if !value.is_string() {return Err(Error::NotString());}
    Ok(value.as_str().unwrap().to_string()) 
}

pub fn get_i64(value: &serde_json::Value) -> Result<i64, Error> {
    if !value.is_i64() {return Err(Error::NotI64());}
    Ok(value.as_i64().unwrap()) 
}

pub fn get_vec_u8(value: &serde_json::Value) -> Result<Vec<u8>, Error> {
    if !value.is_array() {return Err(Error::NotVector());}
    value.as_array().unwrap().iter().map(|chr| {
        if !chr.is_u64() {return Err(Error::NotU64());}
        Ok(chr.as_u64().unwrap() as u8)
    }).collect()
}
