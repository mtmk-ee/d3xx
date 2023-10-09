use std::{
    ffi::{c_uchar, OsString},
    os::windows::prelude::OsStringExt,
    ptr::addr_of_mut,
};

use crate::{ffi, try_d3xx, Result};

/// A USB device descriptor.
pub struct DeviceDescriptor {
    inner: ffi::FT_DEVICE_DESCRIPTOR,
    serial_number: String,
    manufacturer: String,
    product: String,
}

impl DeviceDescriptor {
    pub(crate) fn new(handle: ffi::FT_HANDLE) -> Result<Self> {
        let mut inner = ffi::FT_DEVICE_DESCRIPTOR::default();
        try_d3xx!(unsafe { ffi::FT_GetDeviceDescriptor(handle, addr_of_mut!(inner)) })?;
        // See pg. 5: https://ftdichip.com/wp-content/uploads/2020/08/TN_113_Simplified-Description-of-USB-Device-Enumeration.pdf
        debug_assert_eq!(inner.bLength, 18);
        debug_assert_eq!(inner.bDescriptorType, 1);
        Ok(Self {
            inner,
            serial_number: descriptor_string(handle, inner.iSerialNumber)?,
            manufacturer: descriptor_string(handle, inner.iManufacturer)?,
            product: descriptor_string(handle, inner.iProduct)?,
        })
    }

    /// The device serial number.
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Human-readable manufacturer name.
    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    /// Human-readable product name.
    pub fn product(&self) -> &str {
        &self.product
    }

    /// The vendor ID.
    pub fn vendor_id(&self) -> usize {
        usize::from(self.inner.idVendor)
    }

    /// The product ID.
    pub fn product_id(&self) -> usize {
        usize::from(self.inner.idProduct)
    }

    /// The USB protocol version (e.g. USB 2.0)
    pub fn usb_version(&self) -> UsbVersion {
        UsbVersion(usize::from(self.inner.bcdUSB))
    }

    /// The maximum size, in bytes, of a packet for an endpoint.
    pub fn max_packet_size(&self) -> usize {
        usize::from(self.inner.bMaxPacketSize0)
    }

    /// Device class codes
    pub fn class_codes(&self) -> ClassCodes {
        ClassCodes::new(
            self.inner.bDeviceClass,
            self.inner.bDeviceSubClass,
            self.inner.bDeviceProtocol,
        )
    }
}

/// A USB interface descriptor for a [`Device`](crate::Device).
pub struct InterfaceDescriptor {
    inner: ffi::FT_INTERFACE_DESCRIPTOR,
    description: String,
}

impl InterfaceDescriptor {
    pub fn new(handle: ffi::FT_HANDLE, index: c_uchar) -> Result<Self> {
        let mut inner = ffi::FT_INTERFACE_DESCRIPTOR::default();
        try_d3xx!(unsafe { ffi::FT_GetInterfaceDescriptor(handle, index, addr_of_mut!(inner)) })?;
        // See pg. 8: https://ftdichip.com/wp-content/uploads/2020/08/TN_113_Simplified-Description-of-USB-Device-Enumeration.pdf
        debug_assert_eq!(inner.bLength, 9);
        debug_assert_eq!(inner.bDescriptorType, 4);
        debug_assert_eq!(inner.bInterfaceNumber, index);
        Ok(Self {
            inner,
            description: descriptor_string(handle, inner.iInterface)?,
        })
    }

    pub fn interface_number(&self) -> usize {
        usize::from(self.inner.bInterfaceNumber)
    }

    /// Interface class codes.
    pub fn class_codes(&self) -> ClassCodes {
        ClassCodes::new(
            self.inner.bInterfaceClass,
            self.inner.bInterfaceSubClass,
            self.inner.bInterfaceProtocol,
        )
    }

    /// The number of endpoints used by this interface.
    pub fn endpoints(&self) -> usize {
        usize::from(self.inner.bNumEndpoints)
    }

    /// The interface number.
    pub fn alternate_setting(&self) -> u8 {
        self.inner.bAlternateSetting
    }

    /// A human-readable description of the interface.
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// A USB configuration descriptor for a [`Device`](crate::Device)
///
/// # Resources
/// - <https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__configuration__descriptor.html>
/// - Page 7 of <https://ftdichip.com/wp-content/uploads/2020/08/TN_113_Simplified-Description-of-USB-Device-Enumeration.pdf>
pub struct ConfigurationDescriptor {
    inner: ffi::FT_CONFIGURATION_DESCRIPTOR,
    description: String,
}

impl ConfigurationDescriptor {
    pub fn new(handle: ffi::FT_HANDLE) -> Result<Self> {
        let mut inner = ffi::FT_CONFIGURATION_DESCRIPTOR::default();
        try_d3xx!(unsafe { ffi::FT_GetConfigurationDescriptor(handle, addr_of_mut!(inner)) })?;
        // See pg. 7: https://ftdichip.com/wp-content/uploads/2020/08/TN_113_Simplified-Description-of-USB-Device-Enumeration.pdf
        debug_assert_eq!(inner.bLength, 9);
        debug_assert_eq!(inner.bDescriptorType, 2);
        Ok(Self {
            inner,
            description: descriptor_string(handle, inner.iConfiguration)?,
        })
    }

    /// The number of interfaces supported in this configuration.
    pub fn interfaces(&self) -> usize {
        usize::from(self.inner.bNumInterfaces)
    }

    /// The configuration number.
    pub fn configuration_value(&self) -> u8 {
        self.inner.bConfigurationValue
    }

    /// The configuration description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The maximum power consumption of the device in milliamps.
    pub fn max_power(&self) -> u8 {
        // the value is in 2mA units
        self.inner.MaxPower * 2
    }

    /// Whether the device is self-powered.
    pub fn self_powered(&self) -> bool {
        self.inner.bmAttributes & 0b0100_0000 != 0
    }

    /// Whether the device supports remote wakeup.
    pub fn remote_wakeup(&self) -> bool {
        self.inner.bmAttributes & 0b0010_0000 != 0
    }
}

pub struct UsbVersion(usize);

impl UsbVersion {
    pub fn major(&self) -> usize {
        self.0 >> 8
    }

    pub fn minor(&self) -> usize {
        self.0 & 0xFF
    }
}

/// Class code triple for a device or interface descriptor.
///
/// Contains the class, subclass, and protocol codes.
pub struct ClassCodes {
    class: u8,
    subclass: u8,
    protocol: u8,
}

impl ClassCodes {
    fn new(class: u8, subclass: u8, protocol: u8) -> Self {
        Self {
            class,
            subclass,
            protocol,
        }
    }

    /// Class code (assigned by USB-IF)
    pub fn class(&self) -> u8 {
        self.class
    }

    /// Subclass code (assigned by USB-IF)
    pub fn subclass(&self) -> u8 {
        self.subclass
    }

    /// Protocol code (assigned by USB-IF)
    pub fn protocol(&self) -> u8 {
        self.protocol
    }
}

/// Fetch a string descriptor from the device.
fn descriptor_string(handle: ffi::FT_HANDLE, index: c_uchar) -> Result<String> {
    let mut descriptor = ffi::FT_STRING_DESCRIPTOR::default();
    try_d3xx!(unsafe { ffi::FT_GetStringDescriptor(handle, index, addr_of_mut!(descriptor)) })?;
    Ok(OsString::from_wide(&descriptor.szString)
        .to_string_lossy()
        .into_owned())
}
