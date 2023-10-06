use std::ffi::{c_uint, c_ulong};

use crate::{
    ffi::{self, with_global_lock},
    try_d3xx, Device, Result,
};

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
    pub fn open(&self) -> Result<Device> {
        Device::open(&self.serial_number)
    }

    pub fn is_open(&self) -> bool {
        self.flags & ffi::_FT_FLAGS_FT_FLAGS_OPENED as u32 != 0
    }

    pub fn is_hispeed(&self) -> bool {
        self.flags & ffi::_FT_FLAGS_FT_FLAGS_HISPEED as u32 != 0
    }

    pub fn is_superspeed(&self) -> bool {
        self.flags & ffi::_FT_FLAGS_FT_FLAGS_SUPERSPEED as u32 != 0
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn device_type(&self) -> u32 {
        self.device_type
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn location_id(&self) -> u32 {
        self.location_id
    }

    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    pub fn description(&self) -> &str {
        &self.description
    }

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

fn create_device_info_list() -> Result<usize> {
    let mut num_devices: c_ulong = 0;
    try_d3xx!(unsafe { ffi::FT_CreateDeviceInfoList(&mut num_devices) })?;
    Ok(num_devices as usize)
}
