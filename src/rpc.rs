use std::net::TcpStream;

use crate::motor::{Motor,
                   MotorConfig,
                   MotorState,
                   CurrentMove};
use crate::shuttermsg::{self,
                        CmdSystemState,
                        CmdSystemStateArgs,
                        DriveCmdType,
                        Message,
                        Shuttermsg,
                        ShuttermsgArgs};
use crate::{Error, Result, transport::Session};


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

    pub fn get_state(&mut self) -> Result<Vec<Motor>> {
        self.fbb.reset();
        let data = CmdSystemState::create(&mut self.fbb, &CmdSystemStateArgs{});
        let cmd_buf = command_message(&mut self.fbb, Message::CmdSystemState, data);

        let answ = self.session.exec_cmd(cmd_buf)?;

        let state = shuttermsg::root_as_shuttermsg(&answ)?;
        let state = state.msg_as_rsp_system_state().ok_or(Error::BadAnswer)?;
        let state = state.shutters().ok_or(Error::BadAnswer)?;

        let mut res = Vec::<Motor>::new();
        for m in state.iter() {
            res.push(Motor{
                config: MotorConfig{
                    name: m.description().unwrap_or_default().to_string(),
                    id: m.id(),
                    runtime_ms: None,
                },
                state: MotorState{
                    state: CurrentMove::try_from(m.moving())?,
                    known_min_percentage: m.known_min_percentage(),
                    known_max_percentage: m.known_max_percentage(),
                    last_stop: None,
                }});
        }

        Ok(res)
    }
}

pub enum Command {
    GetState,
    Drive,
}

pub struct CommandData {
    pub cmd: Command,
    pub instructions: Vec<DriveInstruction>,
}

pub struct DriveInstruction {
    pub motor: u8,
    pub movement: Option<CurrentMove>,
    pub target_percentage: Option<u8>,
}

pub fn parse_cmd(cmd_msg: &[u8]) -> Result<CommandData> {
    let msg = shuttermsg::root_as_shuttermsg(cmd_msg)?;
    match msg.msg_type() {
        Message::CmdDrive => {
            let cmd_data = msg.msg_as_cmd_drive().unwrap();
            let cmd_data = cmd_data.motors().ok_or(Error::BadCommand)?;

            let mut instr = Vec::<DriveInstruction>::new();
            for m in cmd_data.iter() {
                if m.cmd() == DriveCmdType::TargetPercentage {
                    instr.push(DriveInstruction{
                        motor: m.motor(),
                        movement: None,
                        target_percentage: Some(m.target_percentage()),
                    });
                } else {
                    instr.push(DriveInstruction{
                        motor: m.motor(),
                        movement: Some(CurrentMove::try_from(m.cmd())?),
                        target_percentage: None,
                    });
                }
            }

            Ok(CommandData{
                cmd: Command::Drive,
                instructions: instr,
            })
        }
        Message::CmdSystemState => {
            Ok(CommandData{
                cmd: Command::GetState,
                instructions: Vec::new(),
            })
        }
        _ => {
            Err(Error::UnknownCommand)
        }
    }
}
