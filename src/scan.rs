use std::ffi::{c_uint, c_ulong};

use crate::{
    ffi::{self, with_global_lock},
    try_d3xx, Device, Result,
};

/// Information about a connected FT60x device.
///
/// This structure is returned by [`list_devices`].
pub struct DeviceInfo {
    flags: u32,
    device_type: u32,
    id: u32,
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
    pub fn is_open(&self) -> bool {
        self.flags & ffi::_FT_FLAGS_FT_FLAGS_OPENED as u32 != 0
    }

    /// Check if the device is a high-speed device.
    pub fn is_hispeed(&self) -> bool {
        self.flags & ffi::_FT_FLAGS_FT_FLAGS_HISPEED as u32 != 0
    }

    /// Check if the device is a superspeed device.
    pub fn is_superspeed(&self) -> bool {
        self.flags & ffi::_FT_FLAGS_FT_FLAGS_SUPERSPEED as u32 != 0
    }

    /// Get the device's flags.
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Get the device's type.
    pub fn device_type(&self) -> u32 {
        self.device_type
    }

    /// Get the device's vendor ID.
    pub fn vid(&self) -> u16 {
        (self.id >> 16) as u16
    }

    /// Get the device's product ID.
    pub fn pid(&self) -> u16 {
        (self.id & 0xFFFF) as u16
    }


    /// Get the device's ID (VID and PID combined)
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the device's location ID.
    pub fn location_id(&self) -> u32 {
        self.location_id
    }

    /// Get the device's serial number.
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Get the device's description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the device's handle.
    ///
    /// This is probably not useful to you.
    pub fn handle(&self) -> ffi::FT_HANDLE {
        self.handle
    }
}

impl From<ffi::FT_DEVICE_LIST_INFO_NODE> for DeviceInfo {
    fn from(info: ffi::FT_DEVICE_LIST_INFO_NODE) -> Self {
        // SAFETY: the strings are guaranteed to be non-null and null-terminated
        let serial_number = unsafe { std::ffi::CStr::from_ptr(info.SerialNumber.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let description = unsafe { std::ffi::CStr::from_ptr(info.Description.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        Self {
            flags: info.Flags,
            device_type: info.Type,
            id: info.ID,
            location_id: info.LocId,
            serial_number,
            description,
            handle: info.ftHandle,
        }
    }
}


/// List all connected FT60x devices.
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
                &mut figuratively_garbage as *mut c_uint,
            )
        })?;
        // SAFETY: the number of devices is known to be correct
        // and the device buffer is fully populated.
        unsafe { devices.set_len(n_devices) };

        Ok(devices)
    })?;

    Ok(devices
        .into_iter()
        .map(|info| DeviceInfo::from(info))
        .collect())
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
