/// Container for the string descriptors of a [`Device`](crate::Device).
///
/// The D3XX API provides access to the string descriptor as a little-endian UTF-16
/// byte array. This struct provides a more convenient interface.
pub struct StringDescriptor {
    manufacturer: String,
    product: String,
    serial_number: String,
}

impl StringDescriptor {
    pub(crate) fn new(descriptor: [u8; 128]) -> Self {
        Self {
            manufacturer: Self::extract_part(&descriptor, 0),
            product: Self::extract_part(&descriptor, 1),
            serial_number: Self::extract_part(&descriptor, 2),
        }
    }

    /// Manufacturer name.
    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    /// Set the manufacturer name.
    ///
    /// The string will be converted to UTF-16 and truncated to 30 characters
    /// when written to the device.
    pub fn set_manufacturer(&mut self, manufacturer: &str) {
        self.manufacturer = manufacturer.to_owned();
    }

    /// Product name.
    pub fn product(&self) -> &str {
        &self.product
    }

    /// Set the product name.
    ///
    /// The string will be converted to UTF-16 and truncated to 62 characters
    /// when written to the device.
    pub fn set_product(&mut self, product: &str) {
        self.product = product.to_owned();
    }

    /// Serial number.
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Set the serial number.
    ///
    /// The string will be converted to UTF-16 and truncated to 30 characters
    /// when written to the device.
    pub fn set_serial_number(&mut self, serial_number: &str) {
        self.serial_number = serial_number.to_owned();
    }

    fn extract_part(descriptors: &[u8], index: usize) -> String {
        const HEADER_SIZE: usize = 2;
        assert!(index < 3);
        let mut descriptor_start = 0;
        for _ in 0..index {
            // first byte is the length of the descriptor (including the header)
            descriptor_start += descriptors[descriptor_start] as usize;
        }
        let len = descriptors[descriptor_start] as usize - HEADER_SIZE;
        let wide_chars = descriptors[descriptor_start + HEADER_SIZE..][..len]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&wide_chars)
    }

    pub fn as_ffi_descriptor(&self) -> [u8; 128] {
        fn set_part(slice: &mut [u8], s: &[u8]) {
            // unwrap is safe because the length of the string is always <= 62
            slice[0] = u8::try_from(s.len() + 2).unwrap();
            slice[1] = 0x03;
            slice[2..][..s.len()].copy_from_slice(s);
        }

        let manufacturer = str_to_utf16(&self.manufacturer, 30);
        let product = str_to_utf16(&self.product, 62);
        let serial_number = str_to_utf16(&self.serial_number, 30);
        let mut descriptor = [0u8; 128];
        let mut offset = 0;

        for str in [&manufacturer, &product, &serial_number] {
            set_part(&mut descriptor[offset..], str);
            offset += descriptor[offset] as usize;
        }
        descriptor
    }
}

fn str_to_utf16(string: &str, max: usize) -> Vec<u8> {
    string
        .encode_utf16()
        .take(max)
        .flat_map(u16::to_le_bytes)
        .collect()
}
