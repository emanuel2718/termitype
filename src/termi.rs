use anyhow::Result;

use crate::config::Config;

pub struct Termi {
    pub config: Config,
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        Termi {
            config: config.clone(),
        }
    }
}

pub fn run(config: &Config) -> Result<()> {
    println!("Running Termitype with config: {:?}", config);

    Ok(())
}
