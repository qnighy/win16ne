use std::convert::TryInto;
use std::io::{self, Read};

/// The DOS header.
#[derive(Debug, Clone, Copy)]
pub struct DosHeader {
    /// MZ Header signature
    pub magic: u16,
    /// Bytes on last page of file
    pub cblp: u16,
    /// Pages in file
    pub cp: u16,
    /// Relocations
    pub crlc: u16,
    /// Size of header in paragraphs
    pub cparhdr: u16,
    /// Minimum extra paragraphs needed
    pub minalloc: u16,
    /// Maximum extra paragraphs needed
    pub maxalloc: u16,
    /// Initial (relative) SS value
    pub ss: u16,
    /// Initial SP value
    pub sp: u16,
    /// Checksum
    pub csum: u16,
    /// Initial IP value
    pub ip: u16,
    /// Initial (relative) CS value
    pub cs: u16,
    /// File address of relocation table
    pub lfarlc: u16,
    /// Overlay number
    pub ovno: u16,
    /// Reserved words
    pub res: [u16; 4],
    /// OEM identifier (for e_oeminfo)
    pub oemid: u16,
    /// OEM information; e_oemid specific
    pub oeminfo: u16,
    /// Reserved words
    pub res2: [u16; 10],
    /// Offset to extended header
    pub lfanew: u32,
}

impl DosHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap());
        let get_u32 = |pos| u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap());

        Ok(Self {
            magic: get_u16(0),
            cblp: get_u16(2),
            cp: get_u16(4),
            crlc: get_u16(6),
            cparhdr: get_u16(8),
            minalloc: get_u16(0xA),
            maxalloc: get_u16(0xC),
            ss: get_u16(0xE),
            sp: get_u16(0x10),
            csum: get_u16(0x12),
            ip: get_u16(0x14),
            cs: get_u16(0x16),
            lfarlc: get_u16(0x18),
            ovno: get_u16(0x1A),
            res: [get_u16(0x1C), get_u16(0x1E), get_u16(0x20), get_u16(0x22)],
            oemid: get_u16(0x24),
            oeminfo: get_u16(0x26),
            res2: [
                get_u16(0x28),
                get_u16(0x2A),
                get_u16(0x2C),
                get_u16(0x2E),
                get_u16(0x30),
                get_u16(0x32),
                get_u16(0x34),
                get_u16(0x36),
                get_u16(0x38),
                get_u16(0x3A),
            ],
            lfanew: get_u32(0x3C),
        })
    }

    pub fn check_magic(&self) -> io::Result<()> {
        // 4D 5A == b"MZ"
        if self.magic != 0x5A4D {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid magic"));
        }
        Ok(())
    }

    pub fn check_sum(buf: &[u8]) -> io::Result<()> {
        let mut sum = 0_u16;
        let mut pos = 0;
        while pos < buf.len() {
            let word = [buf[pos], *buf.get(pos + 1).unwrap_or(&0)];
            let word = u16::from_le_bytes(word);
            sum = sum.wrapping_add(word);
            pos += 2;
        }
        if sum != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid checksum: 0x{:04x}", sum),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DosHeader2 {
        /// MZ Header signature
        pub magic: u16,
        /// Bytes on last page of file
        pub cblp: u16,
        /// Pages in file
        pub cp: u16,
        /// Relocations
        pub crlc: u16,
        /// Size of header in paragraphs
        pub cparhdr: u16,
        /// Minimum extra paragraphs needed
        pub minalloc: u16,
        /// Maximum extra paragraphs needed
        pub maxalloc: u16,
        /// Initial (relative) SS value
        pub ss: u16,
        /// Initial SP value
        pub sp: u16,
        /// Checksum
        pub csum: u16,
        /// Initial IP value
        pub ip: u16,
        /// Initial (relative) CS value
        pub cs: u16,
        /// File address of relocation table
        pub lfarlc: u16,
        /// Overlay number
        pub ovno: u16,
        /// Reserved words
        pub res: [u16; 4],
        /// OEM identifier (for e_oeminfo)
        pub oemid: u16,
        /// OEM information; e_oemid specific
        pub oeminfo: u16,
        /// Reserved words
        pub res2: [u16; 10],
        /// Offset to extended header
        pub lfanew: u32,
    }

    impl From<DosHeader> for DosHeader2 {
        fn from(h: DosHeader) -> Self {
            DosHeader2 {
                magic: h.magic,
                cblp: h.cblp,
                cp: h.cp,
                crlc: h.crlc,
                cparhdr: h.cparhdr,
                minalloc: h.minalloc,
                maxalloc: h.maxalloc,
                ss: h.ss,
                sp: h.sp,
                csum: h.csum,
                ip: h.ip,
                cs: h.cs,
                lfarlc: h.lfarlc,
                ovno: h.ovno,
                res: h.res,
                oemid: h.oemid,
                oeminfo: h.oeminfo,
                res2: h.res2,
                lfanew: h.lfanew,
            }
        }
    }

    #[test]
    fn test_dos_header_size() {
        assert_eq!(std::mem::size_of::<DosHeader>(), 0x40);
    }

    #[test]
    fn test_dos_header() {
        let buf: [u8; 0x40] = *b"\
            MZ\xD4\x01\x06\x00\x00\x00\x20\x00\x00\x00\xFF\xFF\x00\x00\
            \x00\x00\x3D\x98\x00\x00\x00\x00\x40\x00\x00\x00\x01\x00\x00\x00\
            \x00\x00\x00\x00\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
            \x00\x00\x00\x00\x00\x10\x00\x00\x00\x00\x00\x00\x00\x06\x00\x00\
        ";
        let h = DosHeader::read(&mut Cursor::new(buf)).unwrap();
        assert_eq!(
            DosHeader2::from(h),
            DosHeader2 {
                magic: 0x5A4D,
                cblp: 0x01D4,
                cp: 0x0006,
                crlc: 0x0000,
                cparhdr: 0x0020,
                minalloc: 0x0000,
                maxalloc: 0xFFFF,
                ss: 0x0000,
                sp: 0x0000,
                csum: 0x983D,
                ip: 0x0000,
                cs: 0x0000,
                lfarlc: 0x0040,
                ovno: 0x0000,
                res: [0x0001, 0x0000, 0x0000, 0x0000],
                oemid: 0x1000,
                oeminfo: 0x0000,
                res2: [
                    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x1000, 0x0000, 0x0000, 0x0000,
                ],
                lfanew: 0x0600,
            }
        );
    }
}
