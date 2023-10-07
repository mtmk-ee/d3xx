use std::{ffi::c_ulong, future::Future, mem::MaybeUninit};

use crate::{ffi, try_d3xx, D3xxError, Device, Result};

/// Wrapper around the `FT_OVERLAPPED` structure.
///
/// This structure is used to perform asynchronous I/O operations.
/// Async I/O is done in a few steps:
///
/// 1. Create an `Overlapped` instance with [`Overlapped::new`].
/// 2. Perform the read/write operation in overlapped mode.
/// 3. Poll the `Overlapped` instance until the transfer is complete.
///
/// This struct also implements the [`Future`] trait, so it can be used with
/// the `async`/`await` syntax.
pub(crate) struct Overlapped<'a> {
    /// Reference to the device that this overlapped operation is associated with.
    device: &'a Device,
    /// Underlying `FT_OVERLAPPED` structure.
    overlapped: ffi::_OVERLAPPED,
}

impl<'a> Overlapped<'a> {
    /// Create a new `Overlapped` instance using the given device.
    pub fn new(device: &'a Device) -> Result<Self> {
        let mut overlapped: MaybeUninit<ffi::_OVERLAPPED> = MaybeUninit::uninit();
        try_d3xx!(unsafe {
            ffi::FT_InitializeOverlapped(
                device.handle(),
                &mut overlapped as *mut MaybeUninit<ffi::_OVERLAPPED> as *mut ffi::_OVERLAPPED,
            )
        })?;
        // SAFETY: `overlapped` is initialized since the initialization must have
        // succeeded if we're here.
        let overlapped = unsafe { overlapped.assume_init() };
        Ok(Self {
            device,
            overlapped,
        })
    }

    /// Get a reference to the underlying `FT_OVERLAPPED` structure.
    #[inline]
    #[allow(unused)]
    pub fn inner(&self) -> &ffi::_OVERLAPPED {
        &self.overlapped
    }

    /// Get a mutable reference to the underlying `FT_OVERLAPPED` structure.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut ffi::_OVERLAPPED {
        &mut self.overlapped
    }

    /// Poll the overlapped operation once.
    ///
    /// If `wait` is `true` then the operation will block until the transfer is complete.
    /// Otherwise, the operation will return immediately with `D3xxError::IoPending` if the
    /// transfer is not yet complete.
    ///
    /// If the operation is complete then the number of bytes transferred is returned.
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
