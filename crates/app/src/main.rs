use std::sync::Arc;
use std::thread;

use env_logger::Env;
use presentmon::{FpsQuery, FpsSample, PmError, Session, TrackedProcess};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}

struct ActiveTarget {
    query: FpsQuery,
    _tacked: TrackedProcess,
    pid: u32,
}

fn start_tracking(session: &Session, pid: u32) -> Result<ActiveTarget, PmError> {
    let tracked = session.track(pid)?;
    log::info!("start tracking with pid: {}", pid);
    let query = tracked.register_fps_query(presentmon::DEFAULT_WINDOW_MS)?;
    Ok(ActiveTarget {
        query,
        _tacked: tracked,
        pid,
    })
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    log::info!("Starting app");

    let session =Session::open()?;

    log::info!("Session opened successfully");

    let pid = 7548;

    let mut target = start_tracking(&session, pid)?;

    log::info!("start tracking...");

    if let Some(sample) = target.query.poll(pid)? {
        log::info!("fps: {}, delta: {}", sample.displayed_fps, sample.ms_between_presents);
    }

    Ok(())
}
