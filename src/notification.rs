//! Notification support.
//!
//! # Background
//!
//! The notification system is used as a kind of interrupt-based mechanism when
//! receiving data from the device. Once an endpoint is enabled for notifications,
//! the driver will send a notification to the host when data is received on that
//! endpoint, which in turn invokes a user-defined callback. The notification
//! contains the number of bytes received, the endpoint that data was received on,
//! and an optional context value that can be set by the user.
//!
//! The notification system is ideal for endpoints sending short messages, such as
//! start/stop signals or status updates. It is not intended for endpoints that
//! send large amounts of data, such as a video stream.
//!
//! # Example
//!
//! A simple use case might look like this:
//!
//! ```no_run
//! use std::io::Read;
//! use std::sync::{Arc, Mutex};
//!
//! use d3xx::Device;
//! use d3xx::notification::{Notification, NotificationData};
//!
//! fn callback<'a>(notification: Notification<Arc<Mutex<Device>>>) {
//!     match notification.data() {
//!         NotificationData::Data { endpoint, size } => {
//!             let mut buf = vec![0; *size];
//!             let device = notification
//!                 .context()
//!                 .unwrap()
//!                 .lock()
//!                 .unwrap();
//!             device.
//!                 pipe(*endpoint)
//!                .read(&mut buf)
//!                .unwrap();
//!             println!("Data: {:?}", buf);
//!         }
//!         NotificationData::Gpio { gpio0, gpio1 } => {
//!             println!("GPIO0: {gpio0}, GPIO1: {gpio1}");
//!         }
//!     }
//! }
//!
//! let device = Device::open("serial number").unwrap();
//! let device = Arc::new(Mutex::new(device));
//! device
//!     .lock()
//!     .unwrap()
//!     .set_notification_callback(callback, Some(device.clone()))
//!     .unwrap();

use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, UnwindSafe},
};

use crate::{ffi, try_d3xx, D3xxError, Pipe, Result};

/// Information regarding a notification sent by a device.
///
/// [Notification callbacks](NotificationCallback) are called using a [`Notification`] as their
/// only argument. This struct contains the context provided by the user when setting the
/// callback, as well as the notification data sent by the device.
///
/// # Thread Safety
///
/// Any type can be used as the context as long as it is `Sync`. This constraint is necessary
/// because the callback is not guaranteed to be called on the same thread as the one that set
/// the callback, and because unwinding across the FFI boundary is undefined behavior.
pub struct Notification<T: Sync + UnwindSafe> {
    context: *const T,
    /// The notification data.
    data: NotificationData,
}

impl<T: Sync + UnwindSafe> Notification<T> {
    /// Get the context associated with this notification.
    #[must_use]
    pub fn context(&self) -> Option<&'static T> {
        unsafe { self.context.as_ref() }
    }

    /// Get the notification data.
    #[must_use]
    pub fn data(&self) -> &NotificationData {
        &self.data
    }
}

impl<T: Sync + UnwindSafe> UnwindSafe for Notification<T> {}

/// Notification callback context used internally.
///
/// This struct is used to provide [`trampoline`] with the necessary information to call the
/// user-provided callback. It allows users to set closures as callbacks, as well as use
/// arbitrary types as context. Otherwise, a rigid API using function pointers would be
/// required.
struct InternalContext<T, F>
where
    T: Sync + UnwindSafe,
    F: Fn(Notification<T>) + UnwindSafe,
{
    callback: F,
    context: Option<T>,
}

impl<T, F> InternalContext<T, F>
where
    T: Sync + UnwindSafe,
    F: Fn(Notification<T>) + UnwindSafe,
{
    fn context_ptr(&self) -> *const T {
        match &self.context {
            Some(context) => context as *const T,
            None => std::ptr::null(),
        }
    }

    fn callback(&self) -> &F {
        &self.callback
    }
}

impl<T, F> UnwindSafe for InternalContext<T, F>
where
    T: Sync + UnwindSafe,
    F: Fn(Notification<T>) + UnwindSafe,
{
}

/// Data associated with a [`Notification`].
///
/// Two variants are defined: `Data` and `Gpio`. The `Data` variant is used when
/// data is received on an endpoint, while the `Gpio` variant is used when the
/// state of the GPIO pins changes. Note that to receive either variant the
/// corresponding endpoint or GPIO pins must be enabled for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationData {
    /// Data notification.
    Data {
        /// The endpoint that the notification was triggered on.
        endpoint: Pipe,
        /// The number of bytes received.
        size: usize,
    },
    /// GPIO state notification. It is unclear from the documentation
    /// what the data in this variant represents.
    Gpio {
        /// The state of GPIO0.
        gpio0: usize,
        /// The state of GPIO1.
        gpio1: usize,
    },
}

/// Set a notification callback.
///
/// Internally this function registers a separate "trampoline" callback with the driver to
/// support different `T` parameters. The trampoline callback is responsible for calling the
/// user-provided callback with the correct types. For this, a struct containing extra
/// information is leaked to provide the trampoline with the necessary information.
///
/// # Warning
///
/// It is unknown whether the D3XX driver releases the leaked memory when the callback is
/// cleared because the documentation does not specify this. For now it is assumed that the
/// memory is released when the callback is cleared/changed. Until this is confirmed, it is
/// recommended to only set the callback smaller number of times, and with a `T` that is small
/// enough to not cause memory issues.
pub(crate) fn set_notification_callback<F, T>(
    handle: ffi::HANDLE,
    callback: F,
    context: Option<T>,
) -> Result<()>
where
    T: Sync + UnwindSafe,
    F: Fn(Notification<T>) + UnwindSafe,
{
    // TODO: determine whether the memory is freed when the callback is changed.
    // If it isn't, we can store a pointer to the context in the device and free it with a
    // destructor closure when the notification callback is changes or when the
    // device is closed.
    let internal_context = Box::into_raw(Box::new(InternalContext { callback, context }));
    try_d3xx!(unsafe {
        ffi::FT_SetNotificationCallback(handle, Some(trampoline::<T, F>), internal_context.cast())
    })
}

/// Clear the notification callback.
///
/// See the concerns about this in [`set_notification_callback`].
///
/// Note that this function is infallible, and it is unclear why due to incorrect
/// documentation. On one hand the documentation says that the foreign function returns
/// `FT_STATUS`, but on the other hand the header/bindings indicate that nothing is returned.
pub(crate) unsafe fn clear_notification_callback(handle: ffi::HANDLE) {
    unsafe {
        ffi::FT_ClearNotificationCallback(handle);
    }
}

/// Trampoline callback used to call the user-provided callback.
///
/// This function expects that the `callback_context` is a pointer to an [`InternalContext`] with
/// the same `T` parameter.
///
/// # Panics
///
/// This function will never panic, but the user-provided callback may panic. If this happens,
/// the panic will be caught and printed to stderr. It is not possible to propagate the panic
/// across the FFI boundary.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers and casts
/// between different types. To avoid undefined behavior, the caller must ensure
/// the following:
///
/// - `callback_context` is a valid pointer to an [`InternalContext<T>`] and `T` is correct.
/// - `callback_info` matches the corresponding `callback_type`.
unsafe extern "C" fn trampoline<T: Sync + UnwindSafe, F: Fn(Notification<T>) + UnwindSafe>(
    callback_context: *mut c_void,
    callback_type: ffi::E_FT_NOTIFICATION_CALLBACK_TYPE,
    callback_info: *mut c_void,
) {
    let data = extract_notification_data(callback_type, callback_info);
    if let Ok(data) = data {
        let context = &*(callback_context as *const InternalContext<T, F>);
        let callback = AssertUnwindSafe(context.callback());
        let notification = Notification {
            context: context.context_ptr(),
            data,
        };
        if let Err(e) = std::panic::catch_unwind(|| callback(notification)) {
            eprintln!("notification callback panicked: {e:?}");
        }
    };
}

/// Casts the callback info to the correct [`NotificationData`] variant.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers and casts
/// between different types. To avoid undefined behavior, the caller must ensure
/// that the `callback_info` is a valid object of the correct notification type.
unsafe fn extract_notification_data(
    callback_type: ffi::E_FT_NOTIFICATION_CALLBACK_TYPE,
    callback_info: *mut c_void,
) -> Result<NotificationData> {
    fn extract_data_variant(callback_info: *mut c_void) -> Result<NotificationData> {
        let callback_info =
            unsafe { *callback_info.cast::<ffi::FT_NOTIFICATION_CALLBACK_INFO_DATA>() };
        Ok(NotificationData::Data {
            endpoint: Pipe::try_from(callback_info.ucEndpointNo).or(Err(D3xxError::OtherError))?,
            size: callback_info.ulRecvNotificationLength as usize,
        })
    }

    fn extract_gpio_variant(callback_info: *mut c_void) -> NotificationData {
        let callback_info =
            unsafe { *callback_info.cast::<ffi::FT_NOTIFICATION_CALLBACK_INFO_GPIO>() };
        #[allow(clippy::cast_sign_loss)]
        NotificationData::Gpio {
            gpio0: callback_info.bGPIO0 as usize,
            gpio1: callback_info.bGPIO1 as usize,
        }
    }

    match callback_type {
        ffi::E_FT_NOTIFICATION_CALLBACK_TYPE::E_FT_NOTIFICATION_CALLBACK_TYPE_DATA => {
            extract_data_variant(callback_info)
        }
        ffi::E_FT_NOTIFICATION_CALLBACK_TYPE::E_FT_NOTIFICATION_CALLBACK_TYPE_GPIO => {
            Ok(extract_gpio_variant(callback_info))
        }
        #[cfg(not(windows))]
        _ => Err(D3xxError::OtherError),
    }
}
