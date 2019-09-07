use std::io::{self, Read, BufReader, Cursor, Seek, SeekFrom};
use std::fs::File;
use std::convert::TryInto;

use log::debug;

pub mod mz;

use mz::DosHeader;

fn main() -> io::Result<()> {
    env_logger::init();

    let data = {
        let mut f = BufReader::new(File::open("a.exe")?);
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        data
    };

    let mut cursor = Cursor::new(data.as_slice());

    let dos_header = DosHeader::read(&mut cursor)?;
    debug!("dos_header = {:?}", dos_header);
    dos_header.check_magic()?;
    // DosHeader::check_sum(&data)?;

    cursor.seek(SeekFrom::Start(dos_header.lfanew as u64))?;
    let ne_header = NeHeader::read(&mut cursor)?;
    debug!("ne_header = {:#?}", ne_header);
    ne_header.check_magic()?;
    println!("File Type: Windows New Executable");
    println!("Header:");
    println!("    Linker version: {}.{}", ne_header.major_linker_version, ne_header.minor_linker_version);
    print!("    Flags: ");
    {
        let mut flag_found = false;
        for shift in 0..16 {
            let mask = 1 << shift;
            if (mask & ne_header.flags) == 0 {
                continue;
            }
            if flag_found {
                print!(" | ");
            }
            flag_found = true;
            if mask == 0x1 {
                print!("SINGLEDATA");
            } else if mask == 0x2 {
                print!("MULTIPLEDATA");
            } else if mask == 0x2000 {
                print!("LINK_ERROR");
            } else if mask == 0x8000 {
                print!("LIBRARY");
            } else {
                print!("0x{:04x}", mask);
            }
        }
        if !flag_found {
            print!("0");
        }
    }
    println!();
    println!("    Auto-data segment: {}", ne_header.auto_data_segment_index);
    println!("    Initial heap size: {}", ne_header.init_heap_size);
    println!("    Initial stack size: {}", ne_header.init_stack_size);
    println!("    Entry point (CS:IP): {:04X}:{:04X}", ne_header.entry_point >> 16, ne_header.entry_point & 0xFFFF);
    println!("    Initial stack (SS:SP): {:04X}:{:04X}", ne_header.init_stack >> 16, ne_header.init_stack & 0xFFFF);
    println!("    Number of segments: {}", ne_header.segment_count);
    println!("    Number of referenced modules: {}", ne_header.module_references);
    println!("    Number of movable entry points: {}", ne_header.movable_entry_point_count);
    println!("    Number of file alignment shifts: {}", ne_header.file_alignment_shift_count);
    println!("    Number of resource table entries: {}", ne_header.resource_table_entries);
    print!("    Target os: ");
    if ne_header.target_os == 2 {
        print!("Windows");
    } else {
        print!("Unknown ({})", ne_header.target_os);
    }
    println!();
    println!("    Expected Windows version: {}.{}", ne_header.expected_win_ver[1], ne_header.expected_win_ver[0]);

    let segment_table = (0..ne_header.segment_count).map(|_| {
        NeSegment::read(&mut cursor)
    }).collect::<Result<Vec<_>, _>>()?;
    debug!("segment_table = {:#?}", segment_table);
    for (i, segment) in segment_table.iter().enumerate() {
        println!("Segment #{}:", i);
        println!("    Offset on file: 0x{:04X}", segment.data_offset(&ne_header));
        println!("    Length on file: 0x{:04X}", segment.data_length());
        println!("    Flags: 0x{:04X}", segment.flags);
        println!("    Allocation: 0x{:04X}", segment.min_alloc());
    }
    Ok(())
}

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
    pub resource_table_entries:  u16,
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
        let get_u16 = |pos| {
            u16::from_le_bytes(buf[pos..pos+2].try_into().unwrap())
        };
        let get_u32 = |pos| {
            u32::from_le_bytes(buf[pos..pos+4].try_into().unwrap())
        };

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
            resource_table_entries:  get_u16(0x34),
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
        let get_u16 = |pos| {
            u16::from_le_bytes(buf[pos..pos+2].try_into().unwrap())
        };

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
