use crate::{Error, Config};

pub struct SettingsDB {
    database: sqlite::Connection,
}

impl SettingsDB {
    pub fn new(config: &Config) -> Result<SettingsDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("settings.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT 
        );")?;
        Ok(SettingsDB{database: database})
    }
    
    pub fn set(&self, key: &str, value: &str) -> Result<(), Error> {
        self.database.execute(format!("DELETE FROM settings WHERE key = '{}';", key))?;
        self.database.execute(format!("INSERT INTO settings (key, value)
        VALUES('{}', '{}');",
        key, value))?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, Error> {
        match self.database 
            .prepare(format!("SELECT key, value
            FROM settings WHERE key = '{}';",
            key))?.into_iter().next() {
                Some(row) => Ok(Some((row.as_ref().unwrap().read::<&str, _>("value")).to_string())),
                None => Ok(None)
            }
    }
}

pub struct HashesDB {
    database: sqlite::Connection,
}

impl HashesDB {
    pub fn new(config: &Config) -> Result<HashesDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("hashes.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS hashes (
            hash TEXT PRIMARY KEY,
            block_height INT,
            price INT
        );")?;
        Ok(HashesDB{database: database})
    }
    
    pub fn add(&self, hash: &str, block_height: u64, price: u64) -> Result<(), Error> {
      if self.database 
          .prepare(format!("SELECT hash, block_height, price
          FROM hashes WHERE hash = '{}';",
          hash))?.into_iter().next().is_some() {return Ok(());}
        Ok(self.database.execute(format!("
        INSERT INTO hashes (hash, block_height, price)
        VALUES('{}', {}, {});",
        hash, block_height, price))?)
    }
}

pub struct RootDIDsDB {
    database: sqlite::Connection,
}

impl RootDIDsDB {
    pub fn new(config: &Config) -> Result<RootDIDsDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("rootdids.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS rootdids (
            did TEXT PRIMARY KEY,
            block_height INT,
            price INT
        );")?;
        Ok(RootDIDsDB{database: database})
    }
    
    pub fn add(&self, did: &str, block_height: u64, price: u64) -> Result<(), Error> {
      if self.database 
          .prepare(format!("SELECT did, block_height, price
          FROM rootdids WHERE did = '{}';",
          did))?.into_iter().next().is_some() {return Ok(());}
        Ok(self.database.execute(format!("
        INSERT INTO rootdids (did, block_height, price)
        VALUES('{}', {}, {});",
        did, block_height, price))?)
    }
}
