use std::io::Read;
use std::net::TcpListener;
use std::str;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());

                loop {
                    let mut buf = [0; 128];
                    let read_bytes = stream.read(&mut buf).unwrap();
                    println!("received bytes {}", read_bytes);

                    let s = str::from_utf8(&buf[0..read_bytes]).unwrap();
                    println!("{:?}", s);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    drop(listener);
}
