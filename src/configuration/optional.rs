use crate::Pipe;

const FLAG_BATTERY_CHARGING_ENABLE: u16 = 0b0000_0001;
const FLAG_NOTIFICATION_ENABLE_PIPE0: u16 = 0b0000_0100;
const FLAG_UNDERRUN_DISABLE_PIPE0: u16 = 0b0100_0000;
const FLAG_UNDERRUN_DISABLE: u16 = 0b0000_0010;
const FLAG_ALL_ENABLED: u16 = 0xFFFF;

const FLAG_CHARGING_MODE_DCP: u8 = 0xC0;
const FLAG_CHARGING_MODE_CDP: u8 = 0x30;
const FLAG_CHARGING_MODE_SDP: u8 = 0x0C;
const OFFSET_CHARGING_MODE_DCP: usize = 6;
const OFFSET_CHARGING_MODE_CDP: usize = 4;
const OFFSET_CHARGING_MODE_SDP: usize = 2;

/// Optional features.
pub struct OptionalFeatures {
    flags: u16,
    battery_charging: Option<BatteryChargingModes>,
}

impl OptionalFeatures {
    pub(crate) fn new(flags: u16, battery_flags: u8) -> Self {
        Self {
            flags,
            battery_charging: match flags & FLAG_BATTERY_CHARGING_ENABLE {
                0 => None,
                _ => Some(BatteryChargingModes(battery_flags)),
            },
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
        self.flags == FLAG_ALL_ENABLED
    }

    /// Get the battery charging configuration.
    ///
    /// If the device does not support battery charging, this will return `None`.
    #[must_use]
    pub fn battery_charging(&self) -> Option<&BatteryChargingModes> {
        self.battery_charging.as_ref()
    }

    /// Check if notifications are enabled for the given pipe.
    ///
    /// # Panics
    ///
    /// Panics if `in_pipe` is not an input pipe.
    #[must_use]
    pub fn notification_message_enabled(&self, in_pipe: Pipe) -> bool {
        assert!(in_pipe.is_in());
        self.flags & (FLAG_NOTIFICATION_ENABLE_PIPE0 << in_pipe as u16) != 0
    }

    #[must_use]
    pub fn underrun_check_enabled(&self) -> bool {
        self.flags & FLAG_UNDERRUN_DISABLE == 0
    }

    /// Check if sessions are cancelled when an underrun occurs on the given pipe.
    ///
    /// When underrun condition checks are enabled the chip will cancel `IN`
    /// transfers when an underrun occurs from the FIFO master. This method
    /// returns `true` if underrun checks are disabled.
    ///
    /// Underrun conditions occur when the FIFO is provided less data from the
    /// FIFO master than the specified segment size.
    ///
    /// # Panics
    ///
    /// Panics if `in_pipe` is not an input pipe.
    #[must_use]
    pub fn underrun_disabled(&self, in_pipe: Pipe) -> bool {
        assert!(in_pipe.is_in());
        self.flags & (FLAG_UNDERRUN_DISABLE_PIPE0 << in_pipe as u16) != 0
    }
}

/// Battery charging mode.
///
/// This indicates the type of power source the device is connected to.
/// This is only available for configurations in whichs the GPIO pins are
/// properly configured.
///
/// # Further Reading
///
/// See <https://www.analog.com/en/technical-articles/the-basics-of-usb-battery-charging.html>
pub struct BatteryChargingModes(u8);

impl BatteryChargingModes {
    /// Dedicated charging port (DCP).
    ///
    /// This is for USB devices which only support charging, and do not support
    /// data transfer.
    #[must_use]
    pub fn dcp(&self) -> u8 {
        (self.0 & FLAG_CHARGING_MODE_DCP) >> OFFSET_CHARGING_MODE_DCP
    }

    /// Charging downstream port (CDP).
    ///
    /// This is for USB devices which charge above the normal current limits
    /// of USB 2.0 and USB 3.0 ports. This is typical for USB devices with
    /// rechargeable batteries.
    #[must_use]
    pub fn cdp(&self) -> u8 {
        (self.0 & FLAG_CHARGING_MODE_CDP) >> OFFSET_CHARGING_MODE_CDP
    }

    /// Standard downstream port (SDP).
    ///
    /// This is for USB device which do not support charging.
    #[must_use]
    pub fn sdp(&self) -> u8 {
        (self.0 & FLAG_CHARGING_MODE_SDP) >> OFFSET_CHARGING_MODE_SDP
    }
}
