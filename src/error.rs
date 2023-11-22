use std::fmt::Display;

use crate::ffi;

/// A specialized [`Result`] type alias for D3XX operations.
///
/// See [`D3xxError`] for a list of possible errors, and [`try_d3xx`] for a macro
/// producing [`Result`]s from D3XX error codes.
pub type Result<T, E = D3xxError> = std::result::Result<T, E>;

/// Represents an error returned by the D3XX library.
///
/// # Warning About Error Variants
///
/// The documentation on most functions in this crate returning a `Result<T, D3xxError>` does not include an
/// explanation about the error conditions. This is because in most cases the D3XX documentation
/// does not provide any information about what errors can occur and under what circumstances.
/// Because there is no specification for errors, it is not wise to attempt to handle specific
/// errors in a systematic manner, as the error conditions may change in future versions of the
/// D3XX API. Instead, it is recommended to use a catch-all approach in most cases.
///
/// Note also that the error variants are explicitly annotated with `#[non_exhaustive]`. This is
/// to allow for the possibility of new error variants being added in future versions of the D3XX
/// API without the need to introduce breaking changes.
///
/// # Error Codes
///
/// Codes 1 through 32 (inclusive) are defined as an error by the API, while 0 represents success.
/// All other error codes are treated as [`D3xxError::OtherError`]. The [`From`] implementation for
/// [`D3xxError`] maps these codes to the corresponding error variant. Note that the `from` method
/// will panic if the given code is invalid.
///
/// ```
/// use d3xx::D3xxError;
///
/// let err = D3xxError::from(1);
/// assert_eq!(err, D3xxError::InvalidHandle);
/// ```
///
#[allow(unused, clippy::module_name_repetitions, missing_docs)]
#[repr(u8)]
#[non_exhaustive]
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum D3xxError {
    InvalidHandle = 1,
    DeviceNotFound,
    DeviceNotOpened,
    IoError,
    InsufficientResources,
    InvalidParameter,
    InvalidBaudRate,
    DeviceNotOpenedForErase,
    DeviceNotOpenedForWrite,
    FailedToWriteDevice,
    EEPROMReadFailed,
    EEPROMWriteFailed,
    EEPROMEraseFailed,
    EEPROMNotPresent,
    EEPROMNotProgrammed,
    InvalidArgs,
    NotSupported,

    NoMoreItems,
    Timeout,
    OperationAborted,
    ReservedPipe,
    InvalidControlRequestDirection,
    InvalidControLRequestType,
    IoPending,
    IoIncomplete,
    HandleEof,
    Busy,
    NoSystemResources,
    DeviceListNotReady,
    DeviceNotConnected,
    IncorrectDevicePath,

    OtherError,
}

impl D3xxError {
    /// Get the error code as an integer.
    ///
    /// Error codes are defined using values from 1 to 32 (inclusive).
    #[must_use]
    pub fn code(&self) -> u8 {
        *self as u8
    }
}

impl Display for D3xxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.code();
        // VARIANT_NAME (error code CODE)
        write!(f, "{self:?} (error code {code})")
    }
}

impl From<ffi::FT_STATUS> for D3xxError {
    fn from(value: ffi::FT_STATUS) -> Self {
        match value {
            0 => panic!("success is not an error"),
            1 => Self::InvalidHandle,
            2 => Self::DeviceNotFound,
            3 => Self::DeviceNotOpened,
            4 => Self::IoError,
            5 => Self::InsufficientResources,
            6 => Self::InvalidParameter,
            7 => Self::InvalidBaudRate,
            8 => Self::DeviceNotOpenedForErase,
            9 => Self::DeviceNotOpenedForWrite,
            10 => Self::FailedToWriteDevice,
            11 => Self::EEPROMReadFailed,
            12 => Self::EEPROMWriteFailed,
            13 => Self::EEPROMEraseFailed,
            14 => Self::EEPROMNotPresent,
            15 => Self::EEPROMNotProgrammed,
            16 => Self::InvalidArgs,
            17 => Self::NotSupported,
            18 => Self::NoMoreItems,
            19 => Self::Timeout,
            20 => Self::OperationAborted,
            21 => Self::ReservedPipe,
            22 => Self::InvalidControlRequestDirection,
            23 => Self::InvalidControLRequestType,
            24 => Self::IoPending,
            25 => Self::IoIncomplete,
            26 => Self::HandleEof,
            27 => Self::Busy,
            28 => Self::NoSystemResources,
            29 => Self::DeviceListNotReady,
            30 => Self::DeviceNotConnected,
            31 => Self::IncorrectDevicePath,
            _ => Self::OtherError,
        }
    }
}

impl From<D3xxError> for std::io::Error {
    /// Allows propagation of D3XX errors as [`std::io::Error`]s.
    ///
    /// This implementation is useful for functions which return [`io::Result<T>`](std::io::Result),
    /// such as the methods found in [`io::Write`](std::io::Write).
    fn from(value: D3xxError) -> Self {
        Self::new(std::io::ErrorKind::Other, value)
    }
}

/// A handy macro for converting D3XX error codes to [`Result<T, D3xxError>`]s.
///
/// The majority of D3XX functions return an integral status code. This macro
/// maps the status code to a [`Result<T, D3xxError>`] where `T` is `()`.
///
/// # Example
///
/// The following example demonstrates how to use the macro. Normally you would
/// use the macro to wrap a function call, rather than using a constant.
///
/// ```ignore
/// use d3xx::try_d3xx;
///
/// try_d3xx!(0).unwrap(); // Ok
/// try_d3xx!(1).unwrap(); // Error!
/// ```
macro_rules! try_d3xx {
    ($expr:expr) => {
        match $expr {
            0 => Ok(()),
            code => Err(crate::error::D3xxError::from(code)),
        }
    };
}
pub(crate) use try_d3xx;

#[cfg(test)]
mod tests {
    use super::*;

    const ERROR_CODE_MAP: [(D3xxError, ffi::FT_STATUS); 32] = [
        (D3xxError::InvalidHandle, 1),
        (D3xxError::DeviceNotFound, 2),
        (D3xxError::DeviceNotOpened, 3),
        (D3xxError::IoError, 4),
        (D3xxError::InsufficientResources, 5),
        (D3xxError::InvalidParameter, 6),
        (D3xxError::InvalidBaudRate, 7),
        (D3xxError::DeviceNotOpenedForErase, 8),
        (D3xxError::DeviceNotOpenedForWrite, 9),
        (D3xxError::FailedToWriteDevice, 10),
        (D3xxError::EEPROMReadFailed, 11),
        (D3xxError::EEPROMWriteFailed, 12),
        (D3xxError::EEPROMEraseFailed, 13),
        (D3xxError::EEPROMNotPresent, 14),
        (D3xxError::EEPROMNotProgrammed, 15),
        (D3xxError::InvalidArgs, 16),
        (D3xxError::NotSupported, 17),
        (D3xxError::NoMoreItems, 18),
        (D3xxError::Timeout, 19),
        (D3xxError::OperationAborted, 20),
        (D3xxError::ReservedPipe, 21),
        (D3xxError::InvalidControlRequestDirection, 22),
        (D3xxError::InvalidControLRequestType, 23),
        (D3xxError::IoPending, 24),
        (D3xxError::IoIncomplete, 25),
        (D3xxError::HandleEof, 26),
        (D3xxError::Busy, 27),
        (D3xxError::NoSystemResources, 28),
        (D3xxError::DeviceListNotReady, 29),
        (D3xxError::DeviceNotConnected, 30),
        (D3xxError::IncorrectDevicePath, 31),
        (D3xxError::OtherError, 32),
    ];

    #[test]
    fn test_d3xx_error_codes() {
        for (variant, code) in ERROR_CODE_MAP {
            assert_eq!(D3xxError::from(code), variant);
            assert_eq!(u32::from(variant.code()), code);
        }
    }

    #[test]
    fn test_try_d3xx_macro() {
        assert_eq!(try_d3xx!(0), Ok(()));
        for (variant, code) in ERROR_CODE_MAP {
            assert_eq!(try_d3xx!(code), Err(variant));
        }
    }
}
