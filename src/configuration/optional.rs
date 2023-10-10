use crate::PipeId;

/// Optional features.
pub struct OptionalFeatures {
    flags: u16,
    battery_charging: Option<BatteryChargingModes>,
}

impl OptionalFeatures {
    pub(crate) fn new(flags: u16, battery_flags: u8) -> Self {
        let battery_charging = if flags & 1 == 0 {
            None
        } else {
            Some(BatteryChargingModes(battery_flags))
        };
        Self {
            flags,
            battery_charging,
        }
    }

    /// Check if all optional features are disabled.
    #[must_use]
    pub fn all_disabled(&self) -> bool {
        self.flags == 0
    }

    /// Check if all optional features are enabled.
    #[must_use]
    pub fn all_enabled(&self) -> bool {
        self.flags == 0xFFFF
    }

    /// Get the battery charging configuration.
    ///
    /// If the device does not support battery charging, this will return `None`.
    #[must_use]
    pub fn battery_charging(&self) -> Option<&BatteryChargingModes> {
        self.battery_charging.as_ref()
    }

    #[must_use]
    pub fn cancel_session_on_underrun_disabled(&self) -> bool {
        self.flags & 0b0000_0010 != 0
    }

    #[must_use]
    pub fn notification_message_enabled(&self, in_pipe: PipeId) -> bool {
        assert!(in_pipe.is_in());
        self.flags & (0b0000_0100 << in_pipe as u16) != 0
    }

    #[must_use]
    pub fn underrun_disabled(&self, in_pipe: PipeId) -> bool {
        assert!(in_pipe.is_in());
        self.flags & (0b0100_0000 << in_pipe as u16) != 0
    }
}

/// Battery charging configuration.
pub struct BatteryChargingModes(u8);

impl BatteryChargingModes {
    #[must_use]
    pub fn dcp(&self) -> u8 {
        (self.0 & 0xC0) >> 6
    }

    #[must_use]
    pub fn cdp(&self) -> u8 {
        (self.0 & 0x30) >> 4
    }

    #[must_use]
    pub fn sdp(&self) -> u8 {
        (self.0 & 0x0C) >> 2
    }
}
