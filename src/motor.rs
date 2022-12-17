use serde_derive::Deserialize;

pub struct Motor {
    pub config: MotorConfig,
    pub state: MotorState,
}

#[derive(Deserialize)]
pub struct MotorConfig {
    pub name: String,
    pub id: usize,
    pub runtime_ms: u32,
}

pub struct MotorState {
    pub state: CurrentMove,
    pub known_min_percentage: u8,
    pub known_max_percentage: u8,
    pub last_stop: std::time::Instant,
}

pub enum CurrentMove {
    Stopped,
    Up,
    Down,
}
