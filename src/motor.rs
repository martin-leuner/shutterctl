use serde_derive::Deserialize;
use crate::shuttermsg::DriveCmdType;

pub struct Motor {
    pub config: MotorConfig,
    pub state: MotorState,
}

#[derive(Deserialize)]
pub struct MotorConfig {
    pub name: String,
    pub id: u8,
    pub runtime_ms: Option<u32>,
}

pub struct MotorState {
    pub state: CurrentMove,
    pub known_min_percentage: u8,
    pub known_max_percentage: u8,
    pub last_stop: Option<std::time::Instant>,
}

pub enum CurrentMove {
    Stopped,
    Up,
    Down,
}

impl TryFrom<DriveCmdType> for CurrentMove {
    type Error = crate::Error;

    fn try_from(t: DriveCmdType) -> crate::Result<Self> {
        match t {
            DriveCmdType::Stop => Ok(CurrentMove::Stopped),
            DriveCmdType::Up => Ok(CurrentMove::Up),
            DriveCmdType::Down => Ok(CurrentMove::Down),
            _ => Err(crate::Error::UnknownMotorState),
        }
    }
}
