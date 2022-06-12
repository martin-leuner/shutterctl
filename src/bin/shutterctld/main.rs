use std::net::{TcpListener, TcpStream};

use shutterproto::transport::Session;

pub mod shutterctl;


fn socket_is_readable(stream: &TcpStream) -> bool {
    let mut dummy_buf = [0u8; 1];
    if let Ok(size) = stream.peek(&mut dummy_buf) {
        // Successful read of size 0 means socket is closed
        size > 0
    } else {
        false
    }
}

fn answer(stream: &TcpStream) -> shutterproto::Result<()> {
    let mut sess = Session::new(stream)?;

    while socket_is_readable(stream) {
        let _cmd_msg = sess.receive()?;
        // TODO: handle command, build answer message
        sess.send(&[])?;
    }

    Ok(())
}

fn answer_log_err(stream: &TcpStream) {
    if let Err(_e) = answer(stream) {
        // TODO: log error
    }
}

fn main() {
    // TODO: process arguments
    // TODO: set up logging
    // TODO: daemonize

    let listener = TcpListener::bind("127.0.0.1:1337").expect("Failed to listen on port 1337");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            std::thread::spawn(move || {
                answer_log_err(&stream)
            });
        }
    }
}
