use std::collections::HashSet;

use crate::{ffi, D3xxError, Result};


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
    #[inline]
    pub fn is_in(self) -> bool {
        !self.is_out()
    }

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
    pub fn pipe_type(&self) -> PipeType {
        self.pipe_type
    }

    pub fn pipe(&self) -> Pipe {
        self.pipe
    }

    pub fn max_packet_size(&self) -> usize {
        self.max_packet_size
    }

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StreamPipes {
    pipes: HashSet<(Pipe, usize)>,
}

impl StreamPipes {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn with_pipe(mut self, pipe: Pipe, stream_size: usize) -> Self {
        self.pipes.insert((pipe.into(), stream_size));
        self
    }
}

impl Default for StreamPipes {
    fn default() -> Self {
        Self {
            pipes: HashSet::new(),
        }
    }
}

impl IntoIterator for StreamPipes {
    type Item = (Pipe, usize);
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pipes.into_iter()
    }
}
