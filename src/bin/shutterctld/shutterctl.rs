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
                        state: CurrentMove::Stopped,
                        known_min_percentage: 0,
                        known_max_percentage: 100,
                        last_stop: Some(std::time::Instant::now()),
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
                res.state.last_stop = None;
                Some(res)
            }
        }
    }

    pub fn get_state(&self) -> Vec<Motor> {
        self.motors.iter()
            .filter_map(Self::strip_state)
            .collect()
    }

    fn handle_drive_cmd(mot_state: &mut MotorState, cmd: &rpc::DriveInstruction) -> shutterproto::Result<()> {
        todo!()
    }

    pub fn drive(&mut self, instr: &Vec<rpc::DriveInstruction>) -> shutterproto::Result<()> {
        // Check whether any targeted motor is not present in the config
        if instr.iter().any(|x| self.motors.get(x.motor as usize).unwrap_or(&None).is_none()) {
            return Err(shutterproto::Error::InvalidMotorId);
        }
        // Pass motor's state along with the instructions into handler
        for cmd in instr {
            Self::handle_drive_cmd(&mut self.motors.get_mut(cmd.motor as usize).unwrap().as_mut().unwrap().state, &cmd)?;
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
