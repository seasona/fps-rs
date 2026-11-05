use std::rc::Rc;

use crate::error::PmError;
use crate::error::check_Pmstatus;
use crate::loader::Api;
use presentmon_sys as sys;

pub struct Session {
    inner: Rc<SessionInner>,
}

pub(crate) struct SessionInner {
    pub(crate) api: Rc<Api>,
    pub(crate) handle: sys::PM_SESSION_HANDLE,
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

        if version.major != sys::PM_API_VERSION_MAJOR || version.minor != sys::PM_API_VERSION_MINOR {
            return Err(PmError::VersionMismatch {
                dll_major: version.major,
                dll_minor: version.minor,
                expected_major: sys::PM_API_VERSION_MAJOR,
                expected_minor: sys::PM_API_VERSION_MINOR,
            });
        }

        Ok(())
    }
}

impl Drop for SessionInner {
    fn drop(&mut self) {
        log::info!("Session dropped");
        unsafe { self.api.fns.pm_close_session.unwrap()(self.handle) };
    }
}
