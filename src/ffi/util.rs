use std::ffi::c_ulong;

use crate::{try_d3xx, D3xxError};

use super::*;

pub fn write_pipe(handle: FT_HANDLE, pipe: u8, buf: &[u8]) -> Result<usize> {
    let mut bytes_written: c_ulong = 0;
    try_d3xx!(unsafe {
        FT_WritePipe(
            handle,
            pipe as u8,
            buf.as_ptr() as *mut c_uchar,
            buf.len() as c_ulong,
            &mut bytes_written as *mut c_ulong,
            std::ptr::null_mut(),
        )
    })?;
    Ok(bytes_written as usize)
}

#[cfg(windows)]
pub fn write_pipe_async(
    handle: FT_HANDLE,
    pipe: u8,
    buf: &[u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_written: c_ulong = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_WritePipe(
            handle,
            pipe as u8,
            buf.as_ptr() as *mut c_uchar,
            buf.len() as c_ulong,
            &mut bytes_written as *mut c_ulong,
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

#[cfg(not(windows))]
pub fn write_pipe_async(
    device: &Device,
    pipe: u8,
    buf: &[u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_written: c_ulong = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_WritePipeAsync(
            handle,
            pipe as u8,
            buf.as_ptr() as *mut c_uchar,
            buf.len() as c_ulong,
            &mut bytes_written as *mut c_ulong,
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

pub fn read_pipe(handle: FT_HANDLE, pipe: u8, buf: &mut [u8]) -> Result<usize> {
    let mut bytes_read: c_ulong = 0;
    try_d3xx!(unsafe {
        FT_ReadPipe(
            handle,
            pipe as u8,
            buf.as_mut_ptr() as *mut c_uchar,
            buf.len() as c_ulong,
            &mut bytes_read as *mut c_ulong,
            std::ptr::null_mut(),
        )
    })?;
    Ok(bytes_read as usize)
}

#[cfg(windows)]
pub fn read_pipe_async(
    handle: FT_HANDLE,
    pipe: u8,
    buf: &mut [u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_read: c_ulong = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_ReadPipe(
            handle,
            pipe as u8,
            buf.as_mut_ptr() as *mut c_uchar,
            buf.len() as c_ulong,
            &mut bytes_read as *mut c_ulong,
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

#[cfg(not(windows))]
pub fn read_pipe_async(
    handle: FT_HANDLE,
    pipe: u8,
    buf: &mut [u8],
    overlapped: &mut _OVERLAPPED,
) -> Result<()> {
    let mut bytes_read: c_ulong = 0;
    ignore_io_pending(try_d3xx!(unsafe {
        FT_ReadPipeAsync(
            handle,
            pipe as u8,
            buf.as_mut_ptr() as *mut c_uchar,
            buf.len() as c_ulong,
            &mut bytes_read as *mut c_ulong,
            overlapped as *mut _OVERLAPPED,
        )
    }))
}

fn ignore_io_pending(res: Result<()>) -> Result<()> {
    match res {
        Err(D3xxError::IoPending) => Ok(()),
        x => x,
    }
}
