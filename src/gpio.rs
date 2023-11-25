//! Provides access to the GPIO pins of a [`Device`].
//!
//! A [`Gpio`] instance may be obtained using [`Device::gpio`].
//! The `Gpio` struct provides methods to enable the GPIO pins, set the GPIO
//! direction, set the GPIO pull resistors, and read/write the GPIO pins.

use std::marker::PhantomData;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::ffi;
use crate::util::PhantomLifetime;
use crate::{try_d3xx, Device, Result};

/// Provides read/write access to GPIO pins of the chip.
///
/// The function of the pins is determined by the chip configuration. As this crate
/// does not provide a means to configure the chip, it is recommended to use
/// the [FT60X Chip Configuration Programmer](https://ftdichip.com/utilities/) for
/// configuration.
///
/// The lifetime of the `Gpio` instance is tied to the lifetime of the `Device` instance;
/// the device cannot be closed while the `Gpio` instance is in use.
pub struct Gpio<'a> {
    handle: ffi::FT_HANDLE,
    pin: GpioPin,
    /// Ties the lifetime of this struct to the lifetime of the source [`Device`](crate::Device) instance.
    _lifetime_marker: PhantomLifetime<'a>,
}

impl<'a> Gpio<'a> {
    /// Create a new `Gpio` instance using the given device and GPIO pin.
    pub(crate) fn new(device: &'a Device, pin: GpioPin) -> Self {
        Self {
            handle: device.handle(),
            pin,
            _lifetime_marker: PhantomData,
        }
    }

    /// Enable the GPIO in the given direction.
    ///
    /// The D3XX API does not provide a way to disable GPIO pins.
    /// However, the direction of the GPIO may be changed at any time, and
    /// may be set to [`Direction::Input`] to effectively prevent writing
    /// to the GPIO.
    pub fn enable(&self, direction: Direction) -> Result<()> {
        try_d3xx!(unsafe {
            ffi::FT_EnableGPIO(
                self.handle,
                1u32 << u8::from(self.pin),
                u32::from(u8::from(direction) << u8::from(self.pin)),
            )
        })
    }

    /// Set internal GPIO pull-up/pull-down resistors.
    ///
    /// Only available for Rev. B parts or later.
    pub fn set_pull(&self, pull: PullMode) -> Result<()> {
        try_d3xx!(unsafe {
            ffi::FT_SetGPIOPull(
                self.handle,
                1u32 << u8::from(self.pin),
                u32::from(u8::from(pull) << u8::from(self.pin)),
            )
        })
    }

    /// Set the status of the GPIO.
    pub fn write(&self, level: Level) -> Result<()> {
        try_d3xx!(unsafe {
            ffi::FT_WriteGPIO(
                self.handle,
                1u32 << u8::from(self.pin),
                u32::from(u8::from(level) << u8::from(self.pin)),
            )
        })
    }

    /// Read the status of the GPIO.
    #[allow(clippy::missing_panics_doc)]
    pub fn read(&self) -> Result<Level> {
        let mut value: u32 = 0;
        try_d3xx!(unsafe { ffi::FT_ReadGPIO(self.handle, &mut value) })?;
        let bit = ((value >> u8::from(self.pin)) & 1) as u8;
        // unwrap(): value is guaranteed to be 0 or 1, so there is a matching `Level` variant.
        Ok(Level::try_from(bit).unwrap())
    }
}

/// GPIO pin, either `Pin0` or `Pin1`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum GpioPin {
    /// GPIO pin 0.
    ///
    /// This is referred to as GPIO0 in the datasheet.
    Pin0 = 0,
    /// GPIO pin 1.
    ///
    /// This is referred to as GPIO1 in the datasheet.
    Pin1 = 1,
}

/// GPIO direction.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Direction {
    /// Input direction. The GPIO may be read, but not written.
    Input = 0,
    /// Output direction. The GPIO may be written, but not read.
    Output = 1,
}

/// GPIO level.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Level {
    /// Low level (0).
    Low = 0,
    /// High level (1).
    High = 1,
}

/// GPIO pull mode.
///
/// This can be configured once opening a device.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PullMode {
    /// 50 kOhm pull-down.
    PullDown = 0,
    /// High impedance.
    HighImpedance = 1,
    /// 50 kOhm pull-up.
    PullUp = 2,
}
