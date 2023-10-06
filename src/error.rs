use std::fmt::Display;

use crate::ffi;

pub type Result<T, E = D3xxError> = std::result::Result<T, E>;

#[allow(unused)]
#[derive(thiserror::Error, Debug, Clone, Copy)]
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
    pub fn code(&self) -> u8 {
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
            1 => D3xxError::InvalidHandle,
            2 => D3xxError::DeviceNotFound,
            3 => D3xxError::DeviceNotOpened,
            4 => D3xxError::IoError,
            5 => D3xxError::InsufficientResources,
            6 => D3xxError::InvalidParameter,
            7 => D3xxError::InvalidBaudRate,
            8 => D3xxError::DeviceNotOpenedForErase,
            9 => D3xxError::DeviceNotOpenedForWrite,
            10 => D3xxError::FailedToWriteDevice,
            11 => D3xxError::EEPROMReadFailed,
            12 => D3xxError::EEPROMWriteFailed,
            13 => D3xxError::EEPROMEraseFailed,
            14 => D3xxError::EEPROMNotPresent,
            15 => D3xxError::EEPROMNotProgrammed,
            16 => D3xxError::InvalidArgs,
            17 => D3xxError::NotSupported,
            18 => D3xxError::NoMoreItems,
            19 => D3xxError::Timeout,
            20 => D3xxError::OperationAborted,
            21 => D3xxError::ReservedPipe,
            22 => D3xxError::InvalidControlRequestDirection,
            23 => D3xxError::InvalidControLRequestType,
            24 => D3xxError::IoPending,
            25 => D3xxError::IoIncomplete,
            26 => D3xxError::HandleEof,
            27 => D3xxError::Busy,
            28 => D3xxError::NoSystemResources,
            29 => D3xxError::DeviceListNotReady,
            30 => D3xxError::DeviceNotConnected,
            31 => D3xxError::IncorrectDevicePath,
            _ => D3xxError::OtherError,
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
