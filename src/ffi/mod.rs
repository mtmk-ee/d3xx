#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

pub(crate) mod util;

use std::{
    ffi::{c_uchar, c_ulong, c_void},
    sync::Mutex,
};

use crate::{try_d3xx, Result};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Global lock is necessary for certain operations when working with the D3XX driver.
static mut GLOBAL_LOCK: Mutex<()> = Mutex::new(()); // FIXME: is a reentrant mutex needed?

pub(crate) fn with_global_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        let _guard = GLOBAL_LOCK.lock().unwrap();
        f()
    }
}
