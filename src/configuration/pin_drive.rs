use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{D3xxError, Result};

/// Drive strength configuration for a GPIO/FIFO.
///
/// The drive strength configuration indicates the load driven by the GPIO/FIFO.
/// This should be appropriately configured to avoid voltage deviation.
#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum DriveStrength {
    /// 50-Ohm drive strength.
    Ohm50,
    /// 25-Ohm drive strength.
    Ohm35,
    /// 18-Ohm drive strength.
    Ohm25,
    /// 15-Ohm drive strength.
    Ohm18,
}

/// Pin drive strengths.
///
/// The pin drive strengths contain:
/// - GPIO drive strengths
/// - MSIO (FIFO) clock/data drive strengths
pub struct PinDriveStrengths {
    fifo_data: DriveStrength,
    fifo_clock: DriveStrength,
    gpio0: DriveStrength,
    gpio1: DriveStrength,
}

impl PinDriveStrengths {
    pub(crate) fn new(msio: u32, gpio: u32) -> Result<Self> {
        Ok(Self {
            fifo_data: DriveStrength::try_from((msio & 0b11) as u8)
                .or(Err(D3xxError::OtherError))?,
            fifo_clock: DriveStrength::try_from(((msio >> 4) & 0b11) as u8)
                .or(Err(D3xxError::OtherError))?,
            gpio0: DriveStrength::try_from(((gpio >> 8) & 0b11) as u8)
                .or(Err(D3xxError::OtherError))?,
            gpio1: DriveStrength::try_from(((gpio >> 10) & 0b11) as u8)
                .or(Err(D3xxError::OtherError))?,
        })
    }

    /// Get the FIFO data drive strength.
    #[must_use]
    pub fn fifo_data(&self) -> &DriveStrength {
        &self.fifo_data
    }

    /// Get the FIFO clock drive strength.
    #[must_use]
    pub fn fifo_clock(&self) -> &DriveStrength {
        &self.fifo_clock
    }

    /// Get the GPIO0 drive strength.
    #[must_use]
    pub fn gpio0(&self) -> &DriveStrength {
        &self.gpio0
    }

    /// Get the GPIO1 drive strength.
    #[must_use]
    pub fn gpio1(&self) -> &DriveStrength {
        &self.gpio1
    }
}
