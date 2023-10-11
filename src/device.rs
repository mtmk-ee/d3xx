use std::{
    ffi::{c_void, CString},
    fmt::Debug,
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::addr_of_mut,
};

use crate::{
    descriptor::{ConfigurationDescriptor, DeviceDescriptor, InterfaceDescriptor},
    ffi,
    gpio::{Gpio, GpioPin},
    notification::{clear_notification_callback, set_notification_callback, Notification},
    try_d3xx, Pipe, PipeId, Result, Version,
};

type PhantomUnsync = PhantomData<std::cell::Cell<()>>;

/// Handle to a D3XX device.
///
/// The handle is the primary interface for interacting with a `FT60x` device.
/// It provides methods for reading, writing, configuration, and more.
///
/// # Example
///
/// ```no_run
/// use std::io::{Read, Write};
/// use d3xx::{Device, Pipe, PipeId};
///
/// let device = Device::open("ABC123").unwrap();
///
/// // Read 1024 bytes from input pipe 1
/// let mut buf = vec![0u8; 1024];
/// device
///     .pipe(PipeId::In1)
///     .read(&mut buf)
///     .unwrap();
///
/// // Write 1024 bytes to output pipe 1
/// device
///     .pipe(PipeId::Out1)
///     .write(&buf)
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct Device {
    /// Handle returned by the D3XX driver when the device is opened.
    handle: ffi::FT_HANDLE,
    // Cannot share handle across threads since the driver is not thread-safe,
    // and so we need to prevent race conditions on device operations.
    _unsync: PhantomUnsync,
}

impl Device {
    /// Open a device by serial number.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use d3xx::Device;
    ///
    /// let device = Device::open("ABC123").unwrap();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `serial_number` contains a null byte.
    pub fn open(serial_number: &str) -> Result<Self> {
        let serial_cstr = CString::new(serial_number).expect("failed to create CString");
        let mut handle: ffi::FT_HANDLE = std::ptr::null_mut();
        try_d3xx!(unsafe {
            ffi::FT_Create(
                serial_cstr.as_ptr() as *mut c_void,
                ffi::FT_OPEN_BY_SERIAL_NUMBER,
                &mut handle,
            )
        })?;
        if handle.is_null() {
            Err(crate::D3xxError::DeviceNotFound)
        } else {
            Ok(Self {
                handle,
                _unsync: PhantomData,
            })
        }
    }

    /// Open a device using the given handle.
    ///
    /// # Safety
    /// This method is unsafe because the handle must be valid and care should
    /// not be used elsewhere.
    pub unsafe fn with_handle(handle: ffi::FT_HANDLE) -> Self {
        Self {
            handle,
            _unsync: PhantomData,
        }
    }

    /// Get the device's handle.
    ///
    /// This handle is fairly useless on its own. Although not recommended for typical
    /// users, it may be used with the raw D3XX bindings in the [ffi] module.
    #[must_use]
    pub fn handle(&self) -> ffi::FT_HANDLE {
        self.handle
    }

    /// Get the device descriptor.
    pub fn device_descriptor(&self) -> Result<DeviceDescriptor> {
        DeviceDescriptor::new(self.handle)
    }

    /// Get the configuration descriptor.
    pub fn configuration_descriptor(&self) -> Result<ConfigurationDescriptor> {
        ConfigurationDescriptor::new(self.handle)
    }

    /// Get the interface descriptor for the given interface.
    pub fn interface_descriptor(&self, interface: u8) -> Result<InterfaceDescriptor> {
        InterfaceDescriptor::new(self.handle, interface)
    }

    /// Get the chip configuration.
    #[cfg(feature = "config")]
    pub fn chip_configuration(&self) -> Result<crate::ChipConfiguration> {
        crate::ChipConfiguration::new(self.handle)
    }

    /// Returns a [`Pipe`] for pipe I/O and configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::io::Write;
    /// use d3xx::{Device, Pipe, PipeId};
    ///
    /// let device = Device::open("ABC123").unwrap();
    ///
    /// // Write to output pipe 1
    /// let mut buf = vec![0u8; 1024];
    /// device
    ///     .pipe(PipeId::Out1)
    ///     .write(&buf)
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn pipe(&self, id: PipeId) -> Pipe {
        Pipe::new(self, id)
    }

    /// Returns a [`Gpio`] for GPIO pin I/O and configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use d3xx::{Device, GpioPin};
    ///
    /// let device = Device::open("ABC123").unwrap();
    ///
    /// // Write to GPIO pin 0
    /// device
    ///    .gpio(GpioPin::Pin0)
    ///    .write(d3xx::Level::High)
    ///    .unwrap()
    /// ```
    #[must_use]
    pub fn gpio(&self, pin: GpioPin) -> Gpio {
        Gpio::new(self, pin)
    }

    /// Get the D3XX driver version.
    pub fn driver_version(&self) -> Result<Version> {
        let mut version: u32 = 0;
        try_d3xx!(unsafe { ffi::FT_GetDriverVersion(self.handle, addr_of_mut!(version)) })?;
        Ok(Version(version))
    }

    /// Power cycle the device port, causing the device to be re-enumerated by the host.
    pub fn power_cycle_port(self) -> Result<()> {
        // No need to run the destructor since the device will be closed when
        // the port is cycled.
        let device = ManuallyDrop::new(self);
        try_d3xx!(unsafe { ffi::FT_CycleDevicePort(device.handle) })?;
        Ok(())
    }

    /// Get the USB selective suspend timeout in milliseconds.
    pub fn suspend_timeout(&self) -> Result<u32> {
        let mut timeout: u32 = 0;
        try_d3xx!(unsafe { ffi::FT_GetSuspendTimeout(self.handle, addr_of_mut!(timeout)) })?;
        Ok(timeout)
    }

    /// Set the USB selective suspend timeout.
    ///
    /// The timeout is reset to the default of 10 seconds each time
    /// the device is opened.
    pub fn set_suspend_timeout(&self, timeout: Option<u32>) -> Result<()> {
        let timeout = timeout.unwrap_or(0);
        try_d3xx!(unsafe { ffi::FT_SetSuspendTimeout(self.handle, timeout) })?;
        Ok(())
    }

    /// Set the notification callback.
    ///
    /// The callback is invoked by the driver once a notification is received about
    /// data availability on a notification-enabled pipe. Pipes should not be read
    /// outside of the callback when notifications are enabled.
    ///
    /// See page 42 for more information:
    /// <https://ftdichip.com/wp-content/uploads/2020/07/AN_379-D3xx-Programmers-Guide-1.pdf>
    pub fn set_notification_callback<F, T>(&self, callback: F, context: Option<T>) -> Result<()>
    where
        F: Fn(Notification<T>) + 'static,
    {
        set_notification_callback(self.handle, callback, context)
    }

    /// Clear a previously-set notification callback.
    pub fn clear_notification_callback(&self) {
        // SAFETY: the handle exists
        unsafe {
            clear_notification_callback(self.handle);
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            ffi::FT_Close(self.handle);
        }
    }
}
