const FLAG_REMOTE_WAKEUP: u8 = 0x20;
const FLAG_SELF_POWERED: u8 = 0x40;

/// Power configuration contained in the configuration descriptor.
pub struct PowerConfig {
    flags: u8,
    max_power: u16,
}

impl PowerConfig {
    pub(crate) fn new(flags: u8, max_power: u16) -> Self {
        Self { flags, max_power }
    }

    /// Check if the device is bus-powered.
    #[must_use]
    pub fn bus_powered(&self) -> bool {
        !self.self_powered()
    }

    /// Check if the device is self-powered.
    #[must_use]
    pub fn self_powered(&self) -> bool {
        self.flags & FLAG_SELF_POWERED != 0
    }

    /// Check if the device supports remote wakeup.
    ///
    /// Remote wakeup is a feature of some USB devices that allows them to
    /// "wake up" while suspended (in power-saving mode) when an external event
    /// occurs. Examples of such devices include keyboards and mice.
    #[must_use]
    pub fn remote_wakeup(&self) -> bool {
        self.flags & FLAG_REMOTE_WAKEUP != 0
    }

    /// Get the maximum power consumption in milliamps.
    #[must_use]
    pub fn max_power(&self) -> u16 {
        self.max_power * 2 // 2mA units
    }
}
