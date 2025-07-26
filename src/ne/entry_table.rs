use std::convert::TryInto;
use std::io::{self, Read};


///
/// This table contains one member for every entry point in the program (EXE/DRV/SYS) or
/// library module (DLL). 
/// (Every public FAR function or procedure in a module is
/// an entry point.) 
/// 
/// The members in the entry table have ordinal numbers
/// beginning at 1. 
/// These ordinal numbers are referenced by the resident
/// names table and the nonresident names table.
/// 
/// LINK versions 4.0 and later bundle the members of the entry table.
/// Each bundle begins with the following information. (Offsets are from
/// the beginning of the bundle.)
/// 
#[derive(Debug, Clone)]
pub struct EntryTable {
    pub entries: Vec<SegmentEntry>,
}

impl EntryTable {
    ///
    /// Reads EntryTable bundles (`enttab`)
    /// EntryTable contains bundles /groups of exporting procedure addresses/
    /// Entries in one bundle determines as entries of single type.
    /// If entry bundle has flag ENTTAB_BUNDLE_EXPORT => all entries in bundle belongs export.
    ///   
    /// stopped: always length=2.
    /// 
    /// \param r -- reader reference
    /// \param length -- `cbenttab` header field
    pub fn read<R: Read>(r: &mut R, mut length: u16) -> io::Result<Self> {
        let mut entries = Vec::new();
        while length > 0 {
            let num = {
                let mut buf = [0];
                r.read_exact(&mut buf)?;
                buf[0]
            };
            if num == 0 {
                if length != 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Inexact length for entry table: length={}", length) // stopped here. but sunflower not
                    ));
                }
                break;
            }
            let segment = {
                let mut buf = [0];
                r.read_exact(&mut buf)?;
                buf[0]
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
                    format!("Inexact length for entry table: \r\n\tbundle_size={}\r\n\tenttab_length={}", bundle_size, length),
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
    ///
    /// Attempts to rewrite my logic. 
    /// Algorithm mostly bases on Microsoft NE segmentation format.pdf
    /// 
    /// \param r -- binary reader instance
    /// \param cb_ent_tab -- bundles count in EntryTable /see NE Header/
    /// 
    pub fn read_sf<R: Read>(r: &mut R, cb_ent_tab: u16) -> io::Result<Self> {
        let mut entries: Vec<SegmentEntry> = Vec::new();
        let mut bytes_remaining = cb_ent_tab;
        let mut _ordinal: u16 = 1; // entry index means ordinal in non/resident names tables

        while bytes_remaining > 0 {
            // Read bundle header
            let mut buffer = [0; 2];
            r.read_exact(&mut buffer)?;
            bytes_remaining -= 2;

            let entries_count = buffer[0];
            let seg_id = buffer[1];

            if entries_count == 0 {
                // End of table marker
                break;
            }

            if seg_id == 0 {
                // Unused entries (padding between actual entries)
                for _ in 0..entries_count {
                    entries.push(SegmentEntry::Unused);
                    _ordinal += 1;
                }
                continue;
            }

            // Calculate bundle size based on segment type
            let entry_size = if seg_id == 0xFF { 6 } else { 3 };
            let bundle_size = (entries_count as u16) * entry_size;
            
            if bundle_size > bytes_remaining {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Bundle size exceeds remaining bytes: bundle_size={}, remaining={}", 
                            bundle_size, bytes_remaining),
                ));
            }
            bytes_remaining -= bundle_size;

            // Process each entry in the bundle
            for _ in 0..entries_count {
                let entry = if seg_id == 0xFF {
                    // Movable segment entry (6 bytes)
                    SegmentEntry::Moveable(MoveableSegmentEntry::read(r)?)
                } else {
                    // Fixed segment entry (3 bytes)
                    SegmentEntry::Fixed(FixedSegmentEntry::read(r, seg_id)?)
                };
                entries.push(entry);
                _ordinal += 1;
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
