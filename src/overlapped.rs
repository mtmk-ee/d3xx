//! Wrapper around the `FT_OVERLAPPED` structure.
//!
//! The [`Overlapped`] structure is used to perform asynchronous I/O operations, also
//! referred to as "overlapped" operations in the D3XX and Windows APIs. Overlapped
//! I/O operations allow the caller to perform other tasks while waiting for the
//! operation to complete in the background.
//!
//! Internally this boils down to a few steps:
/// 1. Create an `Overlapped` instance with [`Overlapped::new`].
/// 2. Perform the read/write operation in overlapped mode.
/// 3. Poll the `Overlapped` instance until the transfer is complete.
use std::{ffi::c_ulong, future::Future, marker::PhantomData, mem::MaybeUninit};

use crate::{ffi, try_d3xx, util::PhantomLifetime, D3xxError, Device, Result};

/// Wrapper around the `FT_OVERLAPPED` structure.
///
/// This struct is used to perform asynchronous (overlapped) I/O operations.
/// This struct also implements the [`Future`] trait, so it can be used with
/// the `async`/`await` Rust syntax.
pub struct Overlapped<'a> {
    handle: ffi::HANDLE,
    overlapped: ffi::_OVERLAPPED,
    /// Ties the lifetime of this struct to the lifetime of the source [`Device`](crate::Device) instance.
    _lifetime_constraint: PhantomLifetime<'a>,
}

impl<'a> Overlapped<'a> {
    /// Create a new `Overlapped` instance using the given device.
    ///
    /// The lifetime of the `Overlapped` instance is tied to the lifetime of the `Device` instance
    /// to avoid use-after-free errors.
    #[allow(unused)]
    pub(crate) fn new(device: &'a Device) -> Result<Self> {
        Self::with_handle(device.handle())
    }

    /// Create a new `Overlapped` instance using the given handle.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the handle is valid for the lifetime of the `Overlapped`
    /// instance.
    pub(crate) fn with_handle(handle: ffi::FT_HANDLE) -> Result<Self> {
        let mut overlapped: MaybeUninit<ffi::_OVERLAPPED> = MaybeUninit::uninit();
        try_d3xx!(unsafe {
            ffi::FT_InitializeOverlapped(handle, std::ptr::addr_of_mut!(overlapped).cast())
        })?;
        // SAFETY: `overlapped` is properly initialized by `FT_InitializeOverlapped`.
        let overlapped = unsafe { overlapped.assume_init() };
        Ok(Self {
            handle,
            overlapped,
            _lifetime_constraint: PhantomData,
        })
    }

    /// Get a reference to the underlying `FT_OVERLAPPED` structure.
    ///
    /// This can be used to pass the overlapped structure to FFI functions such as
    /// [`FT_WritePipe`](crate::ffi::FT_WritePipe).
    #[inline]
    #[must_use]
    #[allow(unused)]
    pub fn inner(&self) -> &ffi::_OVERLAPPED {
        &self.overlapped
    }

    /// Get a mutable reference to the underlying `FT_OVERLAPPED` structure.
    ///
    /// This can be used to pass the overlapped structure to FFI functions.
    #[inline]
    #[must_use]
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
                self.handle,
                self.inner_mut(),
                std::ptr::addr_of_mut!(transferred),
                i32::from(wait),
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
            ffi::FT_ReleaseOverlapped(self.handle, self.inner_mut() as *mut ffi::_OVERLAPPED);
        }
    }
}
