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
            0x82 => Ok(Pipe::In0),
            0x83 => Ok(Pipe::In1),
            0x84 => Ok(Pipe::In2),
            0x85 => Ok(Pipe::In3),
            0x02 => Ok(Pipe::Out0),
            0x03 => Ok(Pipe::Out1),
            0x04 => Ok(Pipe::Out2),
            0x05 => Ok(Pipe::Out3),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PipeType {
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

impl TryFrom<i32> for PipeType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PipeType::Control),
            1 => Ok(PipeType::Isochronous),
            2 => Ok(PipeType::Bulk),
            3 => Ok(PipeType::Interrupt),
            _ => Err(()),
        }
    }
}

impl Pipe {
    /// Check if the pipe is an input (read) pipe.
    #[inline]
    pub fn is_in(self) -> bool {
        !self.is_out()
    }

    /// Check if the pipe is an output (write) pipe.
    #[inline]
    pub fn is_out(self) -> bool {
        (self as u8) & 0x80 == 0
    }
}

pub struct PipeInfo {
    pipe_type: PipeType,
    pipe: Pipe,
    max_packet_size: usize,
    interval: u8,
}

impl PipeInfo {
    /// The type of pipe.
    pub fn pipe_type(&self) -> PipeType {
        self.pipe_type
    }

    /// The pipe identifier.
    pub fn pipe(&self) -> Pipe {
        self.pipe
    }

    /// The maximum packet size in bytes.
    pub fn max_packet_size(&self) -> usize {
        self.max_packet_size
    }

    /// The polling interval in milliseconds.
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StreamPipes {
    pipes: HashMap<Pipe, usize>,
}

impl StreamPipes {
    /// Create a new empty set.
    pub fn none() -> Self {
        Self::default()
    }

    /// Add a pipe to the set of stream pipes.
    ///
    /// If a pipe of the same variant already exists the stream size
    /// will be updated.
    pub fn with_pipe(mut self, pipe: Pipe, stream_size: usize) -> Self {
        self.pipes.insert(pipe.into(), stream_size);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Pipe, &usize)> {
        self.pipes.iter()
    }
}

impl Default for StreamPipes {
    fn default() -> Self {
        Self {
            pipes: HashMap::new(),
        }
    }
}

impl IntoIterator for StreamPipes {
    type Item = (Pipe, usize);
    type IntoIter = std::collections::hash_map::IntoIter<Pipe, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.pipes.into_iter()
    }
}
