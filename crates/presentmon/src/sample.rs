use std::rc::Rc;

use crate::error::{PmError, check_Pmstatus};
use crate::session::TrackedProcessInner;
use presentmon_sys as sys;

#[derive(Debug, Clone, Copy)]
pub struct FpsSample {
    pub displayed_fps: f64,
    pub ms_between_presents: f64,
}

/// Registered dynamic query.
pub struct FpsQuery {
    handle: sys::PM_DYNAMIC_QUERY_HANDLE,
    tracker: Rc<TrackedProcessInner>,
    fps_offset: usize,
    between_offset: usize,
    blob: Vec<u8>,
}

impl FpsQuery {
    pub(crate) fn register(
        tracker: Rc<TrackedProcessInner>,
        window_ms: f64,
    ) -> Result<Self, PmError> {
        let session = &tracker.session;
        let api = &session.api;

        log::debug!("start register fps query");

        // PM_METRIC_BETWEEN_PRESENTS is a frame-event-only metric and is rejected
        // from a dynamic query (PM_STATUS_QUERY_MALFORMED); FrameTime-Presents is
        // its dynamic equivalent: ms between this Present call and the previous one.
        // deviceId stays 0 (first adapter): the installed service rejects every
        // element that asks for the universal device id.
        let mut elements = [
            sys::PM_QUERY_ELEMENT {
                metric: sys::PM_METRIC_DISPLAYED_FPS,
                stat: sys::PM_STAT_AVG,
                ..Default::default()
            },
            sys::PM_QUERY_ELEMENT {
                metric: sys::PM_METRIC_PRESENTED_FRAME_TIME,
                stat: sys::PM_STAT_AVG,
                ..Default::default()
            },
        ];

        let register_dynamic_query_fn = api
            .fns
            .pm_register_dynamic_query
            .ok_or(PmError::SymbolMissing("pmRegisterDynamicQuery".to_string()))?;

        let mut handle: sys::PM_DYNAMIC_QUERY_HANDLE = std::ptr::null_mut();
        let status = unsafe {
            register_dynamic_query_fn(
                session.handle,
                &mut handle,
                elements.as_mut_ptr(),
                elements.len() as u64,
                window_ms,
                0.0,
            )
        };
        check_Pmstatus(status, tracker.pid)?;

        log::debug!("register fps query successfully");

        // The API backfills offsets/sizes during registration;
        // the blob is the concatenation of elements, so its size is the last end offset.
        let (fps_offset, fps_size) = (elements[0].dataOffset, elements[0].dataSize);
        let (between_offset, between_size) = (elements[1].dataOffset, elements[1].dataSize);

        let free_fn = |handle| unsafe {
            let _ = (api.fns.pm_free_dynamic_query.unwrap()(handle));
        };

        if fps_size != 8 {
            free_fn(handle);
            return Err(PmError::UnexpectedLayout {
                metric: sys::PM_METRIC_DISPLAYED_FPS as u32,
                size: fps_size,
            });
        }

        if between_size != 8 {
            free_fn(handle);
            return Err(PmError::UnexpectedLayout {
                metric: sys::PM_METRIC_PRESENTED_FRAME_TIME as u32,
                size: between_size,
            });
        }

        log::debug!(
            "fps_offset: {}, fps_size: {}, between_offset: {}, between_size: {}",
            fps_offset,
            fps_size,
            between_offset,
            between_size
        );

        let blob_size_u64 = (fps_offset + fps_size).max(between_offset + between_size);
        let blob_size = usize::try_from(blob_size_u64).map_err(|_| {
            free_fn(handle);
            PmError::Other(sys::PM_STATUS_QUERY_MALFORMED)
        })?;

        Ok(Self {
            handle,
            tracker,
            fps_offset: fps_offset as usize,
            between_offset: between_offset as usize,
            blob: vec![0u8; blob_size],
        })
    }

    /// Poll the query for `pid`, `Ok(None)` means no frame data yet
    pub fn poll(&mut self, pid: u32) -> Result<Option<FpsSample>, PmError> {
        let mut num_swap_chains: u32 = 1;
        let poll_query_fn = self
            .tracker
            .session
            .api
            .fns
            .pm_poll_dynamic_query
            .ok_or(PmError::SymbolMissing("pmPollDynamicQuery".to_string()))?;

        let status = unsafe {
            poll_query_fn(
                self.handle,
                pid,
                self.blob.as_mut_ptr(),
                &mut num_swap_chains,
            )
        };
        check_Pmstatus(status, pid)?;

        if num_swap_chains == 0 {
            return Ok(None);
        }

        let displayed_fps = self.read_f64(self.fps_offset);
        let ms_between_presents = self.read_f64(self.between_offset);

        Ok(Some(FpsSample {
            displayed_fps,
            ms_between_presents,
        }))
    }

    fn read_f64(&self, offset: usize) -> f64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.blob[offset..offset + 8]);
        f64::from_le_bytes(bytes)
    }
}

impl Drop for FpsQuery {
    fn drop(&mut self) {
        unsafe {
            let _ = self.tracker.session.api.fns.pm_free_dynamic_query.unwrap()(self.handle);
        };
    }
}
