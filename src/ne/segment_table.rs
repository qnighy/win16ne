use std::convert::TryInto;
use std::io::{self, Read, Seek, SeekFrom};
///
/// This table contains one 8-byte record for every code and data segment
/// in the program or library module. 
/// 
/// Each segment has an #ordinal number
/// associated with it. For example, the first segment has an ordinal
/// number of 1. 
/// These segment numbers are used to reference the segments
/// in other sections of the New Executable file. 
/// (Offsets are from the beginning of the record.)
/// 

/// The New Executable segment table entry.
#[derive(Debug, Clone)]
pub struct NeSegment {
    pub header: NeSegmentHeader,
    pub shift_count: u16,
    pub data: Option<Vec<u8>>,
}

impl NeSegment {
    pub fn read<R: Read>(r: &mut R, shift_count: u16) -> io::Result<Self> {
        Ok(Self {
            header: NeSegmentHeader::read(r)?,
            shift_count,
            data: None,
        })
    }

    pub fn read_data<R: Read + Seek>(&mut self, r: &mut R) -> io::Result<()> {
        if self.header.data_offset_shifted == 0 {
            return Ok(());
        }
        let data_offset = self.data_offset();
        let data_length = self.data_length();
        r.seek(SeekFrom::Start(data_offset))?;
        let mut data = vec![0; data_length as usize];
        r.read_exact(&mut data)?;
        self.data = Some(data);
        Ok(())
    }

    pub fn data_offset(&self) -> u64 {
        (self.header.data_offset_shifted as u64) << self.shift_count
    }

    pub fn data_length(&self) -> u64 {
        if self.header.data_length == 0 {
            0x10000
        } else {
            self.header.data_length as u64
        }
    }

    pub fn min_alloc(&self) -> u64 {
        if self.header.min_alloc == 0 {
            0x10000
        } else {
            self.header.min_alloc as u64
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NeSegmentHeader {
    pub data_offset_shifted: u16,
    pub data_length: u16,
    pub flags: u16,
    pub min_alloc: u16,
}
impl NeSegmentHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x8];
        r.read_exact(&mut buf)?;
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap());

        Ok(Self {
            data_offset_shifted: get_u16(0),
            data_length: get_u16(2),
            flags: get_u16(4),
            min_alloc: get_u16(6),
        })
    }
}
