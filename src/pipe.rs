use std::{
    ffi::c_uchar,
    io::{Read, Write},
    marker::PhantomData,
};

use crate::{ffi, overlapped::Overlapped, try_d3xx, D3xxError, Device, Result};

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
    id: u8,
    _lifetime_constraint: PhantomLifetime<'a>,
}

impl<'a> Pipe<'a> {
    pub(crate) fn new(device: &'a Device, id: PipeId) -> Self {
        Self {
            handle: device.handle(),
            id: id as u8,
            _lifetime_constraint: PhantomData,
        }
    }

    #[must_use]
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Get the descriptor for this endpoint.
    pub fn descriptor(&self) -> Result<PipeInfo> {
        // FT60x devices have 2 interfaces, and 0 is reserved.
        // Page 33: https://ftdichip.com/wp-content/uploads/2020/07/AN_379-D3xx-Programmers-Guide-1.pdf
        const INTERFACE_INDEX: c_uchar = 1;
        let mut info = ffi::FT_PIPE_INFORMATION::default();
        try_d3xx!(unsafe {
            ffi::FT_GetPipeInformation(self.handle, INTERFACE_INDEX, self.id, &mut info)
        })?;
        PipeInfo::try_from(info)
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
        try_d3xx!(unsafe { ffi::FT_AbortPipe(self.handle, self.id) })
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
        try_d3xx!(unsafe { ffi::FT_GetPipeTimeout(self.handle, self.id, &mut timeout) })?;
        Ok(timeout)
    }

    /// Set the timeout in milliseconds for the specified pipe.
    pub fn set_timeout(&self, timeout: u32) -> Result<()> {
        try_d3xx!(unsafe { ffi::FT_SetPipeTimeout(self.handle, self.id, timeout) })
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
            self.id,
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
            self.id,
            buf,
            overlapped.inner_mut(),
        ))?;
        overlapped.await
    }
}

impl<'a> Write for Pipe<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let res = ffi::util::write_pipe(self.handle, self.id, buf);
        Ok(self.maybe_abort(res)?)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        try_d3xx!(unsafe { ffi::FT_FlushPipe(self.handle, self.id) })?;
        Ok(())
    }
}

impl<'a> Read for Pipe<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = ffi::util::read_pipe(self.handle, self.id, buf);
        Ok(self.maybe_abort(res)?)
    }
}

/// Identifies a unique read/write pipe on a device.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum PipeId {
    In0 = 0x82,
    In1 = 0x83,
    In2 = 0x84,
    In3 = 0x85,
    Out0 = 0x02,
    Out1 = 0x03,
    Out2 = 0x04,
    Out3 = 0x05,
}

impl TryFrom<u8> for PipeId {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x82 => Ok(Self::In0),
            0x83 => Ok(Self::In1),
            0x84 => Ok(Self::In2),
            0x85 => Ok(Self::In3),
            0x02 => Ok(Self::Out0),
            0x03 => Ok(Self::Out1),
            0x04 => Ok(Self::Out2),
            0x05 => Ok(Self::Out3),
            _ => Err(()),
        }
    }
}

/// The type of a pipe.
///
/// This is used to determine the type of transfer to use.
#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PipeType {
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
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

/// Information about a pipe on a device.
///
/// This is returned by [`Device::pipe_info`].
#[allow(clippy::module_name_repetitions)]
pub struct PipeInfo {
    pipe_type: PipeType,
    pipe: PipeId,
    max_packet_size: usize,
    interval: u8,
}

impl PipeInfo {
    /// The type of pipe.
    #[must_use]
    pub fn pipe_type(&self) -> PipeType {
        self.pipe_type
    }

    /// The pipe ID.
    #[must_use]
    pub fn id(&self) -> PipeId {
        self.pipe
    }

    /// The maximum packet size in bytes.
    #[must_use]
    pub fn max_packet_size(&self) -> usize {
        self.max_packet_size
    }

    /// The polling interval in milliseconds.
    #[must_use]
    pub fn interval(&self) -> u8 {
        self.interval
    }
}

impl TryFrom<ffi::FT_PIPE_INFORMATION> for PipeInfo {
    type Error = D3xxError;

    fn try_from(value: ffi::FT_PIPE_INFORMATION) -> Result<Self, Self::Error> {
        Ok(Self {
            pipe_type: PipeType::try_from(value.PipeType).or(Err(D3xxError::OtherError))?,
            pipe: PipeId::try_from(value.PipeId).or(Err(D3xxError::OtherError))?,
            max_packet_size: value.MaximumPacketSize as usize,
            interval: value.Interval,
        })
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
        assert_eq!(PipeId::try_from(0x00), Err(()));
        assert_eq!(PipeId::try_from(0x01), Err(()));
        assert_eq!(PipeId::try_from(0x06), Err(()));
        assert_eq!(PipeId::try_from(0x81), Err(()));
        assert_eq!(PipeId::try_from(0x86), Err(()));
        assert_eq!(PipeId::try_from(0xFF), Err(()));
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

    #[test]
    fn pipe_info_try_from() {
        let info = ffi::FT_PIPE_INFORMATION {
            PipeType: ffi::FT_PIPE_TYPE::FTPipeTypeControl,
            PipeId: 0x82,
            MaximumPacketSize: 64,
            Interval: 0,
        };
        let info = PipeInfo::try_from(info).unwrap();
        assert_eq!(info.pipe_type(), PipeType::Control);
        assert_eq!(info.id(), PipeId::In0);
        assert_eq!(info.max_packet_size(), 64);
        assert_eq!(info.interval(), 0);
    }
}
