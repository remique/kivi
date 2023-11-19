use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:7878");
    let args = std::env::args().collect::<Vec<String>>();

    // Skip first argument
    let s = args[1..].join(" ");

    match conn {
        Ok(mut stream) => {
            println!("Connected to server");

            stream.write_all(s.as_bytes()).unwrap();

            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).expect("failed to read to end");
            println!("Response: {:?}", str::from_utf8(&buf).unwrap());
        }
        Err(e) => {
            println!("Could not connect: {}", e);
        }
    }
}
