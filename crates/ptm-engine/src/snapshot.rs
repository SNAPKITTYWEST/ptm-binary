// Snapshot: PTM configuration snapshot for debugging and audit.
// Uses magic 0x534E4150 ('SNAP'). 16-byte standard header + 32 bytes metadata.

use ptm_core::header::FileHeader;
use ptm_core::magic::Magic;

pub const SNAP_HEADER_SIZE: usize = 16 + 32;

#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    pub step_number: u32,
    pub current_state: u32,
    pub input_head: u32,
    pub work_head: u32,
    pub control_head: u32,
    pub output_head: u32,
    pub work_tape_len: u32,
    pub control_tape_len: u32,
}

impl SnapshotMetadata {
    pub fn encode(&self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[0..4].copy_from_slice(&self.step_number.to_be_bytes());
        buf[4..8].copy_from_slice(&self.current_state.to_be_bytes());
        buf[8..12].copy_from_slice(&self.input_head.to_be_bytes());
        buf[12..16].copy_from_slice(&self.work_head.to_be_bytes());
        buf[16..20].copy_from_slice(&self.control_head.to_be_bytes());
        buf[20..24].copy_from_slice(&self.output_head.to_be_bytes());
        buf[24..28].copy_from_slice(&self.work_tape_len.to_be_bytes());
        buf[28..32].copy_from_slice(&self.control_tape_len.to_be_bytes());
        buf
    }

    pub fn decode(buf: &[u8; 32]) -> Self {
        SnapshotMetadata {
            step_number: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            current_state: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            input_head: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            work_head: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            control_head: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            output_head: u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]),
            work_tape_len: u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]),
            control_tape_len: u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub header: FileHeader,
    pub metadata: SnapshotMetadata,
}

impl Snapshot {
    pub fn new(metadata: SnapshotMetadata) -> Self {
        let payload_len = 32;
        let header = FileHeader::new(Magic::SNAP, payload_len as u32);
        Snapshot { header, metadata }
    }

    pub fn encode(&self) -> Vec<u8> {
        let payload_len = 32;
        let header = FileHeader::new(Magic::SNAP, payload_len as u32);
        let mut buf = Vec::with_capacity(16 + payload_len);
        buf.extend_from_slice(&header.encode());
        buf.extend_from_slice(&self.metadata.encode());
        buf
    }

    pub fn decode(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 16 + 32 {
            return Err("buffer too short".into());
        }
        let mut hdr = [0u8; 16];
        hdr.copy_from_slice(&buf[0..16]);
        let header = FileHeader::decode(&hdr).map_err(|e| e.to_string())?;
        let mut meta_buf = [0u8; 32];
        meta_buf.copy_from_slice(&buf[16..48]);
        let metadata = SnapshotMetadata::decode(&meta_buf);
        Ok(Snapshot { header, metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_roundtrip() {
        let meta = SnapshotMetadata {
            step_number: 42,
            current_state: 1,
            input_head: 3,
            work_head: 0,
            control_head: 0,
            output_head: 0,
            work_tape_len: 10,
            control_tape_len: 5,
        };
        let snap = Snapshot::new(meta);
        let buf = snap.encode();
        let snap2 = Snapshot::decode(&buf).unwrap();
        assert_eq!(snap2.metadata.step_number, 42);
        assert_eq!(snap2.metadata.current_state, 1);
        assert_eq!(snap2.metadata.input_head, 3);
    }
}
