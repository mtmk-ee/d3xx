use std::fmt::Display;

use crate::ffi;

pub type Result<T, E = D3xxError> = std::result::Result<T, E>;

/// Represents an error returned by the D3XX API.
///
/// Codes 1 through 32 are defined as error by the API.
///
/// If necessary, a [`D3xxError`] may be constructed from an error code:
///
/// ```
/// use d3xx::D3xxError;
///
/// let err = D3xxError::from(1);
/// assert_eq!(err, D3xxError::InvalidHandle);
/// ```
///
/// Note that the `from` method will panic if the given code is invalid.
#[allow(unused)]
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
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
    #[must_use] pub fn code(&self) -> u8 {
        *self as u8
    }
}

impl Display for D3xxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.code();
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

macro_rules! try_d3xx {
    ($expr:expr) => {
        match $expr {
            0 => Ok(()),
            code => Err(crate::error::D3xxError::from(code)),
        }
    };
}

pub(crate) use try_d3xx;
