//! USB descriptor types.
//!
//! Descriptor types are used to describe the capabilities of a USB device.
//! The D3XX API provides access to the following descriptors:
//!
//! 1. A [device descriptor](crate::descriptor::DeviceDescriptor)
//! 2. A [configuration descriptor](crate::descriptor::ConfigurationDescriptor)
//! 3. One or more [interface descriptors](crate::descriptor::InterfaceDescriptor)
//! 4. One or more [pipe (endpoint) descriptors](crate::descriptor::PipeInfo)
//!
//! Descriptors (1), (2), and (3) are returned by [`Device::device_descriptor`](crate::Device),
//! [`Device::configuration_descriptor`](crate::Device), and [`Device::interface_descriptor`](crate::Device),
//! respectively. Descriptor (4) maybe obtained using [`Pipe::descriptor`](crate::Pipe) on a pipe
//! obtained from a [`Device`](crate::Device).
//!
//! Although USB devices may provide more types of descriptors, they are not supported by the D3XX API.
//!
//! # Further Reading
//! - <https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__descriptors.html>
//! - <https://ftdichip.com/wp-content/uploads/2020/08/TN_113_Simplified-Description-of-USB-Device-Enumeration.pdf>

use std::ptr::addr_of_mut;

use crate::{ffi, try_d3xx, D3xxError, Pipe, PipeType, Result};

/// A USB device descriptor.
///
/// There is one device descriptor provided by a D3XX device.
/// This descriptor holds very basic information about the device, such as
/// its identification, USB version, and function.
pub struct DeviceDescriptor {
    /// The inner descriptor struct.
    ///
    /// Contains the raw data returned by the driver. Additional information
    /// is provided by the other fields of this struct.
    inner: ffi::FT_DEVICE_DESCRIPTOR,
    serial_number: String,
    manufacturer: String,
    product: String,
}

impl DeviceDescriptor {
    /// Build a new `DeviceDescriptor` instance by reading the device.
    ///
    /// The descriptor and corresponding descriptor strings are pulled from
    /// the device. This operation will fail if the handle is not valid.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the descriptor returned by the driver is invalid.
    /// This is intended for debugging purposes, and the behavior is likely to change
    /// in the future.
    pub(crate) fn new(handle: ffi::FT_HANDLE) -> Result<Self> {
        let mut inner = ffi::FT_DEVICE_DESCRIPTOR::default();
        try_d3xx!(unsafe { ffi::FT_GetDeviceDescriptor(handle, addr_of_mut!(inner)) })?;
        // The device descriptor has a particular format, so we can perform a sanity check here
        // to avoid reading from potentially invalid memory.
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
    #[must_use]
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Human-readable manufacturer name.
    #[must_use]
    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    /// Human-readable product name.
    #[must_use]
    pub fn product(&self) -> &str {
        &self.product
    }

    /// The vendor ID.
    #[must_use]
    pub fn vendor_id(&self) -> usize {
        usize::from(self.inner.idVendor)
    }

    /// The product ID.
    #[must_use]
    pub fn product_id(&self) -> usize {
        usize::from(self.inner.idProduct)
    }

    /// The USB protocol version (e.g. USB 2.0)
    #[must_use]
    pub fn usb_version(&self) -> UsbVersion {
        UsbVersion(usize::from(self.inner.bcdUSB))
    }

    /// The maximum size, in bytes, of a packet for an endpoint.
    ///
    /// This is typically irrelevant for the user.
    #[must_use]
    pub fn max_packet_size(&self) -> usize {
        usize::from(self.inner.bMaxPacketSize0)
    }

    /// Returns a struct containing the interface class codes.
    ///
    /// The codes are used to indicate the class, subclass, and protocol.
    #[must_use]
    pub fn class_codes(&self) -> ClassCodes {
        ClassCodes::new(
            self.inner.bDeviceClass,
            self.inner.bDeviceSubClass,
            self.inner.bDeviceProtocol,
        )
    }
}

/// A USB interface descriptor for a [`Device`](crate::Device).
///
/// There is one interface descriptor per interface. This descriptor holds
/// information about the interface, such as its class codes, and information
/// about the endpoints used by the interface.
pub struct InterfaceDescriptor {
    /// The inner descriptor struct.
    ///
    /// Contains the raw data returned by the driver. Additional information
    /// is provided by the other fields of this struct.
    inner: ffi::FT_INTERFACE_DESCRIPTOR,
    description: String,
}

impl InterfaceDescriptor {
    /// Build a new `InterfaceDescriptor` instance by reading the device.
    ///
    /// The descriptor and corresponding descriptor strings are pulled from
    /// the device. This operation will fail if the handle is not valid.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the descriptor returned by the driver is invalid.
    /// This is intended for debugging purposes, and the behavior is likely to change
    /// in the future.
    pub(crate) fn new(handle: ffi::FT_HANDLE, index: u8) -> Result<Self> {
        let mut inner = ffi::FT_INTERFACE_DESCRIPTOR::default();
        try_d3xx!(unsafe { ffi::FT_GetInterfaceDescriptor(handle, index, addr_of_mut!(inner)) })?;
        // The device descriptor has a particular format, so we can perform a sanity check here
        // to avoid reading from potentially invalid memory.
        //
        // See pg. 8: https://ftdichip.com/wp-content/uploads/2020/08/TN_113_Simplified-Description-of-USB-Device-Enumeration.pdf
        debug_assert_eq!(inner.bLength, 9);
        debug_assert_eq!(inner.bDescriptorType, 4);
        debug_assert_eq!(inner.bInterfaceNumber, index);
        Ok(Self {
            inner,
            description: descriptor_string(handle, inner.iInterface)?,
        })
    }

    /// The interface this descriptor describes.
    ///
    /// The interface number is unique per configuration, but may be
    /// reused across configurations.
    #[must_use]
    pub fn interface_number(&self) -> usize {
        usize::from(self.inner.bInterfaceNumber)
    }

    /// Returns a struct containing the interface class codes.
    ///
    /// The codes are used to indicate the class, subclass, and protocol.
    #[must_use]
    pub fn class_codes(&self) -> ClassCodes {
        ClassCodes::new(
            self.inner.bInterfaceClass,
            self.inner.bInterfaceSubClass,
            self.inner.bInterfaceProtocol,
        )
    }

    /// The number of endpoints used by this interface.
    #[must_use]
    pub fn endpoints(&self) -> usize {
        usize::from(self.inner.bNumEndpoints)
    }

    /// A value used to select an alternate setting for this interface.
    ///
    /// The D3XX API does not provide a way to switch to the alternate setting.
    #[must_use]
    pub fn alternate_setting(&self) -> u8 {
        self.inner.bAlternateSetting
    }

    /// A human-readable description of the interface.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// A USB configuration descriptor for a [`Device`](crate::Device)
///
/// There is one configuration descriptor per configuration. This descriptor holds
/// information about the configuration, such as its description, power settings,
/// and its interfaces.
///
/// # Resources
/// - <https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__configuration__descriptor.html>
/// - Page 7 of <https://ftdichip.com/wp-content/uploads/2020/08/TN_113_Simplified-Description-of-USB-Device-Enumeration.pdf>
pub struct ConfigurationDescriptor {
    /// The inner descriptor struct.
    ///
    /// Contains the raw data returned by the driver. Additional information
    /// is provided by the other fields of this struct.
    inner: ffi::FT_CONFIGURATION_DESCRIPTOR,
    description: String,
}

impl ConfigurationDescriptor {
    /// Build a new `ConfigurationDescriptor` instance by reading the device.
    ///
    /// The descriptor and corresponding descriptor strings are pulled from
    /// the device. This operation will fail if the handle is not valid.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the descriptor returned by the driver is invalid.
    /// This is intended for debugging purposes, and the behavior is likely to change
    /// in the future.
    pub(crate) fn new(handle: ffi::FT_HANDLE) -> Result<Self> {
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
    #[must_use]
    pub fn interfaces(&self) -> usize {
        usize::from(self.inner.bNumInterfaces)
    }

    /// The configuration number.
    ///
    /// The D3XX API does not provide a way to switch to a different configuration.
    #[must_use]
    pub fn configuration_value(&self) -> u8 {
        self.inner.bConfigurationValue
    }

    /// A human-readable description of the configuration.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The maximum power consumption of the device in milliamps.
    #[must_use]
    pub fn max_power(&self) -> u8 {
        // the value is in 2mA units
        self.inner.MaxPower * 2
    }

    /// Whether the device is self-powered.
    #[must_use]
    pub fn self_powered(&self) -> bool {
        self.inner.bmAttributes & CONFIGURATION_ATTRIBUTE_SELF_POWERED != 0
    }

    /// Whether the device supports remote wakeup.
    #[must_use]
    pub fn remote_wakeup(&self) -> bool {
        self.inner.bmAttributes & CONFIGURATION_ATTRIBUTE_REMOTE_WAKEUP != 0
    }
}

// Bit flags for the `bmAttributes` field of a configuration descriptor.
const CONFIGURATION_ATTRIBUTE_SELF_POWERED: u8 = 0b0100_0000;
const CONFIGURATION_ATTRIBUTE_REMOTE_WAKEUP: u8 = 0b0010_0000;

/// Indicates the USB protocol version (e.g. USB 3.1)
pub struct UsbVersion(usize);

impl UsbVersion {
    /// Major version number.
    #[must_use]
    pub fn major(&self) -> usize {
        self.0 >> 8
    }

    /// Minor version number.
    #[must_use]
    pub fn minor(&self) -> usize {
        self.0 & 0xFF
    }
}

/// Information about a pipe.
///
/// Note that this information is very similar to what the USB standard refers to as
/// an "endpoint descriptor". However, the D3XX API provides a slightly different structure
/// containing a subset of this data.
///
/// This is returned by [`PipeIo::descriptor`](crate::PipeIo).
///
/// See for more information:
/// <https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__endpoint__descriptor.html>
pub struct PipeInfo {
    pipe: Pipe,
    pipe_type: PipeType,
    max_packet_size: usize,
    interval: u8,
}

impl PipeInfo {
    /// Create a new `PipeInfo` instance from the given [`ffi::FT_PIPE_INFORMATION`] struct.
    ///
    /// Although unlikely if `info` has been obtained directly from the driver, this may fail
    /// if the pipe type or ID is invalid.
    pub(crate) fn new(info: ffi::FT_PIPE_INFORMATION) -> Result<Self> {
        Ok(Self {
            pipe_type: PipeType::from(info.PipeType),
            pipe: Pipe::try_from(info.PipeId).or(Err(D3xxError::OtherError))?,
            max_packet_size: info.MaximumPacketSize as usize,
            interval: info.Interval,
        })
    }

    /// The type of transfer used for the pipe.
    #[must_use]
    pub fn pipe_type(&self) -> PipeType {
        self.pipe_type
    }

    /// The pipe ID.
    #[must_use]
    pub fn id(&self) -> Pipe {
        self.pipe
    }

    /// The maximum packet size in bytes.
    ///
    /// This is typically irrelevant for the user.
    #[must_use]
    pub fn max_packet_size(&self) -> usize {
        self.max_packet_size
    }

    /// The polling interval for data transfers.
    ///
    /// What this value corresponds to depends on the `pipe_type`.
    /// See <https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__endpoint__descriptor.html>
    #[must_use]
    pub fn interval(&self) -> u8 {
        self.interval
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
    /// Create a new `ClassCodes` instance with the given codes.
    fn new(class: u8, subclass: u8, protocol: u8) -> Self {
        Self {
            class,
            subclass,
            protocol,
        }
    }

    /// Class code (assigned by USB-IF)
    #[must_use]
    pub fn class(&self) -> u8 {
        self.class
    }

    /// Subclass code (assigned by USB-IF)
    #[must_use]
    pub fn subclass(&self) -> u8 {
        self.subclass
    }

    /// Protocol code (assigned by USB-IF)
    #[must_use]
    pub fn protocol(&self) -> u8 {
        self.protocol
    }
}

/// Fetch a string descriptor from the device.
///
/// It is important that `index` is valid, as unknown behavior may occur from
/// attempting to read past the end of the descriptor table.
fn descriptor_string(handle: ffi::FT_HANDLE, index: u8) -> Result<String> {
    let mut descriptor = ffi::FT_STRING_DESCRIPTOR::default();
    try_d3xx!(unsafe { ffi::FT_GetStringDescriptor(handle, index, addr_of_mut!(descriptor)) })?;
    Ok(widestring::U16CStr::from_slice(&descriptor.szString)
        .or(Err(D3xxError::OtherError))?
        .to_string_lossy())
}

#[cfg(test)]
mod test {
    use crate::{descriptor::PipeInfo, ffi, Pipe, PipeType};

    #[test]
    fn pipe_info_try_from() {
        let info = ffi::FT_PIPE_INFORMATION {
            PipeType: ffi::FT_PIPE_TYPE::FTPipeTypeControl,
            PipeId: 0x82,
            MaximumPacketSize: 64,
            Interval: 0,
        };
        let info = PipeInfo::new(info).unwrap();
        assert_eq!(info.pipe_type(), PipeType::Control);
        assert_eq!(info.id(), Pipe::In0);
        assert_eq!(info.max_packet_size(), 64);
        assert_eq!(info.interval(), 0);
    }

    #[test]
    fn class_code() {
        let codes = super::ClassCodes::new(0x00, 0x00, 0x00);
        assert_eq!(codes.class(), 0x00);
        assert_eq!(codes.subclass(), 0x00);
        assert_eq!(codes.protocol(), 0x00);

        let codes = super::ClassCodes::new(0x01, 0x02, 0x03);
        assert_eq!(codes.class(), 0x01);
        assert_eq!(codes.subclass(), 0x02);
        assert_eq!(codes.protocol(), 0x03);
    }

    #[test]
    fn usb_version() {
        let version = super::UsbVersion(0x0200);
        assert_eq!(version.major(), 2);
        assert_eq!(version.minor(), 0);

        let version = super::UsbVersion(0x0210);
        assert_eq!(version.major(), 2);
        assert_eq!(version.minor(), 16);
    }
}
