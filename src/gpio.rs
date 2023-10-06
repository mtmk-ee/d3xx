use crate::ffi;
use crate::{try_d3xx, Device, Result};

pub trait GpioExt {
    fn enable_gpio(&self, gpio: Gpio, direction: Direction) -> Result<()>;
    fn write_gpio(&self, gpio: Gpio, level: Level) -> Result<()>;
    fn read_gpio(&self, gpio: Gpio) -> Result<Level>;
    // fn set_gpio_pull_mode(&self, gpio: Gpio, pull_mode: PullMode) -> Result<()>;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Gpio {
    Pin0 = 0,
    Pin1 = 1,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    Input = 0,
    Output = 1,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Level {
    Low = 0,
    High = 1,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum PullMode {
    PullDown = 0,
    HighImpedance = 1,
    PullUp = 2,
}

impl GpioExt for Device {
    fn enable_gpio(&self, gpio: Gpio, direction: Direction) -> Result<()> {
        try_d3xx!(unsafe {
            ffi::FT_EnableGPIO(
                self.handle(),
                gpio as u32,
                (direction as u32) << (gpio as u32),
            )
        })
    }

    fn write_gpio(&self, gpio: Gpio, level: Level) -> Result<()> {
        try_d3xx!(unsafe {
            ffi::FT_WriteGPIO(self.handle(), gpio as u32, (level as u32) << (gpio as u32))
        })
    }

    fn read_gpio(&self, gpio: Gpio) -> Result<Level> {
        let mut value: u32 = 0;
        try_d3xx!(unsafe { ffi::FT_ReadGPIO(self.handle(), &mut value as *mut u32) })?;
        Ok(match (value >> (gpio as u32)) & 1 {
            0 => Level::Low,
            1 => Level::High,
            _ => Err(crate::D3xxError::OtherError)?,
        })
    }
}
