use crate::{Error, Config, DataBlob};
use crate::{hex_encode, hex_decode};



pub struct MainlineRPC {
    database: sqlite::Connection
}

impl MainlineRPC {
    pub fn new(config: &Config) -> Result<MainlineRPC, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("mainline.db");
        let database = sqlite::open(database_path)?;
        let query = "
        CREATE TABLE IF NOT EXISTS mainline (
            hash TEXT PRIMARY KEY,
            data TEXT 
        );
        ";
        database.execute(query)?;
        Ok(MainlineRPC{database: database})
    }
    
    pub fn add_blob(&self, blob: DataBlob) -> Result<DataBlob, Error> {
        self.database.execute("
        INSERT INTO mainline (hash, data)
        VALUES(".to_owned()+
        "'"+&hex_encode(&blob.hash())+"', "+
        "'"+&hex_encode(&blob.data)+"'"+
        ");")?;
        Ok(blob)
    }

    pub fn get_blob(&self, hash: &str) -> Option<DataBlob> {
        let query = "SELECT hash, data 
            FROM mainline WHERE hash = '".to_owned()+hash+"';";
        match self.database 
            .prepare(query)
            .unwrap()
            .into_iter()
            .next() {
                Some(row) => Some(DataBlob::new(
                    match hex_decode((row.as_ref().unwrap().read::<&str, _>("data")).to_string()) {
                        Err(_) => return None,
                        Ok(data) => data,
                    }
                ).unwrap()),
                None => None
            }
    }


 
}
