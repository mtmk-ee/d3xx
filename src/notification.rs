//! Notification support.
//!
//! This module contains types used in notification callbacks.
//! Note that callbacks are only ever invoked on notification-enabled
//! enpoints.
//!
//! A simple callback might look like this:
//!
//! ```no_run
//! use d3xx::notification::{Notification, NotificationData};
//!
//! fn callback(notification: Notification<String>) {
//!     match notification.data() {
//!         NotificationData::Data { endpoint, size } => {
//!             println!("Received {size} bytes on endpoint {endpoint:?}");
//!         }
//!         NotificationData::Gpio { gpio0, gpio1 } => {
//!             println!("GPIO0: {gpio0}, GPIO1: {gpio1}");
//!         }
//!     }
//!     // context is set when the callback is set
//!     println!("Context: {:?}", notification.context());
//! }
use std::ffi::c_void;

use crate::{error::try_d3xx, ffi, pipe::Endpoint, Result};

type NotificationCallback<T> = dyn Fn(Notification<T>) + 'static;

/// Information regarding a notification sent by a device.
///
/// Notification callbacks are called using this struct.
pub struct Notification<'a, T> {
    /// The context provided by the user when setting the callback.
    ///
    /// The type inside the `Option` is a reference to the context
    /// because the context is owned by the driver.
    context: Option<&'a T>,
    /// The notification data.
    data: NotificationData,
}

impl<T> Notification<'_, T> {
    /// Get the context associated with this notification.
    #[must_use]
    pub fn context(&self) -> Option<&T> {
        self.context
    }

    /// Get the notification data.
    #[must_use]
    pub fn data(&self) -> &NotificationData {
        &self.data
    }
}

/// Notification callback context used internally.
///
/// This struct is used to provide [`trampoline`] with the necessary information to call the
/// user-provided callback. It allows users to set closures as callbacks, as well as use
/// arbitrary types as context. Otherwise, a rigid API using function pointers would be
/// required.
struct InternalContext<T> {
    /// The user-provided callback.
    callback: Box<NotificationCallback<T>>,
    /// The context provided by the user when setting the callback.
    context: Option<T>,
}

/// Data associated with a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationData {
    /// Data notification.
    Data {
        /// The endpoint that the notification was triggered on.
        endpoint: Endpoint,
        /// The number of bytes received.
        size: usize,
    },
    /// GPIO state notification. It is unclear from the documentation
    /// what the data in this variant represents.
    Gpio {
        /// The state of GPIO0.
        gpio0: i32,
        /// The state of GPIO1.
        gpio1: i32,
    },
}

/// Set a notification callback.
///
/// Internally this function registers a separate "trampoline" callback with the driver to
/// support different `T` parameters. The trampoline callback is responsible for calling the
/// user-provided callback with the correct types. For this, a struct containing extra
/// information is leaked to provide the trampoline with the necessary information.
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
    F: Fn(Notification<T>) + 'static,
{
    // TODO: determine whether the memory is freed when the callback is changed.
    // If it isn't, we can store a pointer to the context in the device and free it with a
    // destructor closure when the notification callback is changes or when the
    // device is closed.
    let internal_context = Box::into_raw(Box::new(InternalContext {
        callback: Box::new(callback),
        context,
    }));
    try_d3xx!(unsafe {
        ffi::FT_SetNotificationCallback(handle, Some(trampoline::<T>), internal_context.cast())
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
extern "C" fn trampoline<T>(
    callback_context: *mut c_void,
    callback_type: ffi::E_FT_NOTIFICATION_CALLBACK_TYPE,
    callback_info: *mut c_void,
) {
    let data = match callback_type {
        ffi::E_FT_NOTIFICATION_CALLBACK_TYPE::E_FT_NOTIFICATION_CALLBACK_TYPE_DATA => {
            let callback_info =
                unsafe { *callback_info.cast::<ffi::FT_NOTIFICATION_CALLBACK_INFO_DATA>() };
            NotificationData::Data {
                endpoint: match Endpoint::try_from(callback_info.ucEndpointNo) {
                    Ok(endpoint) => endpoint,
                    Err(_) => return, // would rather not unwind across the FFI boundary
                },
                size: callback_info.ulRecvNotificationLength as usize,
            }
        }
        ffi::E_FT_NOTIFICATION_CALLBACK_TYPE::E_FT_NOTIFICATION_CALLBACK_TYPE_GPIO => {
            let callback_info =
                unsafe { *callback_info.cast::<ffi::FT_NOTIFICATION_CALLBACK_INFO_GPIO>() };
            NotificationData::Gpio {
                gpio0: callback_info.bGPIO0,
                gpio1: callback_info.bGPIO1,
            }
        }
    };
    let context = unsafe { &*(callback_context as *const InternalContext<T>) };
    let notification = Notification {
        context: context.context.as_ref(),
        data,
    };
    (context.callback)(notification);
}
