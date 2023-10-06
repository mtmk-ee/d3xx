# d3xx

This crate provides a safe Rust wrapper around FTDI's D3XX driver API.

**Disclaimer:** this crate is unofficial, and is not affiliated with FTDI in any way.

**Note:** this crate is still in early development and is not yet ready for production use.
Feedback and contributions are welcome!

The D3XX driver provides a low-level interface for interacting with FT60x devices.
It is intended to be used in conjunction with the FT60x "Super Speed" series of ICs, which provide
a USB 3.0 interface for transferring data at high speeds.

The primary interface for interacting with devices is the [`Device`] struct. It provides methods
for reading, writing, configuration, and more. See the [`Device`] documentation for more details.

# Simple Example

```rust
use d3xx::{Device, Pipe};

let device = Device::open("ABC123").unwrap();

// Read 1024 bytes from input pipe 1
let mut buf = vec![0u8; 1024];
device.read(Pipe::In1, &mut buf).unwrap();

// Write 1024 bytes to output pipe 1
device.write(Pipe::Out1, &buf).unwrap();
```
