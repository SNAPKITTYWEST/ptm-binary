// File header: 16 bytes fixed for all artifact types.
// Bytes 0-3: magic, 4-5: version, 6-7: flags, 8-11: payload length, 12-15: CRC-32.

use crate::crc32::crc32;
use crate::magic::Magic;

pub const HEADER_SIZE: usize = 16;
pub const CURRENT_VERSION: u16 = 0x0001;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags: u16 {
        const ENCRYPTED  = 0x8000;
        const COMPRESSED = 0x4000;
        const PLACEHOLDER= 0x2000;
    }
}

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub magic: Magic,
    pub version: u16,
    pub flags: Flags,
    pub payload_len: u32,
}

impl FileHeader {
    pub fn new(magic: Magic, payload_len: u32) -> Self {
        FileHeader {
            magic,
            version: CURRENT_VERSION,
            flags: Flags::empty(),
            payload_len,
        }
    }

    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic.as_bytes());
        buf[4..6].copy_from_slice(&self.version.to_be_bytes());
        buf[6..8].copy_from_slice(&self.flags.bits().to_be_bytes());
        buf[8..12].copy_from_slice(&self.payload_len.to_be_bytes());
        let checksum = crc32(&buf[0..12]);
        buf[12..16].copy_from_slice(&checksum.to_be_bytes());
        buf
    }

    pub fn decode(buf: &[u8; HEADER_SIZE]) -> Result<Self, HeaderError> {
        let magic_val = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let magic = Magic::from_u32(magic_val).ok_or(HeaderError::InvalidMagic(magic_val))?;
        let version = u16::from_be_bytes([buf[4], buf[5]]);
        let flag_bits = u16::from_be_bytes([buf[6], buf[7]]);
        let flags = Flags::from_bits(flag_bits).ok_or(HeaderError::InvalidFlags(flag_bits))?;
        let payload_len = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let stored_crc = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let computed_crc = crc32(&buf[0..12]);
        if stored_crc != computed_crc {
            return Err(HeaderError::ChecksumMismatch { stored: stored_crc, computed: computed_crc });
        }
        Ok(FileHeader { magic, version, flags, payload_len })
    }

    pub fn total_size(&self) -> usize {
        HEADER_SIZE + self.payload_len as usize
    }
}

#[derive(Debug)]
pub enum HeaderError {
    InvalidMagic(u32),
    InvalidFlags(u16),
    ChecksumMismatch { stored: u32, computed: u32 },
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidMagic(v) => write!(f, "invalid magic: 0x{:08X}", v),
            Self::InvalidFlags(v) => write!(f, "invalid flags: 0x{:04X}", v),
            Self::ChecksumMismatch { stored, computed } =>
                write!(f, "CRC-32 mismatch: stored 0x{:08X}, computed 0x{:08X}", stored, computed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let h = FileHeader::new(Magic::PTMI, 1024);
        let buf = h.encode();
        let h2 = FileHeader::decode(&buf).unwrap();
        assert_eq!(h2.magic, Magic::PTMI);
        assert_eq!(h2.payload_len, 1024);
    }
}
