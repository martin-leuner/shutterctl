pub struct Motor {
}

pub struct System {
    motors: Vec<Option<Motor>>,
}

impl System {
    pub fn from_config() -> anyhow::Result<Self> {
        let mut motors = Vec::<Option<Motor>>::new();
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "shutterctl") {
            let _cfgfile = proj_dirs.config_dir().join("config");
            // TODO: parse cfgfile as TOML, fill motors accordingly
        } else {
            // TODO: return some error
        }
        Ok(Self{motors})
    }
}
