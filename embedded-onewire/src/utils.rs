#[derive(Debug, Default)]
/// Calculate CRC-8 used in 1-Wire communications.
pub struct OneWireCrc(u8);

impl OneWireCrc {
    /// Get the current CRC value
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Update the CRC with the incoming byte.
    ///
    /// # Arguments
    /// * `byte` - The byte to update the CRC with.
    ///
    /// # Note
    /// This method uses a lookup table for CRC calculation if the `crc-table` feature is enabled.
    /// Otherwise, it uses bit shifts and XOR operations for CRC calculation.
    #[inline(always)]
    pub fn update(&mut self, byte: u8) {
        #[cfg(feature = "crc-table")]
        {
            self.update_table(byte); // Use the lookup table for CRC calculation
        }
        #[cfg(not(feature = "crc-table"))]
        {
            self.update_calc(byte); // Use the direct calculation for CRC
        }
    }

    /// Valudate a sequence of bytes where the last byte is the 1-Wire CRC of
    /// the previous bytes.
    ///
    /// # Note
    /// For such a sequence, the CRC should be `0x00`.
    pub fn validate(sequence: &[u8]) -> bool {
        let mut crc = OneWireCrc(0);
        for &byte in sequence.iter() {
            crc.update(byte); // Update CRC with the all bytes of the ROM
        }
        crc.0 == 0x0 // If the last byte of the ROM is the CRC, the result should be 0
    }

    #[allow(dead_code)]
    pub(crate) fn update_table(&mut self, byte: u8) {
        const ONEWIRE_SRC_TABLE: [u8; 256] = [
            0, 94, 188, 226, 97, 63, 221, 131, 194, 156, 126, 32, 163, 253, 31, 65, 157, 195, 33,
            127, 252, 162, 64, 30, 95, 1, 227, 189, 62, 96, 130, 220, 35, 125, 159, 193, 66, 28,
            254, 160, 225, 191, 93, 3, 128, 222, 60, 98, 190, 224, 2, 92, 223, 129, 99, 61, 124,
            34, 192, 158, 29, 67, 161, 255, 70, 24, 250, 164, 39, 121, 155, 197, 132, 218, 56, 102,
            229, 187, 89, 7, 219, 133, 103, 57, 186, 228, 6, 88, 25, 71, 165, 251, 120, 38, 196,
            154, 101, 59, 217, 135, 4, 90, 184, 230, 167, 249, 27, 69, 198, 152, 122, 36, 248, 166,
            68, 26, 153, 199, 37, 123, 58, 100, 134, 216, 91, 5, 231, 185, 140, 210, 48, 110, 237,
            179, 81, 15, 78, 16, 242, 172, 47, 113, 147, 205, 17, 79, 173, 243, 112, 46, 204, 146,
            211, 141, 111, 49, 178, 236, 14, 80, 175, 241, 19, 77, 206, 144, 114, 44, 109, 51, 209,
            143, 12, 82, 176, 238, 50, 108, 142, 208, 83, 13, 239, 177, 240, 174, 76, 18, 145, 207,
            45, 115, 202, 148, 118, 40, 171, 245, 23, 73, 8, 86, 180, 234, 105, 55, 213, 139, 87,
            9, 235, 181, 54, 104, 138, 212, 149, 203, 41, 119, 244, 170, 72, 22, 233, 183, 85, 11,
            136, 214, 52, 106, 43, 117, 151, 201, 74, 20, 246, 168, 116, 42, 200, 150, 21, 75, 169,
            247, 182, 232, 10, 84, 215, 137, 107, 53,
        ];
        self.0 = ONEWIRE_SRC_TABLE[(self.0 ^ byte) as usize];
    }

    #[allow(dead_code)]
    pub(crate) fn update_calc(&mut self, byte: u8) {
        let mut crc = self.0 ^ byte;
        for _ in 0..8 {
            if crc & 0x01 == 0x01 {
                crc = (crc >> 1) ^ 0x8C; // Polynomial: x^8 + x^5 + x^4 + 1
            } else {
                crc >>= 1;
            }
        }
        self.0 = crc;
    }
}

mod test {
    #[test]
    fn test_crc_update() {
        use super::OneWireCrc;
        #[cfg(test)]
        extern crate std;
        use rand::prelude::*;
        let mut rng = rand::rng();
        let buf = (0..100)
            .map(|_| rng.random::<u8>())
            .collect::<std::vec::Vec<u8>>();
        let mut crc = OneWireCrc::default();
        for &byte in buf.iter() {
            crc.update(byte);
        }
        let table = crc.value();
        std::println!("CRC after calc: {table:#04x}");
        let mut crc = OneWireCrc::default();
        for &byte in buf.iter() {
            crc.update_table(byte);
        }
        let calc = crc.value();
        std::println!("CRC after table: {calc:#04x}");
        assert_eq!(table, calc, "CRC values do not match");
    }
}
