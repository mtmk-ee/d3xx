use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{D3xxError, Result};

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum DriveStrength {
    Ohm50,
    Ohm35,
    Ohm25,
    Ohm18,
}

pub struct PinDriveStrengths {
    fifo_data: DriveStrength,
    fifo_clock: DriveStrength,
    gpio0: DriveStrength,
    gpio1: DriveStrength,
}

impl PinDriveStrengths {
    pub fn new(msio: u32, gpio: u32) -> Result<Self> {
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

    pub fn fifo_data(&self) -> &DriveStrength {
        &self.fifo_data
    }

    pub fn fifo_clock(&self) -> &DriveStrength {
        &self.fifo_clock
    }

    pub fn gpio0(&self) -> &DriveStrength {
        &self.gpio0
    }

    pub fn gpio1(&self) -> &DriveStrength {
        &self.gpio1
    }
}
