//! This crate provides a safe, idiomatic Rust wrapper around FTDI's D3XX driver API.
//!
//! # Disclaimer
//!
//! This crate is unofficial and is not affiliated with FTDI in any way.
//!
//! The crate is still in early development and is not yet ready for production use.
//! Feedback and contributions are welcome!
//!
//! # What This Crate Does
//!
//! This crate provides much of the functionality of the D3XX API, including:
//! - Device enumeration
//! - Reading device configurations
//! - Pipe I/O
//! - GPIO control
//! - Notifications
//! - Overlapped (Asynchronous) I/O
//!
//! This crate does not wrap functionality for configuring the device. If it is deemed necessary,
//! the unsafe FFI functions may be called directly. However, it is recommended to use the
//! [FT60X Chip Configuration Programmer](https://ftdichip.com/utilities/) instead for this purpose.
//!
//! # Background
//!
//! Future Technology Devices International (FTDI) produces the FT60X series of chips (e.g. FT601),
//! which act as SuperSpeed USB 3.0 to FIFO bridges. FTDI provides a proprietary driver for these chips,
//! called D3XX, which exposes a low-level API for interacting with the devices through its DLL/shared library.
//!
//! ## Brief Refresher on USB 3.0
//!
//! ### Terminology
//! USB peripherals contain a series of numbered endpoints, which are essentially physical data buffers. Each endpoint may contain
//! one or two buffers, corresponding to the direction of data flow (IN or OUT). D3XX devices have 8 endpoints, 4 each for IN and OUT transfers.
//! The software representation of an endpoint is known as a "pipe" and is unidirectional.
//!
//! Endpoints are collected into "interfaces", which are logical groupings of endpoints that serve a common purpose.
//! These interfaces are then collected into "configurations", which are collections of interfaces that represent
//! a complete set of functionality for the device. A device may have multiple configurations, but only one may be active
//! at a time.
//!
//! ### Transfers
//!
//! Data is transferred to and from the device in packets called "transactions". There are four types of transfers
//! that can be performed:
//!
//! - Control transfers: mainly used for configuration and identification by the host.
//! - Bulk transfers: used for large data transfers where correctness is prioritized over throughput.
//! - Isochronous transfers: used for streaming data where throughput is prioritized over correctness.
//! - Interrupt transfers: used for small data transfers that require low latency.
//!
//! For example, a keyboard or mouse would use interrupt transfers, while a web camera would use isochronous transfers.
//!
//!
//!
//! # Simple Example
//!
//! ```no_run
//! use std::io::{Read, Write};
//! use d3xx::{list_devices, Pipe};
//!
//! // Scan for connected devices.
//! let all_devices = list_devices().expect("failed to list devices");
//!
//! /// Open the first device found.
//! let device = all_devices[0].open().expect("failed to open device");
//!
//! // Read 1024 bytes from input pipe 1
//! let mut buf = vec![0; 1024];
//! device
//!     .pipe(Pipe::In1)
//!     .read(&mut buf)
//!     .expect("failed to read from pipe");
//!
//! // Write 1024 bytes to output pipe 2
//! device
//!     .pipe(Pipe::Out2)
//!     .write(&buf)
//!     .expect("failed to write to pipe");
//! ```
//!
//! # Error Handling
//!
//! The documentation on most functions returning a `Result<T, D3xxError>` does not include an
//! explanation about the error conditions. This is because in most cases the D3XX documentation
//! does not provide any information about what errors can occur and under what circumstances.
//!
//! Because of the lack of a clear specification it is not wise to attempt to handle specific
//! errors in a programmatic manner, as the error conditions may change in future versions of the
//! D3XX API. Instead, it is recommended to use a catch-all approach in most cases.
//!
//! # Further Reading
//!
//! It is recommended to read the [D3XX Programmers Guide](https://ftdichip.com/wp-content/uploads/2020/07/AN_379-D3xx-Programmers-Guide-1.pdf)
//! for more information about the capabilities provided by the D3XX API.

#![warn(clippy::all, clippy::pedantic, clippy::cargo, missing_docs)]
// Allow missing error documentation since the D3XX documentation is vague about error conditions.
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

#[cfg(feature = "config")]
pub mod configuration;
pub mod descriptor;
mod device;
mod error;
pub mod ffi;
mod gpio;
pub mod notification;
mod overlapped;
mod pipe;
mod prelude;
mod scan;

pub use device::Device;
pub(crate) use error::try_d3xx;
pub use error::{D3xxError, Result};
pub use gpio::{Direction, Gpio, GpioPin, Level, PullMode};
pub use pipe::{Endpoint, Pipe, PipeIo, PipeType};
pub use scan::{list_devices, DeviceInfo, DeviceType};

/// Get the version of the D3XX library.
pub fn library_version() -> Result<Version> {
    let mut version: u32 = 0;
    try_d3xx!(unsafe { ffi::FT_GetLibraryVersion(&mut version) })?;
    Ok(Version(version))
}

/// D3XX library or driver version.
pub struct Version(u32);

impl Version {
    /// Major version number.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn major(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// Minor version number.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn minor(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// Build/subversion version number.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn build(&self) -> u16 {
        self.0 as u16
    }
}
