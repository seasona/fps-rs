use presentmon_sys as sys;
use thiserror::Error;

/// Error wrapper
#[derive(Error, Debug)]
pub enum PmError {
    #[error("PresentMonAPI2 DLL not found (searched: {0})")]
    DllNotFound(String),
    #[error("PresentMonAPI2 DLL load failed (path: {0})")]
    DllLoadFailed(String),
    #[error("Symbol missing from PresentMonAPI2 DLL (symbol: {0})")]
    SymbolMissing(String),
    #[error("pmOpenSession failed (status: {0})")]
    OpenSession(sys::PM_STATUS),
    #[error(
        "PresentMonAPI2 version {dll_major} {dll_minor} mismatch with {expected_major} {expected_minor}"
    )]
    VersionMismatch {
        dll_major: u16,
        dll_minor: u16,
        expected_major: u16,
        expected_minor: u16,
    },
    #[error("PresentMonAPI2 error (status: {0})")]
    Other(i32),
}

pub(crate) fn check_Pmstatus(status: sys::PM_STATUS, pid: u32) -> Result<(), PmError> {
    match status {
        sys::PM_STATUS_SUCCESS => Ok(()),
        _ => Err(PmError::Other(status)),
    }
}
