//! Miscellaneous utility functions.
//!
//! This module contains functions which are used internally by the crate, but are not
//! part of the public API. These functions may be changed or removed at any time.

use super::*;
use crate::{try_d3xx, D3xxError};

/// Write to a pipe synchronously.
///
/// If the operation fails it is the responsibility of the caller to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes written is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(windows)]
pub(crate) fn write_pipe(handle: FT_HANDLE, pipe: u8, buf: &[u8]) -> Result<usize> {
    let mut bytes_written: ULONG = 0;
    try_d3xx!(unsafe {
        FT_WritePipe(
            handle,
            pipe,
            buf.as_ptr().cast_mut(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_written),
            std::ptr::null_mut(),
        )
    })?;
    Ok(bytes_written as usize)
}

/// Write to a pipe synchronously.
///
/// If the operation fails it is the responsibility of the caller to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes written is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(not(windows))]
pub(crate) fn write_pipe(handle: FT_HANDLE, pipe: u8, buf: &[u8]) -> Result<usize> {
    let mut bytes_written: ULONG = 0;
    try_d3xx!(unsafe {
        FT_WritePipe(
            handle,
            pipe,
            buf.as_ptr().cast_mut(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_written),
            0,
        )
    })?;
    Ok(bytes_written as usize)
}

/// Asynchronous write to the specified pipe.
///
/// If the operation fails it is the responsibility of the user to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes written is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(windows)]
pub(crate) fn write_pipe_async(
    handle: FT_HANDLE,
    pipe: u8,
    buf: &[u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_written: ULONG = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_WritePipe(
            handle,
            pipe,
            buf.as_ptr().cast_mut(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_written),
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

/// Asynchronous write to the specified pipe.
///
/// If the operation fails it is the responsibility of the user to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes written is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(not(windows))]
pub fn write_pipe_async(
    handle: FT_HANDLE,
    pipe: u8,
    buf: &[u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_written: ULONG = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_WritePipeAsync(
            handle,
            pipe as u8,
            buf.as_ptr().cast_mut(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_written),
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

/// Read from a pipe synchronously.
///
/// If the operation fails it is the responsibility of the user to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes read is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(windows)]
pub(crate) fn read_pipe(handle: FT_HANDLE, pipe: u8, buf: &mut [u8]) -> Result<usize> {
    let mut bytes_read: ULONG = 0;
    try_d3xx!(unsafe {
        FT_ReadPipe(
            handle,
            pipe,
            buf.as_mut_ptr().cast(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_read),
            std::ptr::null_mut(),
        )
    })?;
    Ok(bytes_read as usize)
}

/// Read from a pipe synchronously.
///
/// If the operation fails it is the responsibility of the user to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes read is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(not(windows))]
pub(crate) fn read_pipe(handle: FT_HANDLE, pipe: u8, buf: &mut [u8]) -> Result<usize> {
    let mut bytes_read: ULONG = 0;
    try_d3xx!(unsafe {
        FT_ReadPipe(
            handle,
            pipe as u8,
            buf.as_mut_ptr().cast(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_read),
            0,
        )
    })?;
    Ok(bytes_read as usize)
}

/// Asynchronous read from the specified pipe.
///
/// If the operation fails it is the responsibility of the user to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes read is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(windows)]
pub(crate) fn read_pipe_async(
    handle: FT_HANDLE,
    pipe: u8,
    buf: &mut [u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_read: ULONG = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_ReadPipe(
            handle,
            pipe,
            buf.as_mut_ptr().cast(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_read),
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

/// Asynchronous read from the specified pipe.
///
/// If the operation fails it is the responsibility of the user to
/// abort any ongoing transfers for the pipe.
///
/// On success the number of bytes read is returned.
///
/// # Panics
///
/// Panics if `buf.len()` exceeds [`ULONG::MAX`]
#[cfg(not(windows))]
pub fn read_pipe_async(
    handle: FT_HANDLE,
    pipe: u8,
    buf: &mut [u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_read: ULONG = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_ReadPipeAsync(
            handle,
            pipe,
            buf.as_mut_ptr().cast(),
            ULONG::try_from(buf.len()).expect("buffer length exceeds ULONG::MAX"),
            std::ptr::addr_of_mut!(bytes_read),
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

/// Filter out `D3xxError::IoPending` errors, since they are expected for
/// asynchronous I/O operations.
#[inline]
fn ignore_io_pending(res: Result<()>) -> Result<()> {
    match res {
        Err(D3xxError::IoPending) => Ok(()),
        x => x,
    }
}
