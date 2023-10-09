use std::ffi::{c_uint, c_ulong};

use crate::{
    ffi::{self, with_global_lock},
    try_d3xx, Device, Result,
};

/// Information about a connected `FT60x` device.
///
/// This structure is returned by [`list_devices`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    flags: u32,
    device_type: DeviceType,
    vid: u16,
    pid: u16,
    location_id: u32,
    serial_number: String,
    description: String,
    handle: ffi::FT_HANDLE,
}

impl DeviceInfo {
    /// Attempt to open the device.
    ///
    /// This is a convenience method that calls `Device::open` with the device's serial number.
    pub fn open(&self) -> Result<Device> {
        Device::open(&self.serial_number)
    }

    /// Check if the device is open, either by this process or another.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.flags & ffi::FT_FLAGS::FT_FLAGS_OPENED as u32 != 0
    }

    /// Check if the device is a high-speed device.
    #[must_use]
    pub fn is_hispeed(&self) -> bool {
        self.flags & ffi::FT_FLAGS::FT_FLAGS_HISPEED as u32 != 0
    }

    /// Check if the device is a superspeed device.
    #[must_use]
    pub fn is_superspeed(&self) -> bool {
        self.flags & ffi::FT_FLAGS::FT_FLAGS_SUPERSPEED as u32 != 0
    }

    /// Get the device's flags.
    #[must_use]
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Get the device's type.
    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    /// Get the device's vendor ID.
    #[must_use]
    pub fn vid(&self) -> u16 {
        self.vid
    }

    /// Get the device's product ID.
    #[must_use]
    pub fn pid(&self) -> u16 {
        self.pid
    }

    /// Get the device's location ID.
    #[must_use]
    pub fn location_id(&self) -> u32 {
        self.location_id
    }

    /// Get the device's serial number.
    #[must_use]
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Get the device's description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the device's handle.
    ///
    /// This is probably not useful to you.
    #[must_use]
    pub fn handle(&self) -> ffi::FT_HANDLE {
        self.handle
    }
}

impl From<ffi::FT_DEVICE_LIST_INFO_NODE> for DeviceInfo {
    fn from(info: ffi::FT_DEVICE_LIST_INFO_NODE) -> Self {
        Self::from(&info)
    }
}

impl From<&ffi::FT_DEVICE_LIST_INFO_NODE> for DeviceInfo {
    fn from(info: &ffi::FT_DEVICE_LIST_INFO_NODE) -> Self {
        // SAFETY: the strings are guaranteed to be non-null and null-terminated
        let serial_number = unsafe { std::ffi::CStr::from_ptr(info.SerialNumber.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let description = unsafe { std::ffi::CStr::from_ptr(info.Description.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        Self {
            flags: info.Flags,
            device_type: DeviceType::from(info.Type),
            vid: (info.ID >> 16) as u16,
            pid: (info.ID & 0xffff) as u16,
            location_id: info.LocId,
            serial_number,
            description,
            handle: info.ftHandle,
        }
    }
}

/// Represents the type of `FT60x` device.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeviceType {
    Unknown,
    FT600,
    FT601,
}

impl From<u32> for DeviceType {
    fn from(value: u32) -> Self {
        match value {
            600 => Self::FT600,
            601 => Self::FT601,
            _ => Self::Unknown,
        }
    }
}

/// List all connected `FT60x` devices.
pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    // global lock needed to prevent concurrent access to the driver's internal device table
    let devices = with_global_lock(|| -> Result<_> {
        let n_devices = create_device_info_list()?;
        // output parameter is guaranteed to be exactly equal to `n_devices`
        let mut figuratively_garbage: c_uint = 0;
        let mut devices: Vec<ffi::FT_DEVICE_LIST_INFO_NODE> = Vec::with_capacity(n_devices);
        try_d3xx!(unsafe {
            ffi::FT_GetDeviceInfoList(
                devices.as_mut_ptr(),
                std::ptr::addr_of_mut!(figuratively_garbage),
            )
        })?;
        // SAFETY: the number of devices is known to be correct
        // and the device buffer is fully populated.
        unsafe { devices.set_len(n_devices) };

        Ok(devices)
    })?;

    Ok(devices.into_iter().map(DeviceInfo::from).collect())
}

/// Create a device info list and return the number of devices.
///
/// This must be done at least once before calling `FT_GetDeviceInfoList`.
///
/// Note: the underlying device table does not automatically update; it
/// must be refreshed when needed by calling this function again.
fn create_device_info_list() -> Result<usize> {
    let mut num_devices: c_ulong = 0;
    try_d3xx!(unsafe { ffi::FT_CreateDeviceInfoList(&mut num_devices) })?;
    Ok(num_devices as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_from() {
        assert_eq!(DeviceType::from(600), DeviceType::FT600);
        assert_eq!(DeviceType::from(601), DeviceType::FT601);
        assert_eq!(DeviceType::from(0), DeviceType::Unknown);
    }

    #[test]
    fn device_info_from() {
        fn array<const A: usize, const B: usize>(array: &[u8; A]) -> [i8; B] {
            assert!(A < B);
            let mut out = [0i8; B];
            array
                .iter()
                .enumerate()
                .for_each(|(i, &a)| out[i] = i8::try_from(a).unwrap());
            out
        }

        let serial = array(b"ABC123");
        let description = array(b"FT601");
        let info = ffi::FT_DEVICE_LIST_INFO_NODE {
            Flags: 1,
            Type: 600,
            ID: 0x0403_6010,
            LocId: 2,
            SerialNumber: serial,
            Description: description,
            ftHandle: std::ptr::null_mut(),
        };
        let info = DeviceInfo::from(info);
        assert_eq!(info.flags(), 1);
        assert_eq!(info.device_type(), DeviceType::FT600);
        assert_eq!(info.vid(), 0x0403);
        assert_eq!(info.pid(), 0x6010);
        assert_eq!(info.location_id(), 2);
        assert_eq!(info.serial_number(), "ABC123");
        assert_eq!(info.description(), "FT601");
        assert_eq!(info.handle(), std::ptr::null_mut());
    }

    #[test]
    fn device_info_flags() {
        let mut raw_info = ffi::FT_DEVICE_LIST_INFO_NODE {
            Flags: 0,
            Type: 0,
            ID: 0,
            LocId: 0,
            SerialNumber: [0; 16],
            Description: [0; 32],
            ftHandle: std::ptr::null_mut(),
        };
        let info = DeviceInfo::from(&raw_info);
        assert!(!info.is_open());
        assert!(!info.is_hispeed());
        assert!(!info.is_superspeed());

        raw_info.Flags = ffi::FT_FLAGS::FT_FLAGS_OPENED as u32;
        let info = DeviceInfo::from(&raw_info);
        assert!(info.is_open());
        assert!(!info.is_hispeed());
        assert!(!info.is_superspeed());

        raw_info.Flags = ffi::FT_FLAGS::FT_FLAGS_HISPEED as u32;
        let info = DeviceInfo::from(&raw_info);
        assert!(!info.is_open());
        assert!(info.is_hispeed());
        assert!(!info.is_superspeed());

        raw_info.Flags = ffi::FT_FLAGS::FT_FLAGS_SUPERSPEED as u32;
        let info = DeviceInfo::from(&raw_info);
        assert!(!info.is_open());
        assert!(!info.is_hispeed());
        assert!(info.is_superspeed());
    }
}
