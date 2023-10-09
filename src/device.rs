use std::{
    ffi::{c_void, CString},
    marker::PhantomData,
};

use crate::{
    descriptor::{ConfigurationDescriptor, DeviceDescriptor, InterfaceDescriptor},
    ffi, try_d3xx, Pipe, PipeId, Result,
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
/// use d3xx::{Device, Pipe};
///
/// let device = Device::open("ABC123").unwrap();
///
/// // Read 1024 bytes from input pipe 1
/// let mut buf = vec![0u8; 1024];
/// device.read(Pipe::In1, &mut buf).unwrap();
///
/// // Write 1024 bytes to output pipe 1
/// device.write(Pipe::Out1, &buf).unwrap();
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

    /// Returns a [`Pipe`] for pipe I/O and configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
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
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            ffi::FT_Close(self.handle);
        }
    }
}
