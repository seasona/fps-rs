//! Raw, unchecked bindings to the PresentMon API.
//!
//! The declarations come from `vendor/PresentMonAPI.h`, run through bindgen into
//! `$OUT_DIR/bindings.rs` by `build.rs` on every build, so compiling this crate
//! needs libclang and the MSVC headers.
//!
//! The `pm*` entry points are not link-time symbols: they live in
//! `PresentMonAPI2.dll`, which ships with PresentMon and may be absent or older than
//! the vendored header. [`FnTable`] mirrors the DLL's exports as function pointers,
//! to be filled by resolving each [`FnTable`] field name against the loaded module.

use core::ffi::c_char;

mod ffi {
    #![allow(
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        dead_code,
        clippy::all
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use ffi::*;

pub const PM_API_VERSION_MAJOR: u16 = ffi::PM_API_VERSION_MAJOR as u16;
pub const PM_API_VERSION_MINOR: u16 = ffi::PM_API_VERSION_MINOR as u16;

pub type PmOpenSession = unsafe extern "C" fn(*mut PM_SESSION_HANDLE) -> PM_STATUS;
pub type PmOpenSessionWithPipe =
    unsafe extern "C" fn(*mut PM_SESSION_HANDLE, *const c_char) -> PM_STATUS;
pub type PmCloseSession = unsafe extern "C" fn(PM_SESSION_HANDLE) -> PM_STATUS;
pub type PmStartTrackingProcess = unsafe extern "C" fn(PM_SESSION_HANDLE, u32) -> PM_STATUS;
pub type PmStartPlaybackTracking = unsafe extern "C" fn(PM_SESSION_HANDLE, u32, u32) -> PM_STATUS;
pub type PmStopTrackingProcess = unsafe extern "C" fn(PM_SESSION_HANDLE, u32) -> PM_STATUS;
pub type PmGetIntrospectionRoot =
    unsafe extern "C" fn(PM_SESSION_HANDLE, *mut *const PM_INTROSPECTION_ROOT) -> PM_STATUS;
pub type PmFreeIntrospectionRoot = unsafe extern "C" fn(*const PM_INTROSPECTION_ROOT) -> PM_STATUS;
pub type PmSetTelemetryPollingPeriod =
    unsafe extern "C" fn(PM_SESSION_HANDLE, u32, u32) -> PM_STATUS;
pub type PmSetEtwFlushPeriod = unsafe extern "C" fn(PM_SESSION_HANDLE, u32) -> PM_STATUS;
pub type PmFlushFrames = unsafe extern "C" fn(PM_SESSION_HANDLE, u32) -> PM_STATUS;
pub type PmRegisterDynamicQuery = unsafe extern "C" fn(
    PM_SESSION_HANDLE,
    *mut PM_DYNAMIC_QUERY_HANDLE,
    *mut PM_QUERY_ELEMENT,
    u64,
    f64,
    f64,
) -> PM_STATUS;
pub type PmFreeDynamicQuery = unsafe extern "C" fn(PM_DYNAMIC_QUERY_HANDLE) -> PM_STATUS;
pub type PmPollDynamicQuery =
    unsafe extern "C" fn(PM_DYNAMIC_QUERY_HANDLE, u32, *mut u8, *mut u32) -> PM_STATUS;
pub type PmPollDynamicQueryWithTimestamp =
    unsafe extern "C" fn(PM_DYNAMIC_QUERY_HANDLE, u32, *mut u8, *mut u32, u64) -> PM_STATUS;
pub type PmPollStaticQuery =
    unsafe extern "C" fn(PM_SESSION_HANDLE, *const PM_QUERY_ELEMENT, u32, *mut u8) -> PM_STATUS;
pub type PmRegisterFrameQuery = unsafe extern "C" fn(
    PM_SESSION_HANDLE,
    *mut PM_FRAME_QUERY_HANDLE,
    *mut PM_QUERY_ELEMENT,
    u64,
    *mut u32,
) -> PM_STATUS;
pub type PmConsumeFrames =
    unsafe extern "C" fn(PM_FRAME_QUERY_HANDLE, u32, *mut u8, *mut u32) -> PM_STATUS;
pub type PmFreeFrameQuery = unsafe extern "C" fn(PM_FRAME_QUERY_HANDLE) -> PM_STATUS;
pub type PmGetApiVersion = unsafe extern "C" fn(*mut PM_VERSION) -> PM_STATUS;
pub type PmStartEtlLogging =
    unsafe extern "C" fn(PM_SESSION_HANDLE, *mut PM_ETL_HANDLE, u64, u64) -> PM_STATUS;
pub type PmFinishEtlLogging =
    unsafe extern "C" fn(PM_SESSION_HANDLE, PM_ETL_HANDLE, *mut c_char, u32) -> PM_STATUS;

// The aliases are hand-written against bindgen's output. Coercing each entry point
// to its alias keeps a signature mismatch after a header bump a compile error
// instead of undefined behaviour at the call site.
const _: PmOpenSession = ffi::pmOpenSession;
const _: PmOpenSessionWithPipe = ffi::pmOpenSessionWithPipe;
const _: PmCloseSession = ffi::pmCloseSession;
const _: PmStartTrackingProcess = ffi::pmStartTrackingProcess;
const _: PmStartPlaybackTracking = ffi::pmStartPlaybackTracking;
const _: PmStopTrackingProcess = ffi::pmStopTrackingProcess;
const _: PmGetIntrospectionRoot = ffi::pmGetIntrospectionRoot;
const _: PmFreeIntrospectionRoot = ffi::pmFreeIntrospectionRoot;
const _: PmSetTelemetryPollingPeriod = ffi::pmSetTelemetryPollingPeriod;
const _: PmSetEtwFlushPeriod = ffi::pmSetEtwFlushPeriod;
const _: PmFlushFrames = ffi::pmFlushFrames;
const _: PmRegisterDynamicQuery = ffi::pmRegisterDynamicQuery;
const _: PmFreeDynamicQuery = ffi::pmFreeDynamicQuery;
const _: PmPollDynamicQuery = ffi::pmPollDynamicQuery;
const _: PmPollDynamicQueryWithTimestamp = ffi::pmPollDynamicQueryWithTimestamp;
const _: PmPollStaticQuery = ffi::pmPollStaticQuery;
const _: PmRegisterFrameQuery = ffi::pmRegisterFrameQuery;
const _: PmConsumeFrames = ffi::pmConsumeFrames;
const _: PmFreeFrameQuery = ffi::pmFreeFrameQuery;
const _: PmGetApiVersion = ffi::pmGetApiVersion;
const _: PmStartEtlLogging = ffi::pmStartEtlLogging;
const _: PmFinishEtlLogging = ffi::pmFinishEtlLogging;

/// Every export of `PresentMonAPI2.dll`, keyed by the symbol name in snake case.
///
/// Fields are `Option<_>` because the DLL may be older than this header and
/// legitimately miss some entry points: the loader fills what it needs and
/// leaves the rest `None`; call sites unwrap defensively.
#[derive(Clone, Copy, Debug, Default)]
pub struct FnTable {
    pub pm_open_session: Option<PmOpenSession>,
    pub pm_open_session_with_pipe: Option<PmOpenSessionWithPipe>,
    pub pm_close_session: Option<PmCloseSession>,
    pub pm_start_tracking_process: Option<PmStartTrackingProcess>,
    pub pm_start_playback_tracking: Option<PmStartPlaybackTracking>,
    pub pm_stop_tracking_process: Option<PmStopTrackingProcess>,
    pub pm_get_introspection_root: Option<PmGetIntrospectionRoot>,
    pub pm_free_introspection_root: Option<PmFreeIntrospectionRoot>,
    pub pm_set_telemetry_polling_period: Option<PmSetTelemetryPollingPeriod>,
    pub pm_set_etw_flush_period: Option<PmSetEtwFlushPeriod>,
    pub pm_flush_frames: Option<PmFlushFrames>,
    pub pm_register_dynamic_query: Option<PmRegisterDynamicQuery>,
    pub pm_free_dynamic_query: Option<PmFreeDynamicQuery>,
    pub pm_poll_dynamic_query: Option<PmPollDynamicQuery>,
    pub pm_poll_dynamic_query_with_timestamp: Option<PmPollDynamicQueryWithTimestamp>,
    pub pm_poll_static_query: Option<PmPollStaticQuery>,
    pub pm_register_frame_query: Option<PmRegisterFrameQuery>,
    pub pm_consume_frames: Option<PmConsumeFrames>,
    pub pm_free_frame_query: Option<PmFreeFrameQuery>,
    pub pm_get_api_version: Option<PmGetApiVersion>,
    pub pm_start_etl_logging: Option<PmStartEtlLogging>,
    pub pm_finish_etl_logging: Option<PmFinishEtlLogging>,
}
