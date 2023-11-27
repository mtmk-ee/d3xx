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
    try_d3xx,
    util::PhantomUnsync,
    Pipe, PipeIo, Result, Version,
};

/// This struct acts as a handle to a D3XX device, and the primary interface for all operations.
///
/// Once a device is opened with [`Device::open`], it is possible to perform operations such as
/// reading and writing to pipes, configuring GPIO pins, and more. While the device is open the
/// resource is owned by the `Device` struct, and will be closed when the struct is dropped.
/// It is not possible to open the same device multiple times simultaneously.
///
/// # Thread Safety
///
/// The `Device` struct offers unsynchronized interior mutability, meaning that it is not protected
/// from shared write operations, but can be moved between threads as long as it is not used
/// concurrently. In otherwords, `Device: Send + !Sync`. This is done out of an abundance of caution,
/// as FTDI provides no information regarding the thread-safety of the D3XX driver.
///
/// For multi-threaded applications, it is recommended to use a synchronization primitive such as a
/// [`Mutex`](std::sync::Mutex) to ensure that the device is not used concurrently. A
/// [`RwLock<Device>`](std::sync::RwLock) cannot be shared between threads because of the bounds on
/// its [`Sync`] implementation.
///
/// # Example
///
/// ```no_run
/// use std::io::{Read, Write};
/// use d3xx::{Device, Pipe};
///
/// let device = Device::open("ABC123").unwrap();
///
/// // Read 1024 bytes from input pipe 1
/// let mut buf = vec![0u8; 1024];
/// device
///     .pipe(Pipe::In1)
///     .read(&mut buf)
///     .unwrap();
///
/// // Write 1024 bytes to output pipe 1
/// device
///     .pipe(Pipe::Out1)
///     .write(&buf)
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct Device {
    /// Handle returned by the D3XX driver when the device is opened.
    handle: ffi::FT_HANDLE,
    /// Used to force `!Sync` since the driver may or may not be thread-safe.
    _unsync: PhantomUnsync,
}

impl Device {
    /// Open a device by serial number.
    ///
    /// The serial number is a unique identifier assigned to each device, and may be
    /// programmed by the user. UTF-8 is supported. If it is not known, it may be
    /// obtained by calling [`list_devices`](crate::list_devices) or another enumeration
    /// method.
    ///
    /// The serial number must be convertible to a [`CString`], and must not contain
    /// any internal null bytes.
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
    /// Panics if `serial_number` contains an internal null byte.
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
            // SAFETY: the handle is logically valid if the device was opened
            // successfully, and is not in use elsewhere.
            Ok(unsafe { Self::with_handle(handle) })
        }
    }

    /// Open a device using the given handle.
    ///
    /// # Safety
    ///
    /// The handle must be valid, already opened, and not in use elsewhere for the duration
    /// of the `Device` instance's lifetime.
    pub unsafe fn with_handle(handle: ffi::FT_HANDLE) -> Self {
        Self {
            handle,
            _unsync: PhantomData,
        }
    }

    /// Get the device's handle.
    ///
    /// The handle is fairly useless on its own. Although not recommended for typical
    /// users, it may be used with the raw D3XX bindings in the [ffi] module.
    #[must_use]
    pub fn handle(&self) -> ffi::FT_HANDLE {
        self.handle
    }

    /// Get the device's handle, consuming `self`.
    ///
    /// Note that the device is not closed by this method, and the handle must be closed
    /// manually using [`ffi::FT_Close`], or by re-wrapping it in a new `Device` instance
    /// using [`Device::with_handle`].
    ///
    /// The handle is fairly useless on its own. Although not recommended for typical
    /// users, it may be used with the raw D3XX bindings in the [ffi] module.
    #[must_use]
    pub fn into_handle(self) -> ffi::FT_HANDLE {
        let device = ManuallyDrop::new(self);
        device.handle
    }

    /// Get the USB device descriptor.
    ///
    /// The device descriptor contains information such as identifiers, device class,
    /// versions, and more.
    pub fn device_descriptor(&self) -> Result<DeviceDescriptor> {
        DeviceDescriptor::new(self.handle)
    }

    /// Get the configuration descriptor.
    ///
    /// The configuration descriptor contains information about the device's configuration,
    /// power requirements, and more.
    pub fn configuration_descriptor(&self) -> Result<ConfigurationDescriptor> {
        ConfigurationDescriptor::new(self.handle)
    }

    /// Get the interface descriptor for the given interface.
    ///
    /// The interface descriptor contains information about the interface class, endpoints,
    /// and more.
    ///
    /// The interface number must correspond to a valid interface under the current
    /// configuration.
    pub fn interface_descriptor(&self, interface: u8) -> Result<InterfaceDescriptor> {
        InterfaceDescriptor::new(self.handle, interface)
    }

    /// Get the chip configuration.
    ///
    /// The chip configuration is an FTDI-defined structure containing information about
    /// the chip and its configuration. Some of the information is also available through
    /// the various descriptors.
    #[cfg(feature = "config")]
    pub fn chip_configuration(&self) -> Result<crate::configuration::ChipConfiguration> {
        crate::configuration::ChipConfiguration::new(self.handle)
    }

    /// Returns a [`Pipe`] for pipe I/O and configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::io::Write;
    /// use d3xx::{Device, Pipe};
    ///
    /// let device = Device::open("ABC123").unwrap();
    ///
    /// // Write to output pipe 1
    /// let mut buf = vec![0u8; 1024];
    /// device
    ///     .pipe(Pipe::Out1)
    ///     .write(&buf)
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn pipe(&self, id: Pipe) -> PipeIo {
        PipeIo::new(self, id)
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
    ///
    /// This method consumes `self`, as the device is closed when the port is cycled.
    /// After a short delay the device will be re-enumerated and may be opened again.
    pub fn power_cycle_port(self) -> Result<()> {
        // No need to run the destructor since the device will be closed when
        // the port is cycled.
        let handle = self.into_handle();
        try_d3xx!(unsafe { ffi::FT_CycleDevicePort(handle) })?;
        Ok(())
    }

    /// Get the USB selective suspend timeout in milliseconds.
    ///
    /// Selective suspend is a power-saving feature that allows the host to power-down
    /// individual USB devices if no I/O requests have been made for a certain period
    /// of time. The device will be powered-up when an I/O request is made.
    #[cfg(windows)]
    pub fn suspend_timeout(&self) -> Result<u32> {
        let mut timeout: u32 = 0;
        try_d3xx!(unsafe { ffi::FT_GetSuspendTimeout(self.handle, addr_of_mut!(timeout)) })?;
        Ok(timeout)
    }

    /// Set the USB selective suspend timeout.
    ///
    /// The timeout is reset to the default of 10 seconds each time
    /// the device is opened.
    ///
    /// Selective suspend is a power-saving feature that allows the host to power-down
    /// individual USB devices if no I/O requests have been made for a certain period
    /// of time. The device will be powered-up when an I/O request is made.
    #[cfg(windows)]
    pub fn set_suspend_timeout(&self, timeout: Option<u32>) -> Result<()> {
        let timeout = timeout.unwrap_or(0);
        try_d3xx!(unsafe { ffi::FT_SetSuspendTimeout(self.handle, timeout) })?;
        Ok(())
    }

    /// Set the notification callback.
    ///
    /// The callback is invoked by the driver once a notification is received indicating
    /// data availability on a notification-enabled pipe. Pipes should not be read
    /// outside of the callback when notifications are enabled. Additionally the callback
    /// function should avoid blocking operations, as this may prevent the driver from
    /// processing other events.
    ///
    /// See the [`notification`](crate::notification) module for more information and
    /// an example.
    ///
    /// # Safety
    ///
    /// It is critical that the callback does not panic, as the callback is invoked through
    /// the driver and unwinding across the FFI boundary is not protected. This issue
    /// will be fixed in the future ([issue](https://github.com/mtmk-ee/d3xx/issues/4)).
    ///
    /// # Memory Leaks
    ///
    /// It is unknown whether the D3XX driver is responsible for freeing the memory allocated
    /// for the context when the callback is cleared because the documentation does not specify this.
    /// For now it is assumed that the memory is released when the callback is cleared/changed.
    /// Until this is confirmed, it is recommended to only set the callback a small number of times,
    /// and with a `T` that is small enough to not cause memory issues.
    ///
    /// # References
    /// See page 42 for more information:
    /// <https://ftdichip.com/wp-content/uploads/2020/07/AN_379-D3xx-Programmers-Guide-1.pdf>
    pub fn set_notification_callback<F, T>(&self, callback: F, context: Option<T>) -> Result<()>
    where
        T: Sync,
        F: Fn(Notification<T>) + 'static,
    {
        set_notification_callback(self.handle, callback, context)
    }

    /// Clear a previously-set notification callback.
    ///
    /// Note that this function is infallible, and it is unclear why due to conflicting
    /// documentation. On one hand the documentation says that the foreign function returns
    /// `FT_STATUS`, but on the other hand the header/bindings indicate that nothing is returned.
    /// It is therefore not possible to check the result of the operation.
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
            let _ = ffi::FT_Close(self.handle);
        }
    }
}

/// While a device is [`!Sync`](Sync), it is perfectly fine for it to be [`Send`]
/// because the device provides *unsynchronized* interior mutability, meaning that
/// the device is not protected by shared writes, but can be moved between threads
/// as long as it is not used concurrently.
unsafe impl Send for Device {}
