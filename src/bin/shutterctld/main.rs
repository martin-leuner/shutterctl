use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use shutterproto::rpc;
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

fn answer(stream: &TcpStream, sys: &Arc<Mutex<shutterctl::System>>) -> shutterproto::Result<()> {
    let mut sess = Session::new(stream)?;

    while socket_is_readable(stream) {
        let cmd_msg = sess.receive()?;
        match shutterctl::handle_cmd(&cmd_msg, &mut sys.lock().unwrap()) {
            Ok(answ) => {
                sess.send(&answ)?;
            },
            Err(e) => {
                let answ = rpc::build_status_answer(&Err(e))?;
                sess.send(&answ)?;
            },
        };
    }

    Ok(())
}

fn answer_log_err(stream: &TcpStream, sys: &Arc<Mutex<shutterctl::System>>) {
    if let Err(e) = answer(stream, sys) {
        // TODO: real logging
        eprintln!("Error while handling command: {e}");
    }
}

fn main() {
    // TODO: process arguments
    // TODO: set up logging
    // TODO: daemonize

    let motor_system = shutterctl::System::from_config().expect("Failed to parse config");
    let motor_system = Arc::new(Mutex::new(motor_system));

    // TODO: read port from config/command line
    let listener = TcpListener::bind("127.0.0.1:1337").expect("Failed to listen on port 1337");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let sys = Arc::clone(&motor_system);
            std::thread::spawn(move || {
                answer_log_err(&stream, &sys)
            });
        }
    }
}
