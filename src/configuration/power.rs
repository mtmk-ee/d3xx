pub struct PowerConfig {
    flags: u8,
    max_power: u16,
}

impl PowerConfig {
    pub fn new(flags: u8, max_power: u16) -> Self {
        Self { flags, max_power }
    }

    pub fn bus_powered(&self) -> bool {
        !self.self_powered()
    }

    pub fn self_powered(&self) -> bool {
        self.flags & 0x40 != 0
    }

    pub fn remote_wakeup(&self) -> bool {
        self.flags & 0x20 != 0
    }

    pub fn max_power(&self) -> u16 {
        self.max_power
    }
}
