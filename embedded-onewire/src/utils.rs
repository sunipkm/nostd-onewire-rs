#[derive(Debug, Default)]
/// Calculate CRC-8 used in 1-Wire communications.
pub struct OneWireCrc(u8);

impl OneWireCrc {
    /// Get the current CRC value
    pub fn value(&self) -> u8 {
        self.0
    }
    
    /// Update the CRC with the incoming byte.
    pub fn update(&mut self, byte: u8) {
        let mut crc = self.0 ^ byte; // XOR the byte with the current CRC value
        for _ in 0..8 {
            if crc & 0x1 == 0x1 {
                crc = (crc >> 1) ^ 0x8c; // Polynomial for CRC-8
            } else {
                crc >>= 1;
            }
        }
        self.0 = crc;
    }

    /// Valudate a sequence of bytes where the last byte is the 1-Wire CRC of
    /// the previous bytes. 
    pub fn validate(sequence: &[u8]) -> bool {
        let mut crc = OneWireCrc(0);
        for &byte in sequence.iter() {
            crc.update(byte); // Update CRC with the all bytes of the ROM
        }
        crc.0 == 0x0 // If the last byte of the ROM is the CRC, the result should be 0
    }
}