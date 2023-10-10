use std::ptr::addr_of_mut;

pub mod data_transfer;
pub mod notification;
pub mod optional;
pub mod pin_drive;
pub mod power;
mod string_descriptor;

use crate::{ffi, try_d3xx, Result};

use self::{
    data_transfer::DataTransferConfig, optional::OptionalFeatures, pin_drive::PinDriveStrengths,
    power::PowerConfig, string_descriptor::StringDescriptor,
};

/// `FT60x` chip configuration.
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
    pub fn vendor_id(&self) -> u16 {
        self.vid
    }

    /// Product ID.
    pub fn product_id(&self) -> u16 {
        self.pid
    }

    pub fn string_descriptor(&self) -> &StringDescriptor {
        &self.string_descriptor
    }

    pub fn string_descriptor_mut(&mut self) -> &mut StringDescriptor {
        &mut self.string_descriptor
    }

    pub fn power_config(&self) -> &PowerConfig {
        &self.power_config
    }

    pub fn power_config_mut(&mut self) -> &mut PowerConfig {
        &mut self.power_config
    }

    pub fn pin_drive_strengths(&self) -> &PinDriveStrengths {
        &self.pin_drive_strength
    }

    pub fn pin_drive_strengths_mut(&mut self) -> &mut PinDriveStrengths {
        &mut self.pin_drive_strength
    }

    /// Interrupt latency. Values may be `1` through `16` (inclusive).
    ///
    /// The latency is translated as `2**(latency-1)` USB frames, and
    /// one USB frame is `125us`.
    pub fn interrupt_latency(&self) -> u8 {
        self.interrupt_latency
    }

    pub fn set_interrupt_latency(&mut self, latency: u8) {
        assert!((1..17).contains(&latency));
        self.interrupt_latency = latency;
    }

    /// Clock speed of FIFO in `MHz`.
    pub fn data_transfer(&self) -> &DataTransferConfig {
        &self.data_transfer
    }

    pub fn data_transfer_mut(&mut self) -> &mut DataTransferConfig {
        &mut self.data_transfer
    }

    pub fn optional_features(&self) -> &OptionalFeatures {
        &self.optional_features
    }

    pub fn optional_features_mut(&mut self) -> &mut OptionalFeatures {
        &mut self.optional_features
    }
}
