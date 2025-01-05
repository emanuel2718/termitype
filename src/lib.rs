use anyhow::Result;
use clap::Parser;
use config::Config;

pub mod config;

pub fn run() -> Result<()> {
    println!("Termitype running");
    let mut config = Config::try_parse()?;

    dbg!(&config);

    config.toggle_use_symbols();

    dbg!("After toggle_use_symbols: {:?}", &config);

    Ok(())
}
