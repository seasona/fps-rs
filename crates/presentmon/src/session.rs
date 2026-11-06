use std::rc::Rc;

use crate::error::PmError;
use crate::error::check_Pmstatus;
use crate::loader::Api;
use crate::sample::FpsQuery;
use presentmon_sys as sys;

pub struct Session {
    inner: Rc<SessionInner>,
}

pub(crate) struct SessionInner {
    pub(crate) api: Rc<Api>,
    pub(crate) handle: sys::PM_SESSION_HANDLE,
}

pub struct TrackedProcess {
    pub(crate) inner: Rc<TrackedProcessInner>,
}

pub struct TrackedProcessInner {
    pub(crate) session: Rc<SessionInner>,
    pub(crate) pid: u32,
}

impl Session {
    /// Open a new Session.
    pub fn open() -> Result<Self, PmError> {
        let api = Rc::new(Api::load()?);
        Self::check_api_version(&api)?;
        log::info!("API loaded successfully");

        let open_session_fn = api
            .fns
            .pm_open_session
            .ok_or(PmError::SymbolMissing("pmOpenSession".to_string()))?;

        let mut handle: sys::PM_SESSION_HANDLE = std::ptr::null_mut();
        let status = unsafe { open_session_fn(&mut handle) };
        check_Pmstatus(status, 0)?;
        if handle.is_null() {
            return Err(PmError::Other((sys::PM_STATUS_FAILURE)));
        }

        log::info!("Session opened successfully");
        Ok(Self {
            inner: Rc::new(SessionInner { api, handle }),
        })
    }

    /// Check if DLL API version matches with bindings.
    fn check_api_version(api: &Api) -> Result<(), PmError> {
        let get_api_version_fn = api
            .fns
            .pm_get_api_version
            .ok_or(PmError::SymbolMissing("pmCheckApiVersion".to_string()))?;

        let mut version = sys::PM_VERSION::default();

        let status = unsafe { get_api_version_fn(&mut version) };
        check_Pmstatus(status, 0)?;

        log::info!("API version: {}.{}", version.major, version.minor);

        if version.major != sys::PM_API_VERSION_MAJOR || version.minor != sys::PM_API_VERSION_MINOR
        {
            return Err(PmError::VersionMismatch {
                dll_major: version.major,
                dll_minor: version.minor,
                expected_major: sys::PM_API_VERSION_MAJOR,
                expected_minor: sys::PM_API_VERSION_MINOR,
            });
        }

        Ok(())
    }

    /// Set the ETW flush period, lower value means more frequent flushes.
    pub fn pm_set_etw_flush_period(&self, ms: u32) -> Result<(), PmError> {
        let set_etw_flush_period_fn = self
            .inner
            .api
            .fns
            .pm_set_etw_flush_period
            .ok_or(PmError::SymbolMissing("pmSetEtwFlushPeriod".to_string()))?;
        let status = unsafe { set_etw_flush_period_fn(self.inner.handle, ms) };
        check_Pmstatus(status, 0)?;
        Ok(())
    }

    /// Start tracking a process with the given PID.
    pub fn track(&self, pid: u32) -> Result<TrackedProcess, PmError> {
        let track_fn = self
            .inner
            .api
            .fns
            .pm_start_tracking_process
            .ok_or(PmError::SymbolMissing("pmTrack".to_string()))?;
        let status = unsafe { track_fn(self.inner.handle, pid) };
        check_Pmstatus(status, 0)?;
        Ok(TrackedProcess {
            inner: Rc::new(TrackedProcessInner {
                session: self.inner.clone(),
                pid,
            }),
        })
    }
}

impl Drop for SessionInner {
    fn drop(&mut self) {
        log::info!("Session dropped");
        unsafe { self.api.fns.pm_close_session.unwrap()(self.handle) };
    }
}

impl TrackedProcess {
    /// Get the PID of the tracked process.
    pub fn pid(&self) -> u32 {
        self.inner.pid
    }

    pub fn register_fps_query(&self, window_ms: f64) -> Result<FpsQuery, PmError> {
        FpsQuery::register(Rc::clone(&self.inner), window_ms)
    }
}

impl Drop for TrackedProcessInner {
    fn drop(&mut self) {
        log::info!("Tracked process dropped");
        unsafe {
            self.session.api.fns.pm_stop_tracking_process.unwrap()(self.session.handle, self.pid)
        };
    }
}
