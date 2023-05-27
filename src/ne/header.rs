use std::convert::TryInto;
use std::io::{self, Read};

/// The New Executable header.
#[derive(Debug, Clone, Copy)]
pub struct NeHeader {
    pub magic: [u8; 2],
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub entry_table_offset: u16,
    pub entry_table_length: u16,
    pub file_load_crc: u32,
    pub flags: u16,
    pub auto_data_segment_index: u16,
    pub init_heap_size: u16,
    pub init_stack_size: u16,
    pub entry_point: u32,
    pub init_stack: u32,
    pub segment_count: u16,
    pub module_references: u16,
    pub non_resident_names_size: u16,
    pub segment_table_offset: u16,
    pub resource_table_offset: u16,
    pub resident_names_table_offset: u16,
    pub module_reference_table_offset: u16,
    pub import_name_table_offset: u16,
    pub non_resident_names_table_offset: u32,
    pub movable_entry_point_count: u16,
    pub file_alignment_shift_count: u16,
    pub resource_table_entries: u16,
    pub target_os: u8,
    pub os2_exe_flags: u8,
    pub return_thunk_offset: u16,
    pub segment_reference_thunk_offset: u16,
    pub min_code_swap: u16,
    pub expected_win_ver: [u8; 2],
}

impl NeHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;
        let get_u8 = |pos| buf[pos];
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap());
        let get_u32 = |pos| u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap());

        Ok(Self {
            magic: [get_u8(0), get_u8(1)],
            major_linker_version: get_u8(2),
            minor_linker_version: get_u8(3),
            entry_table_offset: get_u16(4),
            entry_table_length: get_u16(6),
            file_load_crc: get_u32(8),
            flags: get_u16(0xC),
            auto_data_segment_index: get_u16(0xE),
            init_heap_size: get_u16(0x10),
            init_stack_size: get_u16(0x12),
            entry_point: get_u32(0x14),
            init_stack: get_u32(0x18),
            segment_count: get_u16(0x1C),
            module_references: get_u16(0x1E),
            non_resident_names_size: get_u16(0x20),
            segment_table_offset: get_u16(0x22),
            resource_table_offset: get_u16(0x24),
            resident_names_table_offset: get_u16(0x26),
            module_reference_table_offset: get_u16(0x28),
            import_name_table_offset: get_u16(0x2A),
            non_resident_names_table_offset: get_u32(0x2C),
            movable_entry_point_count: get_u16(0x30),
            file_alignment_shift_count: get_u16(0x32),
            resource_table_entries: get_u16(0x34),
            target_os: get_u8(0x36),
            os2_exe_flags: get_u8(0x37),
            return_thunk_offset: get_u16(0x38),
            segment_reference_thunk_offset: get_u16(0x3A),
            min_code_swap: get_u16(0x3C),
            expected_win_ver: [get_u8(0x3E), get_u8(0x3F)],
        })
    }

    pub fn check_magic(&self) -> io::Result<()> {
        if self.magic != *b"NE" {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid magic"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct NeHeader2 {
        pub magic: [u8; 2],
        pub major_linker_version: u8,
        pub minor_linker_version: u8,
        pub entry_table_offset: u16,
        pub entry_table_length: u16,
        pub file_load_crc: u32,
        pub flags: u16,
        pub auto_data_segment_index: u16,
        pub init_heap_size: u16,
        pub init_stack_size: u16,
        pub entry_point: u32,
        pub init_stack: u32,
        pub segment_count: u16,
        pub module_references: u16,
        pub non_resident_names_size: u16,
        pub segment_table_offset: u16,
        pub resource_table_offset: u16,
        pub resident_names_table_offset: u16,
        pub module_reference_table_offset: u16,
        pub import_name_table_offset: u16,
        pub non_resident_names_table_offset: u32,
        pub movable_entry_point_count: u16,
        pub file_alignment_shift_count: u16,
        pub resource_table_entries: u16,
        pub target_os: u8,
        pub os2_exe_flags: u8,
        pub return_thunk_offset: u16,
        pub segment_reference_thunk_offset: u16,
        pub min_code_swap: u16,
        pub expected_win_ver: [u8; 2],
    }

    impl From<NeHeader> for NeHeader2 {
        fn from(h: NeHeader) -> Self {
            Self {
                magic: h.magic,
                major_linker_version: h.major_linker_version,
                minor_linker_version: h.minor_linker_version,
                entry_table_offset: h.entry_table_offset,
                entry_table_length: h.entry_table_length,
                file_load_crc: h.file_load_crc,
                flags: h.flags,
                auto_data_segment_index: h.auto_data_segment_index,
                init_heap_size: h.init_heap_size,
                init_stack_size: h.init_stack_size,
                entry_point: h.entry_point,
                init_stack: h.init_stack,
                segment_count: h.segment_count,
                module_references: h.module_references,
                non_resident_names_size: h.non_resident_names_size,
                segment_table_offset: h.segment_table_offset,
                resource_table_offset: h.resource_table_offset,
                resident_names_table_offset: h.resident_names_table_offset,
                module_reference_table_offset: h.module_reference_table_offset,
                import_name_table_offset: h.import_name_table_offset,
                non_resident_names_table_offset: h.non_resident_names_table_offset,
                movable_entry_point_count: h.movable_entry_point_count,
                file_alignment_shift_count: h.file_alignment_shift_count,
                resource_table_entries: h.resource_table_entries,
                target_os: h.target_os,
                os2_exe_flags: h.os2_exe_flags,
                return_thunk_offset: h.return_thunk_offset,
                segment_reference_thunk_offset: h.segment_reference_thunk_offset,
                min_code_swap: h.min_code_swap,
                expected_win_ver: h.expected_win_ver,
            }
        }
    }

    #[test]
    fn test_ne_header_size() {
        assert_eq!(std::mem::size_of::<NeHeader>(), 0x40);
    }

    #[test]
    fn test_ne_header() {
        let buf: [u8; 0x40] = *b"\
            NE\x05\x0A\x6C\x01\x02\x00\x46\x45\x52\x47\x12\x03\x02\x00\
            \x00\x10\x00\x50\x10\x00\x01\x00\x00\x00\x02\x00\x09\x00\x01\x00\
            \x1C\x00\x40\x00\x90\x00\x54\x01\x60\x01\x62\x01\x6E\x07\x00\x00\
            \x00\x00\x08\x00\xFF\xFF\x02\x08\x00\x00\x00\x00\x00\x00\x00\x03\
        ";
        let h = NeHeader::read(&mut Cursor::new(buf)).unwrap();
        assert_eq!(
            NeHeader2::from(h),
            NeHeader2 {
                magic: *b"NE",
                major_linker_version: 5,
                minor_linker_version: 10,
                entry_table_offset: 0x016C,
                entry_table_length: 0x0002,
                file_load_crc: 0x47524546,
                flags: 0x0312,
                auto_data_segment_index: 0x0002,
                init_heap_size: 0x1000,
                init_stack_size: 0x5000,
                entry_point: 0x00010010,
                init_stack: 0x00020000,
                segment_count: 0x0009,
                module_references: 0x0001,
                non_resident_names_size: 0x001C,
                segment_table_offset: 0x0040,
                resource_table_offset: 0x0090,
                resident_names_table_offset: 0x0154,
                module_reference_table_offset: 0x0160,
                import_name_table_offset: 0x0162,
                non_resident_names_table_offset: 0x076E,
                movable_entry_point_count: 0x0000,
                file_alignment_shift_count: 0x0008,
                resource_table_entries: 0xFFFF,
                target_os: 0x02,
                os2_exe_flags: 0x08,
                return_thunk_offset: 0x0000,
                segment_reference_thunk_offset: 0x0000,
                min_code_swap: 0x0000,
                expected_win_ver: [0x00, 0x03],
            }
        );
    }
}
