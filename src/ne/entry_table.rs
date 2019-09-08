use std::convert::TryInto;
use std::io::{self, Read};

#[derive(Debug, Clone)]
pub struct EntryTable {
    pub entries: Vec<SegmentEntry>,
}

impl EntryTable {
    pub fn read<R: Read>(r: &mut R, mut length: u16) -> io::Result<Self> {
        let mut entries = Vec::new();
        while length > 0 {
            let [num, segment] = {
                let mut buf = [0; 2];
                r.read_exact(&mut buf)?;
                buf
            };
            let bundle_size = if segment == 0 {
                0
            } else if segment < 0xFF {
                3
            } else {
                6
            } * num as u16
                + 2;
            if bundle_size > length {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Inexact length for entry table",
                ));
            }
            length -= bundle_size;
            for _ in 0..num {
                entries.push(if segment == 0 {
                    SegmentEntry::Unused
                } else if segment < 0xFF {
                    SegmentEntry::Fixed(FixedSegmentEntry::read(r, segment)?)
                } else {
                    SegmentEntry::Moveable(MoveableSegmentEntry::read(r)?)
                });
            }
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone)]
pub enum SegmentEntry {
    Unused,
    Fixed(FixedSegmentEntry),
    Moveable(MoveableSegmentEntry),
}

#[derive(Debug, Clone, Copy)]
pub struct FixedSegmentEntry {
    pub segment: u8,
    pub flags: u8,
    pub offset: u16,
}

impl FixedSegmentEntry {
    pub fn read<R: Read>(r: &mut R, segment: u8) -> io::Result<Self> {
        let mut buf = [0; 3];
        r.read_exact(&mut buf)?;
        Ok(Self {
            segment,
            flags: buf[0],
            offset: u16::from_le_bytes(buf[1..3].try_into().unwrap()),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveableSegmentEntry {
    pub flags: u8,
    pub magic: [u8; 2],
    pub segment: u8,
    pub offset: u16,
}

impl MoveableSegmentEntry {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 6];
        r.read_exact(&mut buf)?;
        Ok(Self {
            flags: buf[0],
            magic: [buf[1], buf[2]],
            segment: buf[3],
            offset: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
        })
    }
}
