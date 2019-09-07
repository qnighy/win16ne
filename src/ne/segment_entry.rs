use std::convert::TryInto;
use std::io::{self, Read};

use crate::ne::header::NeHeader;

/// The New Executable segment table entry.
#[derive(Debug, Clone, Copy)]
pub struct NeSegment {
    pub data_offset: u16,
    pub data_length: u16,
    pub flags: u16,
    pub min_alloc: u16,
}
impl NeSegment {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x8];
        r.read_exact(&mut buf)?;
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap());

        Ok(Self {
            data_offset: get_u16(0),
            data_length: get_u16(2),
            flags: get_u16(4),
            min_alloc: get_u16(6),
        })
    }

    pub fn data_offset(&self, header: &NeHeader) -> u64 {
        (self.data_offset as u64) << header.file_alignment_shift_count
    }

    pub fn data_length(&self) -> u64 {
        if self.data_length == 0 {
            0x10000
        } else {
            self.data_length as u64
        }
    }

    pub fn min_alloc(&self) -> u64 {
        if self.min_alloc == 0 {
            0x10000
        } else {
            self.min_alloc as u64
        }
    }
}
