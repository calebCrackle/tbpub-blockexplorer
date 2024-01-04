use crate::{hex_decode, hex_encode};
use crate::{Error, Config, Page, UnpublishedPage, DataBlob};

pub struct DataDB {
    database: sqlite::Connection,
}

impl DataDB {
    pub fn new(config: &Config) -> Result<DataDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("data.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS data (
            key TEXT PRIMARY KEY,
            value TEXT 
        );")?;
        Ok(DataDB{database: database})
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

pub struct UnpublishedPagesDB {
    database: sqlite::Connection,
}

impl UnpublishedPagesDB {
    pub fn new(config: &Config) -> Result<UnpublishedPagesDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("unpublished_pages.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS pages (
            name TEXT PRIMARY KEY,
            n_value INT,
            data TEXT 
        );")?;
        Ok(UnpublishedPagesDB{database: database})
    }
    
    pub fn add_page(&self, page: UnpublishedPage) -> Result<UnpublishedPage, Error> {
        self.database.execute("
        INSERT INTO pages (name, n_value, data)
        VALUES(".to_owned()+
        "'"+&page.name+"', "+
        &page.n_value.to_string()+", "+
        "'"+&hex_encode(&page.data)+"'"+
        ");")?;
        Ok(page)
    }
    
    pub fn get_page(&self, pagename: &str) -> Result<Option<UnpublishedPage>, Error>{
        let query = "SELECT name, n_value, data 
            FROM pages WHERE name = '".to_owned()+pagename+"';";
        match self.database 
            .prepare(query)
            .unwrap()
            .into_iter()
            .next() {
                Some(row) => Ok(Some(UnpublishedPage::new(
                    (row.as_ref().unwrap().read::<&str, _>("name")).to_string(),
                    (row.as_ref().unwrap().read::<i64, _>("n_value")).to_owned(),
                    hex_decode((row.as_ref().unwrap().read::<&str, _>("data")).to_string())?
                )?)),
                None => Ok(None)
            }
    }

    pub fn remove_page(&self, pagename: &str) -> Result<Option<()>, Error> {
        match self.get_page(pagename)? {
            Some(_) => self.database.execute("DELETE FROM pages 
                WHERE name = '".to_owned()+pagename+"';")?,
            None => return Ok(None)
        
        };
        Ok(Some(()))
    }
    
    pub fn get_pages(&self) -> Result<Vec<UnpublishedPage>, Error> {
        let query = "SELECT name, n_value, data FROM pages";
        self.database 
            .prepare(query)
            .unwrap()
            .into_iter()
            .map(|row| {
                Ok(UnpublishedPage::new(
                    (row.as_ref().unwrap().read::<&str, _>("name")).to_string(),
                    (row.as_ref().unwrap().read::<i64, _>("n_value")).to_owned(),
                    hex_decode((row.as_ref().unwrap().read::<&str, _>("data")).to_string())?
                )?)
            }).collect()
    }

    pub fn remove_pages(&self) -> Result<(), Error> {
        self.database.execute("DELETE FROM pages;")?; 
        Ok(())
    }
}

pub struct RepublisherBlobsDB {
    database: sqlite::Connection,
}

impl RepublisherBlobsDB {
    pub fn new(config: &Config) -> Result<RepublisherBlobsDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("republisher.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS blobs (
            data TEXT PRIMARY KEY
        );")?;
        Ok(RepublisherBlobsDB{database: database})
    }
    
    pub fn add_blob(&self, blob: DataBlob) -> Result<DataBlob, Error> {
        match self.database.execute("
        INSERT INTO blobs (data)
        VALUES(".to_owned()+
        "'"+&hex_encode(&blob.data)+"'"+
        ");") {
            Ok(_) => (),
            Err(err) if err.code.is_some() && err.code.unwrap() == 19 =>(), 
            Err(e) => return Err(Error::SQLiteError(e))
        };

        Ok(blob)
    }
  
    pub fn get_blobs(&self) -> Result<Vec<DataBlob>, Error> {
        let query = "SELECT data FROM blobs";
        self.database 
            .prepare(query)
            .unwrap()
            .into_iter()
            .map(|row| {
                Ok(DataBlob::new(
                    hex_decode((row.as_ref().unwrap().read::<&str, _>("data")).to_string())?
                )?)
            }).collect()
    }
}

pub struct LibraryDB {
    database: sqlite::Connection,
}

impl LibraryDB {
    pub fn new(config: &Config) -> Result<LibraryDB, Error> {
        let mut database_path = config.datadir.clone();
        database_path.push("library.db");
        let database = sqlite::open(database_path)?;
        database.execute("
        CREATE TABLE IF NOT EXISTS library (
            data TEXT PRIMARY KEY,
            block_height INT,
            n_value INT
        );")?;
        Ok(LibraryDB{database: database})
    }

    pub fn add_page(&self, page: Page) -> Result<Page, Error> {
        match self.database.execute("
        INSERT INTO library (data, block_height, n_value)
        VALUES(".to_owned()+
        "'"+&hex_encode(&page.data)+"', "+
        &page.block_height.to_string()+", "+
        &page.n_value.to_string()+""+
        ");") {
            Ok(_) => (),
            Err(err) if err.code.is_some() && err.code.unwrap() == 19 => (), 
            Err(e) => return Err(Error::SQLiteError(e))
        };
        Ok(page)
    }
    
    pub fn get_pages(&self) -> Result<Vec<Page>, Error> {
        let query = "SELECT data, block_height, n_value FROM library";
        self.database 
            .prepare(query)
            .unwrap()
            .into_iter()
            .map(|row| {
                Ok(Page::new(
                    (row.as_ref().unwrap().read::<i64, _>("block_height")).to_owned() as u64,
                    (row.as_ref().unwrap().read::<i64, _>("n_value")).to_owned(),
                    hex_decode((row.as_ref().unwrap().read::<&str, _>("data")).to_string())?
                )?)
            }).collect()
    }
}
