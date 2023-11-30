use std::io::Read;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::str;

use crate::core::{error::Result, kv::KiviStore};

enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Invalid,
}

pub struct KiviServer {
    engine: KiviStore,
}

impl KiviServer {
    pub fn new() -> Result<Self> {
        let engine = KiviStore::new()?;

        Ok(Self { engine })
    }

    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    if let Err(e) = self.serve(&mut s) {
                        log::error!("Error: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn serve(&mut self, stream: &mut TcpStream) -> Result<()> {
        // Buffer
        let mut buf = [0; 1024];
        let bytes_read = stream.read(&mut buf)?;

        let command = Command::get(&buf[0..bytes_read]);

        match command {
            Command::Get { key } => {
                println!("Get");
                let res = self.engine.get(key);
            }
            Command::Set { key, value } => {
                //
            }
            Command::Invalid => {
                //
            }
        }

        Ok(())
    }
}

impl Command {
    fn get(buf: &[u8]) -> Command {
        let as_vec = stream_to_vec(buf);

        // TODO: Do it clean way
        match as_vec[0].as_bytes() {
            b"set" => {
                if as_vec.len() != 3 {
                    return Command::Invalid;
                }

                return Command::Set {
                    key: as_vec[1].clone(),
                    value: as_vec[2].clone(),
                };
            }
            b"get" => {
                if as_vec.len() != 2 {
                    return Command::Invalid;
                }

                return Command::Get {
                    key: as_vec[1].clone(),
                };
            }
            _ => Command::Invalid,
        }
    }
}

fn stream_to_vec(buf: &[u8]) -> Vec<String> {
    let s = str::from_utf8(buf).expect("Could not from utf8");

    let v = s.split(" ").collect::<Vec<&str>>();

    v.iter().map(|x| x.to_string()).collect::<Vec<String>>()
}
