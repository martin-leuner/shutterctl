use serde_derive::Deserialize;

pub struct Motor {
    config: MotorConfig,
    state: MotorState,
}

#[derive(Deserialize)]
pub struct MotorConfig {
    name: String,
    id: usize,
    runtime_ms: u32,
}

#[derive(Deserialize)]
struct Config {
    motor: Vec<MotorConfig>,
}

pub struct MotorState {
    state: CurrentMove,
    known_min_percentage: u8,
    known_max_percentage: u8,
    last_stop: std::time::Instant,
}

pub enum CurrentMove {
    Stopped,
    Up,
    Down,
}

pub struct System {
    motors: Vec<Option<Motor>>,
}

impl System {
    pub fn from_config() -> anyhow::Result<Self> {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "shutterctl") {
            let cfgfile = proj_dirs.config_dir().join("config");
            let cfgfile = std::fs::read_to_string(cfgfile)?;
            let config: Config = toml::from_str(&cfgfile)?;

            let max_id = config.motor.iter().max_by_key(|x| x.id).unwrap().id;
            let mut motors = Vec::<Option<Motor>>::new();
            motors.resize_with(max_id, || None);

            for motor in config.motor {
                let id = motor.id;
                motors[id] = Some(Motor{
                    config: motor,
                    state: MotorState{
                        state: CurrentMove::Stopped,
                        known_min_percentage: 0,
                        known_max_percentage: 100,
                        last_stop: std::time::Instant::now(),
                    }});
            }
            Ok(Self{motors})
        } else {
            // TODO: logging
            Err(anyhow::anyhow!("No config file found"))
        }
    }
}
