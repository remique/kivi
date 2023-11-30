use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;

use kivi::core::kv::KiviStore;

fn initialize_logger() {
    let env = env_logger::Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

enum Command {
    Set,
    Get,
    Invalid,
}

impl Command {
    fn which_command(input: &str) -> Command {
        match input.as_bytes() {
            b"set" => Command::Set,
            b"get" => Command::Get,
            _ => Command::Invalid,
        }
    }
}

// This is probably not the best function, but should work
fn stream_to_vec(buf: &[u8]) -> Vec<String> {
    let s = str::from_utf8(buf).expect("Could not from utf8");

    let v = s.split(" ").collect::<Vec<&str>>();

    v.iter().map(|x| x.to_string()).collect::<Vec<String>>()
}

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 1024];

    let read_bytes = stream.read(&mut buf).unwrap();

    let vec = stream_to_vec(&buf[0..read_bytes]);
    println!("As vec: {:?}", vec);
    let command = Command::which_command(vec[0].as_str());

    match command {
        Command::Get => {
            println!("Get command with arg: {}", vec[1].as_str());
        }
        Command::Set => {
            println!(
                "Set command with key: {}, value: {}",
                vec[1].as_str(),
                vec[2].as_str()
            );
        }
        Command::Invalid => {
            println!("Unrecognized command");
        }
    }

    let s = str::from_utf8(&buf[0..read_bytes]).unwrap();
    println!("{:?}", s);

    // Respond
    stream
        .write_all("Received: {}, response: OK".as_bytes())
        .expect("Could not respond");
}

fn main() {
    initialize_logger();

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    log::debug!("Server listening on port 7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    drop(listener);
}
