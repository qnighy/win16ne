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
