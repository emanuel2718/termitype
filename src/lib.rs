use anyhow::Result;
use clap::Parser;
use config::Config;

pub mod config;
pub mod termi;

pub fn run() -> Result<()> {
    println!("Termitype running");
    let config = Config::try_parse()?;

    let _ = termi::run(&config);

    Ok(())
}
