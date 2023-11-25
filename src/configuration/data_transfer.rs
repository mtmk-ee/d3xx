use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{D3xxError, Result};

/// Configuration regarding data transfer.
///
/// FIFOs are used to transfer data to and from the chip. The data transfer configuration
/// indicates the behavior of the FIFOs. This configuration contains:
///
/// - FIFO clock speed
/// - FIFO mode
/// - Channel configuration
pub struct DataTransferConfig {
    fifo_clock: FifoClock,
    fifo_mode: FifoMode,
    channel_config: ChannelConfiguration,
}

impl DataTransferConfig {
    pub(crate) fn new(fifo_clock: u8, fifo_mode: u8, channel_config: u8) -> Result<Self> {
        Ok(Self {
            fifo_clock: FifoClock::try_from(fifo_clock).or(Err(D3xxError::OtherError))?,
            fifo_mode: FifoMode::try_from(fifo_mode).or(Err(D3xxError::OtherError))?,
            channel_config: ChannelConfiguration::try_from(channel_config)
                .or(Err(D3xxError::OtherError))?,
        })
    }

    /// Get the FIFO clock speed.
    #[must_use]
    pub fn fifo_clock(&self) -> &FifoClock {
        &self.fifo_clock
    }

    /// Get the FIFO mode.
    #[must_use]
    pub fn fifo_mode(&self) -> &FifoMode {
        &self.fifo_mode
    }

    /// Get the channel configuration.
    #[must_use]
    pub fn channel_config(&self) -> &ChannelConfiguration {
        &self.channel_config
    }
}

/// FIFO mode.
///
/// This mode represents the configuration of the FIFOs on the chip in various modes.
/// `Mode600` is the default mode, representing the behavior of FT600 FIFOs.
/// `Mode245` represents the behavior of FT245 FIFOs.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum FifoMode {
    /// 245 FIFO mode.
    Mode245,
    /// 600 FIFO mode (default).
    Mode600,
}

/// Clock speed of the FIFOs.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum FifoClock {
    /// 100 MHz clock speed.
    Clock100Mhz,
    /// 66 MHz clock speed.
    Clock66Mhz,
}

/// Channel configuration.
///
/// A "channel" is a pair of pipes. Each channel has one IN pipe and one OUT pipe,
/// except for `OneOutPipe` and `OneInPipe`, which have only one pipe.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
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
