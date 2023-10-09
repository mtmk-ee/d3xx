use std::collections::HashMap;

use crate::{ffi, D3xxError, Result};

/// Identifies a unique read/write pipe on a device.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Pipe {
    In0 = 0x82,
    In1 = 0x83,
    In2 = 0x84,
    In3 = 0x85,
    Out0 = 0x02,
    Out1 = 0x03,
    Out2 = 0x04,
    Out3 = 0x05,
}

impl TryFrom<u8> for Pipe {
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

/// Information about a pipe on a device.
///
/// This is returned by [`Device::pipe_info`].
#[allow(clippy::module_name_repetitions)]
pub struct PipeInfo {
    pipe_type: PipeType,
    pipe: Pipe,
    max_packet_size: usize,
    interval: u8,
}

impl PipeInfo {
    /// The type of pipe.
    #[must_use]
    pub fn pipe_type(&self) -> PipeType {
        self.pipe_type
    }

    /// The pipe identifier.
    #[must_use]
    pub fn pipe(&self) -> Pipe {
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
            pipe: Pipe::try_from(value.PipeId).or(Err(D3xxError::OtherError))?,
            max_packet_size: value.MaximumPacketSize as usize,
            interval: value.Interval,
        })
    }
}

/// A set of pipes to be used for stream transfers.
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
///    StreamPipes::default().with_pipe(Pipe::In1, 1024)
/// ).unwrap();
///
/// // Enable streaming on several pipes
/// device.set_stream_pipes(
///   StreamPipes::default()
///     .with_pipe(Pipe::In1, 1024)
///     .with_pipe(Pipe::In2, 1024)
///     .with_pipe(Pipe::Out1, 1024)
/// ).unwrap();
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct StreamPipes {
    pipes: HashMap<Pipe, usize>,
}

impl StreamPipes {
    /// Create a new empty set.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Add a pipe to the set of stream pipes.
    ///
    /// If a pipe of the same variant already exists the stream size
    /// will be updated.
    #[must_use]
    pub fn with_pipe(mut self, pipe: Pipe, stream_size: usize) -> Self {
        self.pipes.insert(pipe, stream_size);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Pipe, &usize)> {
        self.pipes.iter()
    }
}

impl IntoIterator for StreamPipes {
    type Item = (Pipe, usize);
    type IntoIter = std::collections::hash_map::IntoIter<Pipe, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.pipes.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_try_from() {
        assert_eq!(Pipe::try_from(0x82), Ok(Pipe::In0));
        assert_eq!(Pipe::try_from(0x83), Ok(Pipe::In1));
        assert_eq!(Pipe::try_from(0x84), Ok(Pipe::In2));
        assert_eq!(Pipe::try_from(0x85), Ok(Pipe::In3));
        assert_eq!(Pipe::try_from(0x02), Ok(Pipe::Out0));
        assert_eq!(Pipe::try_from(0x03), Ok(Pipe::Out1));
        assert_eq!(Pipe::try_from(0x04), Ok(Pipe::Out2));
        assert_eq!(Pipe::try_from(0x05), Ok(Pipe::Out3));
        assert_eq!(Pipe::try_from(0x00), Err(()));
        assert_eq!(Pipe::try_from(0x01), Err(()));
        assert_eq!(Pipe::try_from(0x06), Err(()));
        assert_eq!(Pipe::try_from(0x81), Err(()));
        assert_eq!(Pipe::try_from(0x86), Err(()));
        assert_eq!(Pipe::try_from(0xFF), Err(()));
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
        assert_eq!(info.pipe(), Pipe::In0);
        assert_eq!(info.max_packet_size(), 64);
        assert_eq!(info.interval(), 0);
    }

    #[test]
    fn stream_pipes_with_pipe() {
        let pipes = StreamPipes::default()
            .with_pipe(Pipe::In1, 1024)
            .with_pipe(Pipe::In2, 1024)
            .with_pipe(Pipe::Out1, 1024);
        assert_eq!(pipes.pipes.len(), 3);
        assert_eq!(pipes.pipes[&Pipe::In1], 1024);
        assert_eq!(pipes.pipes[&Pipe::In2], 1024);
        assert_eq!(pipes.pipes[&Pipe::Out1], 1024);
    }

    #[test]
    fn stream_pipes_with_pipe_overwrite() {
        let pipes = StreamPipes::default()
            .with_pipe(Pipe::In1, 1024)
            .with_pipe(Pipe::In1, 2048);
        assert_eq!(pipes.pipes.len(), 1);
        assert_eq!(pipes.pipes[&Pipe::In1], 2048);
    }

    #[test]
    fn stream_pipes_default() {
        let pipes = StreamPipes::default();
        assert_eq!(pipes.pipes.len(), 0);
    }

    #[test]
    fn stream_pipes_none() {
        let pipes = StreamPipes::none();
        assert_eq!(pipes.pipes.len(), 0);
    }

    #[test]
    fn stream_pipes_iter() {
        let pipes = StreamPipes::default()
            .with_pipe(Pipe::In1, 1024)
            .with_pipe(Pipe::In2, 1025)
            .with_pipe(Pipe::Out1, 1026);

        // cannot guarantee order of iteration
        assert!(pipes
            .iter()
            .any(|(&pipe, &size)| pipe == Pipe::In1 && size == 1024));
        assert!(pipes
            .iter()
            .any(|(&pipe, &size)| pipe == Pipe::In2 && size == 1025));
        assert!(pipes
            .iter()
            .any(|(&pipe, &size)| pipe == Pipe::Out1 && size == 1026));
    }
}
