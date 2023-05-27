use std::io::{self, Read};

use bytemuck::{Pod, Zeroable};

use crate::util::endian::{Lu16, Lu32};

/// The DOS header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct DosHeader {
    /// MZ Header signature
    pub magic: Lu16,
    /// Bytes on last page of file
    pub cblp: Lu16,
    /// Pages in file
    pub cp: Lu16,
    /// Relocations
    pub crlc: Lu16,
    /// Size of header in paragraphs
    pub cparhdr: Lu16,
    /// Minimum extra paragraphs needed
    pub minalloc: Lu16,
    /// Maximum extra paragraphs needed
    pub maxalloc: Lu16,
    /// Initial (relative) SS value
    pub ss: Lu16,
    /// Initial SP value
    pub sp: Lu16,
    /// Checksum
    pub csum: Lu16,
    /// Initial IP value
    pub ip: Lu16,
    /// Initial (relative) CS value
    pub cs: Lu16,
    /// File address of relocation table
    pub lfarlc: Lu16,
    /// Overlay number
    pub ovno: Lu16,
    /// Reserved words
    pub res: [Lu16; 4],
    /// OEM identifier (for e_oeminfo)
    pub oemid: Lu16,
    /// OEM information; e_oemid specific
    pub oeminfo: Lu16,
    /// Reserved words
    pub res2: [Lu16; 10],
    /// Offset to extended header
    pub lfanew: Lu32,
}

impl DosHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;
        Ok(bytemuck::cast(buf))
    }

    pub fn check_magic(&self) -> io::Result<()> {
        // 4D 5A == b"MZ"
        if self.magic.value() != 0x5A4D {
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
                magic: h.magic.value(),
                cblp: h.cblp.value(),
                cp: h.cp.value(),
                crlc: h.crlc.value(),
                cparhdr: h.cparhdr.value(),
                minalloc: h.minalloc.value(),
                maxalloc: h.maxalloc.value(),
                ss: h.ss.value(),
                sp: h.sp.value(),
                csum: h.csum.value(),
                ip: h.ip.value(),
                cs: h.cs.value(),
                lfarlc: h.lfarlc.value(),
                ovno: h.ovno.value(),
                res: [
                    h.res[0].value(),
                    h.res[1].value(),
                    h.res[2].value(),
                    h.res[3].value(),
                ],
                oemid: h.oemid.value(),
                oeminfo: h.oeminfo.value(),
                res2: [
                    h.res2[0].value(),
                    h.res2[1].value(),
                    h.res2[2].value(),
                    h.res2[3].value(),
                    h.res2[4].value(),
                    h.res2[5].value(),
                    h.res2[6].value(),
                    h.res2[7].value(),
                    h.res2[8].value(),
                    h.res2[9].value(),
                ],
                lfanew: h.lfanew.value(),
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
