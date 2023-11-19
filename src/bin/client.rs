use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:7878");

    match conn {
        Ok(mut stream) => {
            println!("Connected to server");
            loop {
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).unwrap();

                if s.is_empty() {
                    break;
                }
                stream.write_all(s.as_bytes()).unwrap();
            }
        }
        Err(e) => {
            println!("Could not connect: {}", e);
        }
    }
}
