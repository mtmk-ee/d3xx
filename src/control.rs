use crate::{ffi, Result};

type PhantomLifetime<'a> = &'a ();

pub enum ControlRequest {
    GetStatus,
    ClearFeature,
    SetFeature,
    SetAddress,
    GetDescriptor,
    SetDescriptor,
    GetConfiguration,
    SetConfiguration,
}

struct ControlTransfer<'a> {
    handle: ffi::HANDLE,
    _lifetime: PhantomLifetime<'a>,
}

impl ControlTransfer<'_> {
    pub fn status(&self) -> Result<DeviceStatus> {}

    pub fn clear_feature(&self) -> Result<()> {}
}

pub struct DeviceStatus {
    self_powered: bool,
    remote_wakeup: bool,
}

impl DeviceStatus {
    pub fn self_powered(&self) -> bool {
        self.self_powered
    }

    pub fn remote_wakeup(&self) -> bool {
        self.remote_wakeup
    }
}
