use serde_derive::Deserialize;
use shutterproto::motor::{Motor, MotorConfig, MotorState, CurrentMove};
use shutterproto::rpc;

#[derive(Deserialize)]
struct Config {
    motor: Vec<MotorConfig>,
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
            motors.resize_with(max_id.into(), || None);

            for motor in config.motor {
                let id = motor.id;
                motors[Into::<usize>::into(id) - 1] = Some(Motor{
                    config: motor,
                    state: MotorState{
                        state: CurrentMove::Stop,
                        known_min_percentage: 0,
                        known_max_percentage: 100,
                        last_change: Some(std::time::Instant::now()),
                    }});
            }
            Ok(Self{motors})
        } else {
            // TODO: logging
            Err(anyhow::anyhow!("No config file found"))
        }
    }

    fn strip_state(m: &Option<Motor>) -> Option<Motor> {
        match m {
            None => None,
            Some(m) => {
                let mut res = m.clone();
                res.config.runtime_ms = None;
                res.state.last_change = None;
                Some(res)
            }
        }
    }

    pub fn get_state(&self) -> Vec<Motor> {
        self.motors.iter()
            .filter_map(Self::strip_state)
            .collect()
    }

    fn recalc_position(state: &MotorState, runtime: u32) -> (u8, u8) {
        let ms_elapsed = state.last_change
            .unwrap_or(std::time::Instant::now())
            .elapsed()
            .as_millis();
        let ms_elapsed: u32 = ms_elapsed.try_into().unwrap_or(u32::MAX);
        let runtime = std::cmp::max(runtime, 1);
        let percentage_change: u8 = (ms_elapsed.saturating_mul(100)/runtime)
            .try_into().unwrap_or(100);

        let mut new_min = state.known_min_percentage;
        let mut new_max = state.known_max_percentage;
        match state.state {
            CurrentMove::Stop => { }
            CurrentMove::Up => {
                new_min = std::cmp::min(new_min.saturating_add(percentage_change), 100);
                if new_min > new_max {
                    new_max = new_min;
                }
            }
            CurrentMove::Down => {
                new_max = new_max.saturating_sub(percentage_change);
                if new_max < new_min {
                    new_min = new_max;
                }
            }
        }
        (new_min, new_max)
    }

    fn handle_drive_cmd(state: &mut MotorState,
                        runtime: u32,
                        cmd: &rpc::DriveInstruction) -> shutterproto::Result<()> {
        match cmd.movement {
            Some(mv) => {
                match mv {
                    CurrentMove::Stop => {
                        // TODO: actually stop motor
                        // TODO: recalculate min/max percentage
                        state.last_change = Some(std::time::Instant::now());
                    },
                    CurrentMove::Up => {
                        // TODO
                    },
                    CurrentMove::Down => {
                        // TODO
                    },
                }
                state.state = mv;
            },
            None => {
                if cmd.target_percentage.is_none() {
                    return Err(shutterproto::Error::BadCommand);
                }
                // TODO
            },
        }
        Ok(())
    }

    pub fn drive(&mut self, instr: &Vec<rpc::DriveInstruction>) -> shutterproto::Result<()> {
        // Check whether any targeted motor is not present in the config
        if instr.iter().any(|x| self.motors.get(x.motor as usize).unwrap_or(&None).is_none()) {
            return Err(shutterproto::Error::InvalidMotorId);
        }
        // Pass motor's state along with the instructions into handler
        for cmd in instr {
            let motor = self.motors.get_mut(cmd.motor as usize).unwrap().as_mut().unwrap();
            // If runtime is not set (which shouldn't ever happen), use a default guesstimate
            Self::handle_drive_cmd(&mut motor.state, motor.config.runtime_ms.unwrap_or(30000), &cmd)?;
        }
        Ok(())
    }
}

pub fn handle_cmd(cmd_msg: &[u8], sys: &mut System) -> shutterproto::Result<Vec<u8>> {
    let cmd = rpc::parse_cmd(cmd_msg)?;
    match cmd.cmd {
        rpc::Command::GetState => rpc::build_get_state_answer(&sys.get_state()),
        rpc::Command::Drive => rpc::build_status_answer(&sys.drive(&cmd.instructions)),
    }
}
