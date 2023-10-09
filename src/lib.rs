//! This crate provides a safe Rust wrapper around FTDI's D3XX driver API.
//!
//! **Disclaimer:** this crate is unofficial, and is not affiliated with FTDI in any way.
//!
//! **Note:** this crate is still in early development and is not yet ready for production use.
//! Feedback and contributions are welcome!
//!
//! The D3XX driver provides a low-level interface for interacting with `FT60x` devices.
//! It is intended to be used in conjunction with the `FT60x` "Super Speed" series of ICs, which provide
//! a USB 3.0 interface for transferring data at high speeds.
//!
//! The primary interface for interacting with devices is the [`Device`] struct. It provides methods
//! for reading, writing, configuration, and more. See the [`Device`] documentation for more details.
//!
//! # Simple Example
//!
//! ```no_run
//! use d3xx::{Device, Pipe};
//!
//! let device = Device::open("ABC123").unwrap();
//!
//! // Read 1024 bytes from input pipe 1
//! let mut buf = vec![0u8; 1024];
//! device.read(Pipe::In1, &mut buf).unwrap();
//!
//! // Write 1024 bytes to output pipe 1
//! device.write(Pipe::Out1, &buf).unwrap();
//! ```
//!
//! //! # Error Handling
//!
//! The documentation on most functions returning a `Result<T, D3xxError>` does not include an
//! explanation about the error conditions. This is because the D3XX documentation is vague,
//! providing little information about what errors can occur and under what circumstances.
//!
//! Catching specific errors in an API-backed manner is not possible, so it is recommended to
//! use a catch-all approach in most cases.

#![warn(clippy::all, clippy::pedantic, clippy::cargo, missing_docs)]
// Allow missing error documentation since the D3XX documentation is vague about error conditions.
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

mod descriptor;
mod device;
mod error;
mod ffi;
pub mod gpio;
mod overlapped;
mod pipe;
mod prelude;
mod scan;

pub use device::*;
pub use error::*;
pub use pipe::*;
pub use prelude::*;
pub use scan::*;

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
