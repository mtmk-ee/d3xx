use std::{
    ffi::c_uchar,
    io::{Read, Write},
    marker::PhantomData,
};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    descriptor::PipeInfo, ffi, overlapped::Overlapped, try_d3xx, D3xxError, Device, Result,
};

type PhantomLifetime<'a> = PhantomData<&'a ()>;

/// A read/write pipe on a device.
///
/// This struct implements [`Read`] and [`Write`], so it can be used with
/// the standard library's I/O functions.
///
/// # Examples
///
/// ```no_run
/// use d3xx::{Device, Pipe, PipeId};
///
/// let device = Device::open("ABC123").unwrap();
///
/// // Write to output pipe 1
/// let mut buf = vec![0u8; 1024];
/// device
///    .pipe(PipeId::Out1)
///    .write(&buf)
///    .unwrap();
/// ```
pub struct Pipe<'a> {
    handle: ffi::FT_HANDLE,
    id: PipeId,
    _lifetime_constraint: PhantomLifetime<'a>,
}

impl<'a> Pipe<'a> {
    pub(crate) fn new(device: &'a Device, id: PipeId) -> Self {
        Self {
            handle: device.handle(),
            id,
            _lifetime_constraint: PhantomData,
        }
    }

    /// Get the pipe ID.
    #[must_use]
    pub fn id(&self) -> PipeId {
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

    /// Aborts all pending transfers
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

impl<'a> Write for Pipe<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let res = ffi::util::write_pipe(self.handle, u8::from(self.id), buf);
        Ok(self.maybe_abort(res)?)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        try_d3xx!(unsafe { ffi::FT_FlushPipe(self.handle, u8::from(self.id)) })?;
        Ok(())
    }
}

impl<'a> Read for Pipe<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = ffi::util::read_pipe(self.handle, u8::from(self.id), buf);
        Ok(self.maybe_abort(res)?)
    }
}

/// Identifies a unique read/write pipe on a device.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PipeId {
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

impl TryFrom<ffi::FT_PIPE_TYPE> for PipeType {
    type Error = ();

    fn try_from(value: ffi::FT_PIPE_TYPE) -> Result<Self, Self::Error> {
        match value {
            ffi::FT_PIPE_TYPE::FTPipeTypeControl => Ok(Self::Control),
            ffi::FT_PIPE_TYPE::FTPipeTypeIsochronous => Ok(Self::Isochronous),
            ffi::FT_PIPE_TYPE::FTPipeTypeBulk => Ok(Self::Bulk),
            ffi::FT_PIPE_TYPE::FTPipeTypeInterrupt => Ok(Self::Interrupt),
        }
    }
}

impl PipeId {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeid_try_from() {
        assert_eq!(PipeId::try_from(0x82), Ok(PipeId::In0));
        assert_eq!(PipeId::try_from(0x83), Ok(PipeId::In1));
        assert_eq!(PipeId::try_from(0x84), Ok(PipeId::In2));
        assert_eq!(PipeId::try_from(0x85), Ok(PipeId::In3));
        assert_eq!(PipeId::try_from(0x02), Ok(PipeId::Out0));
        assert_eq!(PipeId::try_from(0x03), Ok(PipeId::Out1));
        assert_eq!(PipeId::try_from(0x04), Ok(PipeId::Out2));
        assert_eq!(PipeId::try_from(0x05), Ok(PipeId::Out3));
        assert!(PipeId::try_from(0x00).is_err());
        assert!(PipeId::try_from(0x01).is_err());
        assert!(PipeId::try_from(0x06).is_err());
        assert!(PipeId::try_from(0x81).is_err());
        assert!(PipeId::try_from(0x86).is_err());
        assert!(PipeId::try_from(0xFF).is_err());
    }

    #[test]
    fn pipe_is_in() {
        assert!(PipeId::In0.is_in());
        assert!(PipeId::In1.is_in());
        assert!(PipeId::In2.is_in());
        assert!(PipeId::In3.is_in());
        assert!(!PipeId::Out0.is_in());
        assert!(!PipeId::Out1.is_in());
        assert!(!PipeId::Out2.is_in());
        assert!(!PipeId::Out3.is_in());
    }

    #[test]
    fn pipe_is_out() {
        assert!(!PipeId::In0.is_out());
        assert!(!PipeId::In1.is_out());
        assert!(!PipeId::In2.is_out());
        assert!(!PipeId::In3.is_out());
        assert!(PipeId::Out0.is_out());
        assert!(PipeId::Out1.is_out());
        assert!(PipeId::Out2.is_out());
        assert!(PipeId::Out3.is_out());
    }
}
