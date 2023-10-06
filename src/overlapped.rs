use std::{ffi::c_ulong, future::Future, mem::MaybeUninit};

use crate::{ffi, try_d3xx, D3xxError, Device, Result};

/// A handle to an overlapped (asynchronous) I/O operation.
///
/// Overlapped IO
pub(crate) struct Overlapped<'a> {
    device: &'a Device,
    overlapped: ffi::_OVERLAPPED,
}

impl<'a> Overlapped<'a> {
    pub fn new(device: &'a Device) -> Result<Self> {
        let mut overlapped: MaybeUninit<ffi::_OVERLAPPED> = MaybeUninit::uninit();
        try_d3xx!(unsafe {
            ffi::FT_InitializeOverlapped(
                device.handle,
                &mut overlapped as *mut MaybeUninit<ffi::_OVERLAPPED> as *mut ffi::_OVERLAPPED,
            )
        })?;
        // SAFETY: `overlapped` is initialized since the initialization must have
        // succeeded if we're here.
        let overlapped = unsafe { overlapped.assume_init() };
        Ok(Self {
            device: &device,
            overlapped,
        })
    }

    #[inline]
    #[allow(unused)]
    pub fn inner(&self) -> &ffi::_OVERLAPPED {
        &self.overlapped
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut ffi::_OVERLAPPED {
        &mut self.overlapped
    }

    fn poll_once(&mut self, wait: bool) -> Result<usize> {
        let mut transferred: c_ulong = 0;
        try_d3xx!(unsafe {
            ffi::FT_GetOverlappedResult(
                self.device.handle(),
                self.inner_mut() as *mut ffi::_OVERLAPPED,
                &mut transferred as *mut c_ulong,
                wait as i32,
            )
        })?;
        Ok(transferred as usize)
    }
}

impl Future for Overlapped<'_> {
    type Output = Result<usize>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.poll_once(false) {
            Ok(transferred) => std::task::Poll::Ready(Ok(transferred)),
            Err(D3xxError::IoPending | D3xxError::IoIncomplete) => {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
            Err(e) => std::task::Poll::Ready(Err(e)),
        }
    }
}

impl Drop for Overlapped<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::FT_ReleaseOverlapped(
                self.device.handle(),
                self.inner_mut() as *mut ffi::_OVERLAPPED,
            );
        }
    }
}
