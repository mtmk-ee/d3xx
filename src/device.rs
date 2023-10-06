use std::{
    ffi::{c_uchar, c_ulong, c_ushort, c_void, CString},
    time::Duration,
};

use crate::{ffi, overlapped::Overlapped, try_d3xx, D3xxError, Pipe, Result, StreamPipes};

type PhantomUnsync = std::marker::PhantomData<std::cell::Cell<()>>;

#[derive(Debug)]
pub struct Device {
    pub(crate) handle: ffi::FT_HANDLE,
    serial_number: String,
    // Cannot share handle across threads since the driver is not thread-safe,
    // and so we need to prevent race conditions on device operations.
    _unsync: PhantomUnsync,
}

impl Device {
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
            Err(crate::D3xxError::DeviceNotFound.into())
        } else {
            Ok(Self {
                handle,
                serial_number: serial_number.to_owned(),
                _unsync: Default::default(),
            })
        }
    }

    pub fn handle(&self) -> ffi::FT_HANDLE {
        self.handle
    }

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

    pub fn abort_pipe(&self, pipe: Pipe) -> Result<()> {
        try_d3xx!(unsafe { ffi::FT_AbortPipe(self.handle, pipe as c_uchar) })
    }

    fn wrap_pipe_io_abort<T>(&self, pipe: Pipe, res: Result<T>) -> Result<T> {
        res.map_err(|e| {
            // Pipe must be aborted on failure (D3XX Programmer's Guide pg. 15)
            let _ = self.abort_pipe(pipe);
            e
        })
    }

    pub fn pipe_timeout(&self, pipe: Pipe) -> Result<Duration> {
        let mut timeout_ms: c_ulong = 0;
        try_d3xx!(unsafe {
            ffi::FT_GetPipeTimeout(self.handle, pipe as c_uchar, &mut timeout_ms)
        })?;
        Ok(Duration::from_millis(timeout_ms as u64))
    }

    pub fn set_pipe_timeout(&self, pipe: Pipe, timeout: Duration) -> Result<()> {
        let timeout_ms = timeout
            .as_millis()
            .try_into()
            .unwrap_or_else(|_| panic!("timeout too large"));
        try_d3xx!(unsafe { ffi::FT_SetPipeTimeout(self.handle, pipe as c_uchar, timeout_ms) })
    }

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
