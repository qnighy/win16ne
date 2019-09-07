use std::convert::TryInto;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct EntryTable {
    pub bundles: Vec<EntryBundle>,
}

impl EntryTable {
    pub fn read<R: Read + Seek>(r: &mut R, offset: u64, length: u16) -> io::Result<Self> {
        let end = offset + length as u64;
        let mut pos = r.seek(SeekFrom::Start(offset))?;
        let mut bundles = Vec::new();
        while pos < end {
            bundles.push(EntryBundle::read(r)?);
            pos = r.seek(SeekFrom::Current(0))?;
        }
        Ok(Self { bundles })
    }
}

#[derive(Debug, Clone)]
pub enum EntryBundle {
    Unused,
    Fixed(FixedEntryBundle),
    Moveable(MoveableEntryBundle),
}

impl EntryBundle {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 2];
        r.read_exact(&mut buf)?;
        let [num, segment] = buf;
        Ok(if segment == 0 {
            EntryBundle::Unused
        } else if segment < 0xFF {
            EntryBundle::Fixed(FixedEntryBundle::read(r, num, segment)?)
        } else {
            EntryBundle::Moveable(MoveableEntryBundle::read(r, num)?)
        })
    }
}

#[derive(Debug, Clone)]
pub struct FixedEntryBundle {
    pub segment: u8,
    pub entries: Vec<FixedSegmentEntry>,
}

impl FixedEntryBundle {
    pub fn read<R: Read>(r: &mut R, num: u8, segment: u8) -> io::Result<Self> {
        let entries = (0..num)
            .map(|_| FixedSegmentEntry::read(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { segment, entries })
    }
}

#[derive(Debug, Clone)]
pub struct MoveableEntryBundle {
    pub entries: Vec<MoveableSegmentEntry>,
}

impl MoveableEntryBundle {
    pub fn read<R: Read>(r: &mut R, num: u8) -> io::Result<Self> {
        let entries = (0..num)
            .map(|_| MoveableSegmentEntry::read(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedSegmentEntry {
    pub flags: u8,
    pub offset: u16,
}

impl FixedSegmentEntry {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 3];
        r.read_exact(&mut buf)?;
        Ok(Self {
            flags: buf[0],
            offset: u16::from_le_bytes(buf[1..3].try_into().unwrap()),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveableSegmentEntry {
    pub flags: u8,
    pub unknown: u8,
    pub magic: u8,
    pub segment: u8,
    pub offset: u16,
}

impl MoveableSegmentEntry {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 6];
        r.read_exact(&mut buf)?;
        Ok(Self {
            flags: buf[0],
            unknown: buf[1],
            magic: buf[2],
            segment: buf[3],
            offset: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
        })
    }
}
