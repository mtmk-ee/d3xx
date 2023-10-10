/// Power configuration.
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
        self.flags & 0x40 != 0
    }

    /// Check if the device supports remote wakeup.
    #[must_use]
    pub fn remote_wakeup(&self) -> bool {
        self.flags & 0x20 != 0
    }

    /// Get the maximum power consumption.
    #[must_use]
    pub fn max_power(&self) -> u16 {
        self.max_power
    }
}
