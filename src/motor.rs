use serde_derive::Deserialize;
use crate::shuttermsg::DriveCmdType;

use std::fmt;

#[derive(Clone)]
pub struct Motor {
    pub config: MotorConfig,
    pub state: MotorState,
}

#[derive(Clone, Deserialize)]
pub struct MotorConfig {
    pub name: String,
    pub id: u8,
    pub runtime_ms: Option<u32>,
}

#[derive(Clone)]
pub struct MotorState {
    pub state: CurrentMove,
    pub known_min_percentage: u8,
    pub known_max_percentage: u8,
    pub last_change: Option<std::time::Instant>,
}

#[derive(Copy, Clone, Debug)]
pub enum CurrentMove {
    Stop,
    Up,
    Down,
}

impl fmt::Display for CurrentMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<DriveCmdType> for CurrentMove {
    type Error = crate::Error;

    fn try_from(t: DriveCmdType) -> crate::Result<Self> {
        match t {
            DriveCmdType::Stop => Ok(CurrentMove::Stop),
            DriveCmdType::Up => Ok(CurrentMove::Up),
            DriveCmdType::Down => Ok(CurrentMove::Down),
            _ => Err(crate::Error::UnknownMotorState),
        }
    }
}
