use std::net::{TcpListener, TcpStream};

fn main() {
    // TODO: process arguments
    // TODO: set up logging
    // TODO: daemonize

    let listener = TcpListener::bind("127.0.0.1:1337").expect("Failed to listen on port 1337");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established!");
    }
}
