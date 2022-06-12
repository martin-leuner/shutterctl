use std::net::TcpStream;

use shutterproto::rpc;


fn main() {
    let stream = TcpStream::connect("127.0.0.1:1337").expect("Failed to open connection to port 1337");
    let mut shutters = rpc::Conn::new(&stream).expect("Failed to create RPC connection object");
    shutters.get_state();
}
