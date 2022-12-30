use std::net::TcpStream;

use shutterproto::rpc;


fn main() {
    let stream = TcpStream::connect("127.0.0.1:1337").expect("Failed to open connection to port 1337");
    let mut shutters = rpc::Conn::new(&stream).expect("Failed to create RPC connection object");
    // TODO: better error handling
    let state = shutters.get_state().expect("Error during `get_state` call");

    println!("Connected to shutter control daemon. State:");
    for m in state.iter() {
        println!(
            "{name:>32}: {state} ({min}%-{max}%)",
            name = m.config.name,
            state = m.state.state,
            min = m.state.known_min_percentage,
            max = m.state.known_max_percentage);
    }
}
