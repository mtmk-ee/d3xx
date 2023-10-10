use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{D3xxError, Result};

pub struct DataTransferConfig {
    fifo_clock: FifoClock,
    fifo_mode: FifoMode,
    channel_config: ChannelConfiguration,
}

impl DataTransferConfig {
    pub fn new(fifo_clock: u8, fifo_mode: u8, channel_config: u8) -> Result<Self> {
        Ok(Self {
            fifo_clock: FifoClock::try_from(fifo_clock).or(Err(D3xxError::OtherError))?,
            fifo_mode: FifoMode::try_from(fifo_mode).or(Err(D3xxError::OtherError))?,
            channel_config: ChannelConfiguration::try_from(channel_config)
                .or(Err(D3xxError::OtherError))?,
        })
    }

    pub fn fifo_clock(&self) -> &FifoClock {
        &self.fifo_clock
    }

    pub fn fifo_mode(&self) -> &FifoMode {
        &self.fifo_mode
    }

    pub fn channel_config(&self) -> &ChannelConfiguration {
        &self.channel_config
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum FifoMode {
    Mode245,
    Mode600,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum FifoClock {
    Clock100Mhz,
    Clock66Mhz,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ChannelConfiguration {
    /// Four OUT and four IN pipes.
    Four,
    /// Two OUT and two IN pipes.
    Two,
    /// One OUT and one IN pipe.
    One,
    /// One OUT pipe only.
    OneOutPipe,
    /// One IN pipe only.
    OneInPipe,
}
