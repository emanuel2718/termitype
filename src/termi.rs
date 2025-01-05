use anyhow::Result;

use crate::{config::Config, tracker::Tracker};

#[derive(Debug)]
pub struct Termi {
    pub config: Config,
    pub tracker: Tracker,
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        let tracker = Tracker::new(&config);
        Termi {
            config: config.clone(),
            tracker,
        }
    }
}

pub fn run(config: &Config) -> Result<()> {
    let termi = Termi::new(&config);
    println!("Running Termitype with config: {:?}", config);
    dbg!(termi);

    Ok(())
}
