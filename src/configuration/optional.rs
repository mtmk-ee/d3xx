use crate::PipeId;

pub struct OptionalFeatures {
    flags: u16,
    battery_charging: Option<BatteryChargingModes>,
}

impl OptionalFeatures {
    pub fn new(flags: u16, battery_flags: u8) -> Self {
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

    pub fn all_disabled(&self) -> bool {
        self.flags == 0
    }

    pub fn all_enabled(&self) -> bool {
        self.flags == 0xFFFF
    }

    pub fn battery_charging(&self) -> Option<&BatteryChargingModes> {
        self.battery_charging.as_ref()
    }

    pub fn cancel_session_on_underrun_disabled(&self) -> bool {
        self.flags & 0b0000_0010 != 0
    }

    pub fn notification_message_enabled(&self, in_pipe: PipeId) -> bool {
        assert!(in_pipe.is_in());
        self.flags & (0b0000_0100 << in_pipe as u16) != 0
    }

    pub fn underrun_disabled(&self, in_pipe: PipeId) -> bool {
        assert!(in_pipe.is_in());
        self.flags & (0b0100_0000 << in_pipe as u16) != 0
    }
}

pub struct BatteryChargingModes(u8);

impl BatteryChargingModes {
    pub fn dcp(&self) -> u8 {
        (self.0 & 0xC0) >> 6
    }

    pub fn cdp(&self) -> u8 {
        (self.0 & 0x30) >> 4
    }

    pub fn sdp(&self) -> u8 {
        (self.0 & 0x0C) >> 2
    }
}
