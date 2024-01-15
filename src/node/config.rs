use crate::error::Error;
use std::env;
use std::path::PathBuf;
use std::fs::{read_to_string, create_dir_all};

#[derive(Debug, Clone)]
pub struct Config {
    pub datadir: PathBuf,
    pub cliurl: String,
    pub rpcurl: String,
    pub rpcpassword: String,
    pub rpcuser: String,
    pub wallet: String,
}

impl Config {
    fn parse_args(&mut self, args: Vec<String>) -> Result<(), Error> {
        for arg in args {
            let argp: Vec<&str> = arg.split("=").collect();
            if argp.len() != 2 {return Err(Error::NodeHelpMessage());}
            let mut key: String = argp[0].to_string();
            let value: String = argp[1].to_string();
            if argp[0].chars().next().unwrap() == '-' {key = key[1..].to_string()};
            match key.as_str() {
                "datadir" => self.datadir = PathBuf::from(value),
                "cliurl" => self.rpcuser = value,
                "rpcurl" => self.rpcurl = value,
                "rpcpassword" => self.rpcpassword = value,
                "rpcuser" => self.rpcuser = value,
                "wallet" => self.wallet = value,
                _ => return Err(Error::UnknownArgument(key)),
            }
        }
        Ok(())
    }

    pub fn new() -> Result<Config, Error> {
        let mut path = home::home_dir().ok_or(Error::NoHomeDir())?;
        path.push(".tbpub");
        let mut config = Config{
            datadir: path,
            cliurl: "127.0.0.1:9443".to_string(),
            rpcurl: "http://localhost:8332".to_string(), 
            rpcpassword: "".to_string(), 
            rpcuser: "".to_string(),
            wallet: "".to_string(),
        };
        create_dir_all(&config.datadir)?;

        // Get ENV Args
        let args: Vec<String> = env::args().collect();

        //Parse ENV Args
        config.parse_args(args[1..args.len()].iter().cloned().collect())?;

        //Parse Config File
        let mut config_file = config.datadir.clone();
        config_file.push("lipnode.conf");
        if config_file.exists() {
            let _ = config.parse_args(read_to_string(config_file
                    .as_os_str()
                    .to_str()
                    .unwrap())?
                .lines()
                .map(String::from)
                .collect());
        }

        //Parse ENV Args A second time to overwrite config
        config.parse_args(args[1..args.len()].iter().cloned().collect())?;
        if config.wallet == "" {return Err(Error::NoWallet());}
        Ok(config)
    }
}
