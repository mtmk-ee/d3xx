//! Types and functions for reading chip configurations.
//!
//! Chip configurations can be set using the [FT60X Chip Configuration Programmer](https://ftdichip.com/utilities/).
//! The configuration of a chip is not one-to-one with the USB configuration descriptor, although
//! some of the fields are.
//!
//! The configuration may be read from a device once it is opened. Writing configuration changes
//! to the device is not yet supported. The chip configuration contains a large amount of information about
//! the device, including:
//!
//! - Identification
//! - Power consumption
//! - Pin drive strengths
//! - Optional features
//! - FIFO timing and behavior
//! - Channel configuration

mod data_transfer;
mod notification;
mod optional;
mod pin_drive;
mod power;
mod string_descriptor;

use std::ptr::addr_of_mut;

use crate::{ffi, try_d3xx, Result};
pub use data_transfer::*;
pub use notification::*;
pub use optional::*;
pub use pin_drive::*;
pub use power::*;
pub use string_descriptor::*;

/// `FT60x` chip configuration.
///
/// The configuration may be read from a device once it is opened.
/// Writing configuration changes to the device is not yet supported.
pub struct ChipConfiguration {
    vid: u16,
    pid: u16,
    string_descriptor: StringDescriptor,
    power_config: PowerConfig,
    pin_drive_strength: PinDriveStrengths,
    interrupt_latency: u8,
    data_transfer: DataTransferConfig,
    optional_features: OptionalFeatures,
}

impl ChipConfiguration {
    /// Create a new `ChipConfiguration` instance using the given handle.
    ///
    /// Attempts to read the chip configuration from the device.
    pub(crate) fn new(handle: ffi::FT_HANDLE) -> Result<Self> {
        let mut config: ffi::FT_60XCONFIGURATION = unsafe { std::mem::zeroed() };
        try_d3xx!(unsafe { ffi::FT_GetChipConfiguration(handle, addr_of_mut!(config).cast()) })?;
        Ok(Self {
            vid: config.VendorID,
            pid: config.ProductID,
            string_descriptor: StringDescriptor::new(config.StringDescriptors),
            power_config: PowerConfig::new(config.PowerAttributes, config.PowerConsumption),
            pin_drive_strength: PinDriveStrengths::new(config.MSIO_Control, config.GPIO_Control)?,
            interrupt_latency: config.bInterval,
            data_transfer: DataTransferConfig::new(
                config.FIFOClock,
                config.FIFOMode,
                config.ChannelConfig,
            )?,
            optional_features: OptionalFeatures::new(
                config.OptionalFeatureSupport,
                config.BatteryChargingGPIOConfig,
            ),
        })
    }

    /// Vendor ID.
    #[must_use]
    pub fn vendor_id(&self) -> u16 {
        self.vid
    }

    /// Product ID.
    #[must_use]
    pub fn product_id(&self) -> u16 {
        self.pid
    }

    /// Get the string descriptor for the device.
    ///
    /// The string descriptor contains:
    /// - Manufacturer name
    /// - Product name
    /// - Serial number
    #[must_use]
    pub fn string_descriptor(&self) -> &StringDescriptor {
        &self.string_descriptor
    }

    /// Get a mutable reference to the string descriptor proxy.
    #[must_use]
    pub fn string_descriptor_mut(&mut self) -> &mut StringDescriptor {
        &mut self.string_descriptor
    }

    /// Get the power configuration.
    ///
    /// The power configuration contains:
    /// - Power source information
    /// - Remote wakeup support
    /// - Maximum power rating
    #[must_use]
    pub fn power_config(&self) -> &PowerConfig {
        &self.power_config
    }

    /// Get a mutable reference to the power configuration proxy.
    #[must_use]
    pub fn power_config_mut(&mut self) -> &mut PowerConfig {
        &mut self.power_config
    }

    /// Get the pin drive strengths.
    ///
    /// The pin drive strengths contain:
    /// - GPIO drive strengths
    /// - MSIO (FIFO) clock/data drive strengths
    #[must_use]
    pub fn pin_drive_strengths(&self) -> &PinDriveStrengths {
        &self.pin_drive_strength
    }

    /// Get a mutable reference to the pin drive strengths configuration.
    #[must_use]
    pub fn pin_drive_strengths_mut(&mut self) -> &mut PinDriveStrengths {
        &mut self.pin_drive_strength
    }

    /// Interrupt latency. Values may be `1` through `16` (inclusive).
    ///
    /// The latency is translated as `2**(latency-1)` USB frames, and
    /// one USB frame is `125us`.
    #[must_use]
    pub fn interrupt_latency(&self) -> u8 {
        self.interrupt_latency
    }

    /// Set the interrupt latency. Values may be `1` through `16` (inclusive).
    ///
    /// The latency is translated as `2**(latency-1)` USB frames, and
    /// one USB frame is `125us`.
    ///
    /// # Panics
    ///
    /// Panics if `latency` is not in the range `1..=16`.
    pub fn set_interrupt_latency(&mut self, latency: u8) {
        assert!((1..17).contains(&latency));
        self.interrupt_latency = latency;
    }

    /// Get the data transfer configuration.
    ///
    /// The data transfer configuration contains:
    /// - FIFO clock speed
    /// - FIFO mode
    /// - Channel configuration
    #[must_use]
    pub fn data_transfer(&self) -> &DataTransferConfig {
        &self.data_transfer
    }

    /// Get a mutable reference to the data transfer configuration.
    #[must_use]
    pub fn data_transfer_mut(&mut self) -> &mut DataTransferConfig {
        &mut self.data_transfer
    }

    /// Get the optional features.
    ///
    /// The optional features contain:
    /// - Battery charging
    /// - Notification
    /// - Underrun detection
    #[must_use]
    pub fn optional_features(&self) -> &OptionalFeatures {
        &self.optional_features
    }

    /// Get a mutable reference to the optional features.
    #[must_use]
    pub fn optional_features_mut(&mut self) -> &mut OptionalFeatures {
        &mut self.optional_features
    }
}
