pub mod util;

use std::{sync::Mutex};

pub use libftd3xx_ffi::*;

use crate::Result;

/// Global lock is necessary for certain operations when working with the D3XX driver.
static mut GLOBAL_LOCK: Mutex<()> = Mutex::new(()); // FIXME: is a reentrant mutex needed?

/// Run the given closure with the global lock held.
///
/// This is necessary for certain operations when working with the D3XX driver.
/// For example, listing devices must be done with the lock held since the
/// operation consists of a write followed by a read of the driver's device table,
/// which may by invalidated at any point by another thread.
pub fn with_global_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        let _guard = GLOBAL_LOCK.lock().unwrap();
        f()
    }
}
