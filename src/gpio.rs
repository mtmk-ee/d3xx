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

/// GPIO access.
///
/// This struct is used to access the GPIO pins of a [`Device`].
pub struct Gpio<'a> {
    handle: ffi::FT_HANDLE,
    pin: GpioPin,
    _lifetime_marker: PhantomLifetime<'a>,
}

impl<'a> Gpio<'a> {
    /// Create a new `Gpio` instance using the given device and GPIO pin.
    ///
    /// The lifetime of the `Gpio` instance is tied to the lifetime of the `Device` instance.
    pub(crate) fn new(device: &'a Device, pin: GpioPin) -> Self {
        Self {
            handle: device.handle(),
            pin,
            _lifetime_marker: PhantomData,
        }
    }

    /// Enable the GPIO in the given direction.
    ///
    /// Once enabled, the GPIO cannot be disabled.
    pub fn enable(&self, direction: Direction) -> Result<()> {
        try_d3xx!(unsafe {
            ffi::FT_EnableGPIO(
                self.handle,
                1u32 << u8::from(self.pin),
                u32::from(u8::from(direction) << u8::from(self.pin)),
            )
        })
    }

    /// Set internal GPIO pull resistors.
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

    /// Return the status of the GPIO.
    #[allow(clippy::missing_panics_doc)]
    pub fn read(&self) -> Result<Level> {
        let mut value: u32 = 0;
        try_d3xx!(unsafe { ffi::FT_ReadGPIO(self.handle, &mut value) })?;
        // unwrap(): value is guaranteed to be 0 or 1, so there is a matching `Level` variant.
        Ok(Level::try_from(((value >> u8::from(self.pin)) & 1) as u8).unwrap())
    }
}

/// GPIO pin, either `Pin0` or `Pin1`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum GpioPin {
    /// GPIO pin 0.
    Pin0 = 0,
    /// GPIO pin 1.
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
