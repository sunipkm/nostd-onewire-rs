#[derive(Debug, Default)]
pub(crate) struct OneWireCrc(u8);

impl OneWireCrc {
    pub(crate) fn update(&mut self, byte: u8) {
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

    pub(crate) fn validate(&self, rom: &[u8; 8]) -> bool {
        let mut crc = OneWireCrc(0);
        for &byte in rom.iter() {
            crc.update(byte); // Update CRC with the all bytes of the ROM
        }
        crc.0 == 0x0 // If the last byte of the ROM is the CRC, the result should be 0
    }
}