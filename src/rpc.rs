use std::net::TcpStream;

use crate::shuttermsg::{CmdSystemState,
CmdSystemStateArgs,
Message,
Shuttermsg,
ShuttermsgArgs};
use crate::{Result, transport::Session};


fn command_message<'b, T>(fbb: &'b mut flatbuffers::FlatBufferBuilder,
                          msg_type: Message,
                          msg_data: flatbuffers::WIPOffset<T>) -> &'b [u8] {
    let msg = Shuttermsg::create(fbb,
                                 &ShuttermsgArgs{msg_type: msg_type,
                                 msg: Some(msg_data.as_union_value())});
    fbb.finish(msg, None);
    fbb.finished_data()
}

pub struct Conn<'a> {
    session: Session<'a>,
    fbb: flatbuffers::FlatBufferBuilder<'a>,
}

impl<'a> Conn<'a> {
    pub fn new(stream: &TcpStream) -> Result<Self> {
        let session = Session::new(stream)?;
        let fbb = flatbuffers::FlatBufferBuilder::new();
        Ok(Self{session, fbb})
    }

    pub fn get_state(&mut self) {
        self.fbb.reset();
        let data = CmdSystemState::create(&mut self.fbb, &CmdSystemStateArgs{});
        let cmd_buf = command_message(&mut self.fbb, Message::CmdSystemState, data);
        let _answ = self.session.exec_cmd(cmd_buf);

        // TODO: parse answ as Shuttermsg flatbuffer containing a RspSystemState
    }
}
