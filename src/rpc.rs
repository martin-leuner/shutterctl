use std::net::TcpStream;

use crate::shuttermsg::{self,
                        CmdSystemState,
                        CmdSystemStateArgs,
                        DriveCmdType,
                        Message,
                        Shuttermsg,
                        ShuttermsgArgs};
use crate::{Error, Result, motor, transport::Session};


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

    pub fn get_state(&mut self) -> Result<Vec<motor::Motor>> {
        self.fbb.reset();
        let data = CmdSystemState::create(&mut self.fbb, &CmdSystemStateArgs{});
        let cmd_buf = command_message(&mut self.fbb, Message::CmdSystemState, data);

        let answ = self.session.exec_cmd(cmd_buf)?;

        let state = shuttermsg::root_as_shuttermsg(&answ)?;
        let state = state.msg_as_rsp_system_state().ok_or(Error::BadAnswer)?;
        let state = state.shutters().ok_or(Error::BadAnswer)?;

        let mut res = Vec::<motor::Motor>::new();
        for m in state.iter() {
            res.push(motor::Motor{
                config: motor::MotorConfig{
                    name: m.description().unwrap_or_default().to_string(),
                    id: m.id(),
                    runtime_ms: None,
                },
                state: motor::MotorState{
                    state: match m.moving() {
                        DriveCmdType::Stop => motor::CurrentMove::Stopped,
                        DriveCmdType::Up => motor::CurrentMove::Up,
                        DriveCmdType::Down => motor::CurrentMove::Down,
                        _ => {
                            return Err(Error::UnknownMotorState)
                        }
                    },
                    known_min_percentage: m.known_min_percentage(),
                    known_max_percentage: m.known_max_percentage(),
                    last_stop: None,
                }});
        }

        Ok(res)
    }
}
