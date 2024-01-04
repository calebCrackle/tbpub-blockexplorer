mod error;
use crate::error::Error;

use serde::{Serialize, Deserialize};

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

fn deserialize<'a, T: serde::Deserialize<'a>>(data: &'a String) -> Result<T, Error> {
    let response: T = serde_json::from_str(&data)?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRequest{
    pub method: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonResponse{
    pub status: u8,
    pub message: String,
}

fn main() -> Result<(), Error> {
    let mut cliurl = "127.0.0.1:9443".to_string();
    let args: Vec<String> = env::args().collect();
    
    //Assert at least one argument
    if args.len() < 2 {return Err(Error::TooFewArgs());}

    let mut arg_index = 1;

    //Check for -cliurl option
    let argp: Vec<&str> = args[arg_index].split("=").collect();
    if argp.len() == 2 && argp[0] == "-cliurl" {
        cliurl = argp[1].to_string();
        arg_index += 1;
    }
    //Assert at least one method
    if args.len() < arg_index+1 {return Err(Error::TooFewArgs());}

    let mut stream = TcpStream::connect(&cliurl)?;
    let request = JsonRequest{ method: (&args[arg_index]).to_string(), args: args[arg_index+1..args.len()].iter().cloned().collect()};
    stream.write(serde_json::to_string(&request)?.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut data = String::new();
    stream.read_to_string(&mut data)?;
    if data == "" {return Err(Error::UnexpectedShutdown());}
    match deserialize::<JsonResponse>(&data) {
        Ok(response) if response.status == 1 => println!("{}", response.message),
        Ok(response) if response.status == 2 => println!("{}", response.message),
        Ok(response) => println!("[ERROR]: {}", response.message),
        Err(_) => return Err(Error::MalformedStream(data)),
    };
    Ok(())
}
