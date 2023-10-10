pub mod util;

use std::{panic::catch_unwind, sync::Mutex};

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

#[allow(clippy::missing_panics_doc)]
pub fn with_global_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    unsafe {
        // unwrap() is safe because we ensure below that the lock is not poisoned.
        let lock = GLOBAL_LOCK.lock().unwrap();
        match catch_unwind(f) {
            Ok(result) => result,
            Err(e) => {
                drop(lock);
                panic!("panicked while holding global lock: {e:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_lock() {
        let _guard = unsafe { GLOBAL_LOCK.lock().unwrap() };
        assert!(unsafe { GLOBAL_LOCK.try_lock() }.is_err());
    }
}
