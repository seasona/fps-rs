use std::sync::Arc;
use std::thread;

use env_logger::Env;
use presentmon::session::Session;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    log::info!("Starting app");

    Session::open()?;

    log::info!("Session opened successfully");

    Ok(())
}
