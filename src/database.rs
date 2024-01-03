use crate::{Config, Error};

pub struct SettingsDB {
    database: sqlite::Connection,
}

impl SettingsDB {
    pub fn new(config: &Config) -> Result<SettingsDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("settings.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS data (
            key TEXT PRIMARY KEY,
            value TEXT 
        );")?;
        Ok(SettingsDB{database: database})
    }
    
    pub fn set(&self, key: &str, value: &str) -> Result<(), Error> {
        self.database.execute("DELETE FROM data WHERE key = '".to_owned()+key+"';")?;
        self.database.execute("INSERT INTO data (key, value)
            VALUES(".to_owned()+
            "'"+key+"', "+
            "'"+value+"'"+
            ");")?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, Error> {
        let query = "SELECT key, value
            FROM data WHERE key = '".to_owned()+key+"';";
        match self.database 
            .prepare(query)
            .unwrap()
            .into_iter()
            .next() {
                Some(row) => Ok(Some((row.as_ref().unwrap().read::<&str, _>("value")).to_string())),
                None => Ok(None)
            }
    }
}
