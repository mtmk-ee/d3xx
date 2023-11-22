use std::{
    ffi::c_uchar,
    io::{Read, Write},
    marker::PhantomData,
};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    descriptor::PipeInfo, ffi, overlapped::Overlapped, try_d3xx, util::PhantomLifetime, D3xxError,
    Device, Result,
};

/// Provides read/write access to an endpoint on the device.
///
/// This struct implements [`Read`] and [`Write`], so it can be used with
/// the standard library's I/O functions.
///
/// # Examples
///
/// ```no_run
/// use std::io::Write;
/// use d3xx::{Device, Pipe};
///
/// let device = Device::open("ABC123").unwrap();
///
/// // Write to output pipe 1
/// let mut buf = vec![0u8; 1024];
/// device
///    .pipe(Pipe::Out1)
///    .write(&buf)
///    .unwrap();
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipeIo<'a> {
    /// Handle to the device.
    ///
    /// Rust's type system ensures through the lifetime parameter that this handle
    /// cannot outlive the `Device` instance it belongs to.
    handle: ffi::FT_HANDLE,
    /// The pipe ID this instance is associated with.
    id: Pipe,
    /// Lifetime marker, required since `PipeIo` does not contain any references
    /// with lifetime `'a`
    _lifetime_constraint: PhantomLifetime<'a>,
}

impl<'a> PipeIo<'a> {
    /// Create a new `PipeIo` instance using the given device and pipe ID.
    ///
    /// The lifetime of the `PipeIo` instance is tied to the lifetime of the `Device` instance;
    /// the `PipeIo` instance cannot outlive the `Device` instance.
    ///
    /// For improved ergonomics it is recommended to use [`Device::pipe`] instead of this method.
    #[must_use]
    pub fn new(device: &'a Device, id: Pipe) -> Self {
        Self {
            handle: device.handle(),
            id,
            _lifetime_constraint: PhantomData,
        }
    }

    /// Get the pipe ID.
    #[must_use]
    pub fn id(&self) -> Pipe {
        self.id
    }

    /// Get the descriptor for this endpoint.
    pub fn descriptor(&self) -> Result<PipeInfo> {
        // FT60x devices have 2 interfaces, and 0 is reserved.
        // Page 33: https://ftdichip.com/wp-content/uploads/2020/07/AN_379-D3xx-Programmers-Guide-1.pdf
        const INTERFACE_INDEX: c_uchar = 1;
        let mut info = ffi::FT_PIPE_INFORMATION::default();
        try_d3xx!(unsafe {
            ffi::FT_GetPipeInformation(self.handle, INTERFACE_INDEX, u8::from(self.id), &mut info)
        })?;
        PipeInfo::new(info)
    }

    /// Set the stream size for this pipe.
    ///
    /// If `size` is `None` then streaming is disabled. Otherwise,
    /// the pipe will be configured for streaming with the given size.
    ///
    /// Stream pipes are general-purpose pipes supporting interrupt, bulk,
    /// and isochronous transfers.
    pub fn set_stream_size(&self, size: Option<usize>) -> Result<()> {
        match size {
            Some(size) => {
                try_d3xx!(unsafe {
                    ffi::FT_SetStreamPipe(
                        self.handle,
                        c_uchar::from(false),
                        c_uchar::from(false),
                        self.id as c_uchar,
                        size.try_into().or(Err(D3xxError::InvalidArgs))?,
                    )
                })
            }
            None => {
                try_d3xx!(unsafe {
                    ffi::FT_ClearStreamPipe(
                        self.handle,
                        c_uchar::from(false),
                        c_uchar::from(false),
                        self.id as c_uchar,
                    )
                })
            }
        }
    }

    /// Aborts all pending transfers.
    ///
    /// There is no guarantee that the device will not send/receive previously-transmitted data
    /// after this method is called.
    ///
    /// It is recommended to call this method
    pub fn abort(&self) -> Result<()> {
        try_d3xx!(unsafe { ffi::FT_AbortPipe(self.handle, u8::from(self.id)) })
    }

    /// Aborts all pending transfers on the specified pipe if the given result is an error.
    ///
    /// This is a convenience method for aborting a pipe on read/write failure, as required
    /// by the driver. See D3XX Programmer's Guide, pg. 15 for more information.
    ///
    /// Returns the given result for convenience.
    fn maybe_abort<T>(&self, res: Result<T>) -> Result<T> {
        res.map_err(|e| {
            let _ = self.abort();
            e
        })
    }

    /// Get the timeout in milliseconds for the specified pipe.
    pub fn timeout(&self) -> Result<u32> {
        let mut timeout = 0;
        try_d3xx!(unsafe { ffi::FT_GetPipeTimeout(self.handle, u8::from(self.id), &mut timeout) })?;
        Ok(timeout)
    }

    /// Set the timeout in milliseconds for the specified pipe.
    pub fn set_timeout(&self, timeout: u32) -> Result<()> {
        try_d3xx!(unsafe { ffi::FT_SetPipeTimeout(self.handle, u8::from(self.id), timeout) })
    }

    /// Asynchronous read into the given buffer.
    ///
    /// On success the number of bytes read is returned.
    ///
    /// # Panics
    ///
    /// Panics if `buf.len()` exceeds [`std::ffi::c_ulong::MAX`]
    pub async fn read_async(&self, buf: &mut [u8]) -> Result<usize> {
        let mut overlapped = Overlapped::new(self.handle)?;
        self.maybe_abort(ffi::util::read_pipe_async(
            self.handle,
            u8::from(self.id),
            buf,
            overlapped.inner_mut(),
        ))?;
        overlapped.await
    }

    /// Asynchronous write.
    ///
    /// On success the number of bytes written is returned.
    ///
    /// # Panics
    ///
    /// Panics if `buf.len()` exceeds [`std::ffi::c_ulong::MAX`]
    pub async fn write_async(&self, buf: &[u8]) -> Result<usize> {
        let mut overlapped = Overlapped::new(self.handle)?;
        self.maybe_abort(ffi::util::write_pipe_async(
            self.handle,
            u8::from(self.id),
            buf,
            overlapped.inner_mut(),
        ))?;
        overlapped.await
    }
}

impl<'a> Write for PipeIo<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let res = ffi::util::write_pipe(self.handle, u8::from(self.id), buf);
        Ok(self.maybe_abort(res)?)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        try_d3xx!(unsafe { ffi::FT_FlushPipe(self.handle, u8::from(self.id)) })?;
        Ok(())
    }
}

impl<'a> Read for PipeIo<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = ffi::util::read_pipe(self.handle, u8::from(self.id), buf);
        Ok(self.maybe_abort(res)?)
    }
}

/// Identifies a unique read/write endpoint on a device.
///
/// D3XX devices have 4 input and 4 output endpoints. The direction of the endpoint is
/// relative to the host, rather than the device. In other words, an input endpoint is used
/// to read data from the device, and an output endpoint is used to write data to the device.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Pipe {
    /// Input pipe 0.
    In0 = 0x82,
    /// Input pipe 1.
    In1 = 0x83,
    /// Input pipe 2.
    In2 = 0x84,
    /// Input pipe 3.
    In3 = 0x85,
    /// Output pipe 0.
    Out0 = 0x02,
    /// Output pipe 1.
    Out1 = 0x03,
    /// Output pipe 2.
    Out2 = 0x04,
    /// Output pipe 3.
    Out3 = 0x05,
}

impl Pipe {
    /// Check if the pipe is an input (read) pipe.
    #[inline]
    #[must_use]
    pub fn is_in(self) -> bool {
        !self.is_out()
    }

    /// Check if the pipe is an output (write) pipe.
    #[inline]
    #[must_use]
    pub fn is_out(self) -> bool {
        (self as u8) & 0x80 == 0
    }
}

/// The type of a pipe.
///
/// This is used to determine the type of transfer to use.
///
/// # References
/// - <https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__endpoints.html>
#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PipeType {
    /// Bidirectional control transfer.
    ///
    /// Reserved for the host to send/request configuration information using endpoint zero.
    Control = 0,
    /// Isochronous transfer.
    ///
    /// Used for time-critical data transfers where data integrity is not critical.
    Isochronous = 1,
    /// Bulk transfer.
    ///
    /// Used for miscellaneous transfers where data integrity is required.
    Bulk = 2,
    /// Interrupt transfer.
    ///
    /// Used in cases where polling intervals are defined.
    Interrupt = 3,
}

impl From<ffi::FT_PIPE_TYPE> for PipeType {
    fn from(value: ffi::FT_PIPE_TYPE) -> Self {
        match value {
            ffi::FT_PIPE_TYPE::FTPipeTypeControl => Self::Control,
            ffi::FT_PIPE_TYPE::FTPipeTypeIsochronous => Self::Isochronous,
            ffi::FT_PIPE_TYPE::FTPipeTypeBulk => Self::Bulk,
            ffi::FT_PIPE_TYPE::FTPipeTypeInterrupt => Self::Interrupt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeid_try_from() {
        assert_eq!(Pipe::try_from(0x82), Ok(Pipe::In0));
        assert_eq!(Pipe::try_from(0x83), Ok(Pipe::In1));
        assert_eq!(Pipe::try_from(0x84), Ok(Pipe::In2));
        assert_eq!(Pipe::try_from(0x85), Ok(Pipe::In3));
        assert_eq!(Pipe::try_from(0x02), Ok(Pipe::Out0));
        assert_eq!(Pipe::try_from(0x03), Ok(Pipe::Out1));
        assert_eq!(Pipe::try_from(0x04), Ok(Pipe::Out2));
        assert_eq!(Pipe::try_from(0x05), Ok(Pipe::Out3));
        assert!(Pipe::try_from(0x00).is_err());
        assert!(Pipe::try_from(0x01).is_err());
        assert!(Pipe::try_from(0x06).is_err());
        assert!(Pipe::try_from(0x81).is_err());
        assert!(Pipe::try_from(0x86).is_err());
        assert!(Pipe::try_from(0xFF).is_err());
    }

    #[test]
    fn pipe_is_in() {
        assert!(Pipe::In0.is_in());
        assert!(Pipe::In1.is_in());
        assert!(Pipe::In2.is_in());
        assert!(Pipe::In3.is_in());
        assert!(!Pipe::Out0.is_in());
        assert!(!Pipe::Out1.is_in());
        assert!(!Pipe::Out2.is_in());
        assert!(!Pipe::Out3.is_in());
    }

    #[test]
    fn pipe_is_out() {
        assert!(!Pipe::In0.is_out());
        assert!(!Pipe::In1.is_out());
        assert!(!Pipe::In2.is_out());
        assert!(!Pipe::In3.is_out());
        assert!(Pipe::Out0.is_out());
        assert!(Pipe::Out1.is_out());
        assert!(Pipe::Out2.is_out());
        assert!(Pipe::Out3.is_out());
    }
}
