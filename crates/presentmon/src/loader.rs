use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use presentmon_sys as sys;
use windows::Win32::Foundation::{FreeLibrary, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::core::{PCWSTR, s};

use crate::error::PmError;

/// Load DLL file and get symbols addresses
pub(crate) struct Api {
    module: HMODULE,
    pub(crate) fns: sys::FnTable,
}

impl Api {
    pub(crate) fn load() -> Result<Self, PmError> {
        let dll_paths = get_dll_paths();
        let mut tried = Vec::with_capacity(dll_paths.len());
        let mut load_err = None;

        for path in dll_paths {
            tried.push(path.display().to_string());
            if !path.exists() || !path.is_file() {
                continue;
            }
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
            match unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) } {
                Ok(module) => {
                    return Ok(Api {
                        module,
                        fns: unsafe { Self::get_fns(module)? },
                    });
                }
                Err(err) => load_err = Some(err.to_string()),
            }
        }

        match load_err {
            Some(err) => Err(PmError::DllLoadFailed(format!(
                "{err} (searched: {})",
                tried.join("; ")
            ))),
            None => Err(PmError::DllNotFound(tried.join("; "))),
        }
    }

    unsafe fn get_fns(module: HMODULE) -> Result<sys::FnTable, PmError> {
        // Required symbols: missing => release the DLL and bail.
        macro_rules! required {
            ($name:literal, $ty:ty) => {
                match unsafe { GetProcAddress(module, s!($name)) } {
                    Some(proc) => Some(unsafe {
                        std::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(proc)
                    }),
                    None => {
                        unsafe {
                            let _ = FreeLibrary(module);
                        }
                        return Err(PmError::SymbolMissing($name.to_string()));
                    }
                }
            };
        }
        // Optional symbols: missing => `None`, caller decides what to do.
        macro_rules! optional {
            ($name:literal, $ty:ty) => {
                unsafe { GetProcAddress(module, s!($name)) }.map(|proc| unsafe {
                    std::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(proc)
                })
            };
        }

        // Any field left out of this literal defaults to `None`, so we only
        // resolve the entry points we actually plan to use.
        Ok(sys::FnTable {
            pm_open_session: required!("pmOpenSession", sys::PmOpenSession),
            pm_close_session: required!("pmCloseSession", sys::PmCloseSession),
            pm_open_session_with_pipe: optional!(
                "pmOpenSessionWithPipe",
                sys::PmOpenSessionWithPipe
            ),
            pm_get_api_version: required!("pmGetApiVersion", sys::PmGetApiVersion),
            ..Default::default()
        })
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.module);
        };
    }
}

fn get_dll_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // first try the user provided path
    if let Some(path) = std::env::var_os("PRESENTMON_DLL_PATH") {
        if !path.is_empty() {
            paths.push(path.into());
        }
    }

    // try executable path
    if let Ok(exepath) = std::env::current_exe() {
        if let Some(dir) = exepath.parent() {
            paths.push(dir.join("PresentMonAPI2.dll"));
            paths.push(dir.join("vendor").join("PresentMonAPI2.dll"));
        }
    }

    // try program files path
    let program_files =
        std::env::var_os("ProgramFiles").unwrap_or_else(|| r"C:\Program Files".into());
    let program_files = PathBuf::from(program_files);
    let intel = program_files.join("Intel");
    paths.push(
        intel
            .join("PresentMonSharedService")
            .join("PresentMonAPI2.dll"),
    );

    paths
}
