use std::net::TcpStream;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:7878");

    match conn {
        Ok(mut stream) => {
            println!("Connected to server");
        }
        Err(e) => {
            println!("Could not connect: {}", e);
        }
    }
}
