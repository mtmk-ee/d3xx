use std::{
    ffi::{c_uchar, c_ulong, c_ushort, c_void, CString},
    time::Duration,
};

use crate::{ffi, overlapped::Overlapped, try_d3xx, D3xxError, Pipe, Result, StreamPipes};

type PhantomUnsync = std::marker::PhantomData<std::cell::Cell<()>>;

/// Handle to a D3XX device.
///
/// The handle is the primary interface for interacting with a FT60x device.
/// It provides methods for reading, writing, configuration, and more.
///
/// # Example
///
/// ```no_run
/// use d3xx::{Device, Pipe};
///
/// let device = Device::open("ABC123").unwrap();
///
/// // Read 1024 bytes from input pipe 1
/// let mut buf = vec![0u8; 1024];
/// device.read(Pipe::In1, &mut buf).unwrap();
///
/// // Write 1024 bytes to output pipe 1
/// device.write(Pipe::Out1, &buf).unwrap();
/// ```
#[derive(Debug)]
pub struct Device {
    /// Handle returned by the D3XX driver when the device is opened.
    handle: ffi::FT_HANDLE,
    /// Serial number of the device.
    serial_number: String,
    // Cannot share handle across threads since the driver is not thread-safe,
    // and so we need to prevent race conditions on device operations.
    _unsync: PhantomUnsync,
}

impl Device {
    /// Open a device by serial number.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use d3xx::Device;
    ///
    /// let device = Device::open("ABC123").unwrap();
    /// ```
    pub fn open(serial_number: &str) -> Result<Self> {
        let serial_cstr = CString::new(serial_number).expect("failed to create CString");
        let mut handle: ffi::FT_HANDLE = std::ptr::null_mut();
        try_d3xx!(unsafe {
            ffi::FT_Create(
                serial_cstr.as_ptr() as *mut c_void,
                ffi::FT_OPEN_BY_SERIAL_NUMBER,
                &mut handle,
            )
        })?;
        if handle.is_null() {
            Err(crate::D3xxError::DeviceNotFound)
        } else {
            Ok(Self {
                handle,
                serial_number: serial_number.to_owned(),
                _unsync: Default::default(),
            })
        }
    }

    /// Get the device's handle.
    ///
    /// This handle is fairly useless on its own. Although not recommended for typical
    /// users, it may be used with the raw D3XX bindings in the [ffi] module.
    pub fn handle(&self) -> ffi::FT_HANDLE {
        self.handle
    }

    /// Get the device's serial number.
    ///
    /// This is the serial number that was passed to `Device::open`.
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Write to the specified pipe.
    ///
    /// This method will block until the transfer is complete.
    ///
    /// On success the number of bytes written is returned.
    pub fn write(&self, pipe: Pipe, buf: &[u8]) -> Result<usize> {
        let res = ffi::util::write_pipe(self.handle, pipe as u8, buf);
        self.wrap_pipe_io_abort(pipe, res)
    }

    /// Asynchronous write to the specified pipe.
    ///
    /// On success the number of bytes written is returned.
    pub async fn write_async<'a>(&'a self, pipe: Pipe, buf: &[u8]) -> Result<usize> {
        let mut overlapped = Overlapped::new(self)?;
        let res = ffi::util::write_pipe_async(self.handle, pipe as u8, buf, overlapped.inner_mut());
        self.wrap_pipe_io_abort(pipe, res)?;
        overlapped.await
    }

    /// Read from the specified pipe into the given buffer.
    ///
    /// This method will block until the transfer is complete.
    ///
    /// On success the number of bytes read is returned.
    pub fn read(&self, pipe: Pipe, buf: &mut [u8]) -> Result<usize> {
        let res = ffi::util::read_pipe(self.handle, pipe as u8, buf);
        self.wrap_pipe_io_abort(pipe, res)
    }

    /// Asynchronous read from the specified pipe into the given buffer.
    ///
    /// On success the number of bytes read is returned.
    pub async fn read_async(&self, pipe: Pipe, buf: &mut [u8]) -> Result<usize> {
        let mut overlapped = Overlapped::new(self)?;
        let res = ffi::util::read_pipe_async(self.handle, pipe as u8, buf, overlapped.inner_mut());
        self.wrap_pipe_io_abort(pipe, res)?;
        overlapped.await
    }

    /// Enable streaming protocol transfer for the specified pipes.
    ///
    /// See D3XX Programmer's Guide, Section 2.14 for more information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use d3xx::{Device, Pipe, StreamPipes};
    ///
    /// let device = Device::open("ABC123").unwrap();
    ///
    /// // Disable streaming on all pipes
    /// device.set_stream_pipes(StreamPipes::default()).unwrap();
    ///
    /// // Enable streaming on input pipe 1 with a stream size of 1024 bytes
    /// device.set_stream_pipes(
    ///     StreamPipes::default().with_pipe(Pipe::In1, 1024)
    /// ).unwrap();
    ///
    /// // Enable streaming on several pipes
    /// device.set_stream_pipes(
    ///    StreamPipes::default()
    ///       .with_pipe(Pipe::In1, 1024)
    ///       .with_pipe(Pipe::In2, 1024)
    ///       .with_pipe(Pipe::Out1, 1024)
    /// ).unwrap();
    /// ```
    pub fn set_stream_pipes(&self, pipes: StreamPipes) -> Result<()> {
        try_d3xx!(unsafe {
            ffi::FT_ClearStreamPipe(self.handle, true as c_uchar, true as c_uchar, 0)
        })?;
        for (pipe, stream_size) in pipes {
            try_d3xx!(unsafe {
                ffi::FT_SetStreamPipe(
                    self.handle,
                    false as c_uchar,
                    false as c_uchar,
                    pipe as c_uchar,
                    stream_size.try_into().or(Err(D3xxError::InvalidArgs))?,
                )
            })?;
        }
        Ok(())
    }

    /// Aborts all pending transfers on the specified pipe.
    pub fn abort_pipe(&self, pipe: Pipe) -> Result<()> {
        try_d3xx!(unsafe { ffi::FT_AbortPipe(self.handle, pipe as c_uchar) })
    }

    /// Aborts all pending transfers on the specified pipe if the given result is an error.
    ///
    /// This is a convenience method for aborting a pipe on read/write failure, as required
    /// by the driver. See D3XX Programmer's Guide, pg. 15 for more information.
    ///
    /// Returns the given result for convenience.
    fn wrap_pipe_io_abort<T>(&self, pipe: Pipe, res: Result<T>) -> Result<T> {
        res.map_err(|e| {
            let _ = self.abort_pipe(pipe);
            e
        })
    }

    /// Get the timeout for the specified pipe.
    ///
    /// The default is 5 seconds for all pipes, and is reset every time the
    /// device is opened.
    pub fn pipe_timeout(&self, pipe: Pipe) -> Result<Duration> {
        let mut timeout_ms: c_ulong = 0;
        try_d3xx!(unsafe {
            ffi::FT_GetPipeTimeout(self.handle, pipe as c_uchar, &mut timeout_ms)
        })?;
        Ok(Duration::from_millis(timeout_ms as u64))
    }

    /// Set the timeout for the specified pipe.
    ///
    /// The maximum timeout is `u32::MAX` milliseconds.
    ///
    /// The default is 5 seconds for all pipes, and is reset every time the
    /// device is opened.
    pub fn set_pipe_timeout(&self, pipe: Pipe, timeout: Duration) -> Result<()> {
        let timeout_ms = timeout
            .as_millis()
            .try_into()
            .unwrap_or_else(|_| panic!("timeout too large"));
        try_d3xx!(unsafe { ffi::FT_SetPipeTimeout(self.handle, pipe as c_uchar, timeout_ms) })
    }

    /// Get the device's vendor ID and product ID.
    pub fn vid_pid(&self) -> Result<(usize, usize)> {
        let mut vid: c_ushort = 0;
        let mut pid: c_ushort = 0;
        try_d3xx!(unsafe {
            ffi::FT_GetVIDPID(
                self.handle,
                &mut vid as *mut c_ushort,
                &mut pid as *mut c_ushort,
            )
        })?;
        Ok((vid as usize, pid as usize))
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            ffi::FT_Close(self.handle);
        }
    }
}
