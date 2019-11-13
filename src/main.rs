extern crate env_logger;
extern crate log;

use listdb_engine::dbprocess::DBResponse::*;
use listdb_engine::DBEngine;
use log::{debug, info};
use properties::Properties;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;

mod properties;

const SERVER_PORT_PROPERTY: &str = "server.port";
const DATA_HOME_PROPERTY: &str = "data.home";
const PROPERTY_FILE: &str = "listdb.properties";

fn handle_client(mut stream: TcpStream, db_home: &str) -> Result<(), Error> {
    let mut db_engine = DBEngine::new(&db_home);
    info!("Incomming connection from: {}", stream.peer_addr()?);
    let mut buffer = [0; 512];
    let mut command = String::new();
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }
        let input = str::from_utf8(&buffer[..bytes_read]).unwrap();
        command.push_str(input);
        if input.contains("\n") {
            debug!("rcvd: {}", input);
            match db_engine.request(command.trim()) {
                Unknown => stream.write("e:unknown request\n".as_bytes())?,
                ROk(_) => stream.write("a\n".as_bytes())?,
                OpenContext(message) => {
                    let response = format!("c:{}\n", message);
                    stream.write(response.as_bytes())?
                }
                Data(values) => {
                    //let mut data_buffer = [0; 512];
                    //write_data(message, &mut data_buffer);
                    //stream.write(&data_buffer)?
                    let response = format_data(values);
                    stream.write(response.as_bytes())?
                }
                Invalid(message) => {
                    let response = format!("e:{}\n", message);
                    stream.write(response.as_bytes())?
                }
                Error(message) => {
                    let response = format!("x:{}\n", message);
                    stream.write(response.as_bytes())?
                }
                _ => stream.write(b"not implemented\n")?,
            };
            command.clear();
        }
    }
}

fn main() {
    env_logger::init();
    let mut properties = Properties::new();
    properties.load(PROPERTY_FILE);
    let port = properties.get(SERVER_PORT_PROPERTY);
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(bind_addr).expect("Could not bind");
    info!("Server listenting on port: {}", port);
    for stream in listener.incoming() {
        let db_home = properties.get(DATA_HOME_PROPERTY);
        match stream {
            Err(e) => eprintln!("failed: {}", e),
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream, &db_home).unwrap_or_else(|error| eprintln!("{:?}", error))
                });
            }
        }
    }
}

fn format_data(data: Vec<(String, String)>) -> String {
    let count = format!("c{}:", data.len());
    let mut sizes = "s".to_string();
    let mut values = String::new();

    for (key, value) in data {
        //Temporary fix. Should be handled better
        let temp_key = if key.len() == 36 {
            &key
        } else {
            "                                    "
        };
        let total = temp_key.len() + &value.len();
        sizes.push_str(&total.to_string());
        sizes.push(':');
        values.push_str(temp_key);
        values.push_str(&value.to_string());
    }

    format!("d{}{}{}\n", count, sizes, values)
}
