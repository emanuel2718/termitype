// use std::io;
//
// use crossterm::{event::EnableMouseCapture, execute, terminal::{self, EnterAlternateScreen}};
//
// pub fn run() -> Result<(), Box<dyn Error>> {
//     terminal::enable_raw_mode()?;
//     let mut stdout = io::stdout();
//     execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//
//
// }

use std::error::Error;


pub fn run() -> Result<(), Box<dyn Error>> {
    println!("Hello termitype!");
    Ok(())
}
