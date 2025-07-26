use log::{debug, error};
use std::io::{self, Read, Seek, SeekFrom};

use self::entry_table::EntryTable;
use self::header::NeHeader;
use self::module_reference_table::ModuleReferenceTable;
use self::nonresident_name_table::NonresidentNameTable;
use self::resident_name_table::ResidentNameTable;
use self::resource_table::NeResourceTable;
use self::segment_table::NeSegment;
use crate::mz::DosHeader;
use crate::ne::segment_relocations::{RelocationTable, RelocationTarget};

pub mod entry_table;
pub mod header;
pub mod module_reference_table;
pub mod nonresident_name_table;
pub mod resident_name_table;
pub mod resource_table;
pub mod segment_table;
pub mod segment_relocations;

/// The parsed New Executable binary.
#[derive(Debug, Clone)]
pub struct NeExecutable {
    pub dos_header: Box<DosHeader>,
    pub ne_header: Box<NeHeader>,
    pub segment_entries: Vec<NeSegment>,
    pub resource_table: NeResourceTable,
    pub resident_name_table: ResidentNameTable,
    pub module_reference_table: ModuleReferenceTable,
    pub entry_table: EntryTable,
    pub nonresident_name_table: NonresidentNameTable,
    pub relocation_tables_per_segment: Vec<RelocationTable>
}

impl NeExecutable {
    ///
    /// Just reads NE image structures
    /// 
    pub fn read<R: Read + Seek>(file: &mut R) -> io::Result<Self> {
        let dos_header = DosHeader::read(file)?;
        debug!("dos_header = {:?}", dos_header);
        
        match dos_header.check_magic() {
            Ok(_) => (),
            Err(e) => {
                return Err(e); // |<-- target application can't be NE segmented image 
            }
        };

        let lfanew = dos_header.lfanew.value() as u64;

        file.seek(SeekFrom::Start(lfanew))?;

        let ne_header = NeHeader::read(file)?;
        ne_header.check_magic()?;

        file.seek(SeekFrom::Start(
            lfanew + ne_header.segment_table_offset.value() as u64,
        ))?;

        let mut segment_entries = (0..ne_header.segment_count.value())
            .map(|_| NeSegment::read(file, ne_header.file_alignment_shift_count.value()))
            .collect::<Result<Vec<_>, _>>()?;
        
        let rt_offset = lfanew + ne_header.resource_table_offset.value() as u64;

        file.seek(SeekFrom::Start(rt_offset))?;
        let resource_table = if ne_header.resource_table_entries.value() == 0xFFFF {
            NeResourceTable::read_variadic(file)?
        } else {
            NeResourceTable::read(file, ne_header.resource_table_entries.value())?
        };
        
        let rnt_offset = lfanew + ne_header.resident_names_table_offset.value() as u64;
        file.seek(SeekFrom::Start(rnt_offset))?;
        let resident_name_table = ResidentNameTable::read(file)?;
        
        let mrt_offset = lfanew + ne_header.module_reference_table_offset.value() as u64;
        file.seek(SeekFrom::Start(mrt_offset))?;
        let mut module_reference_table =
            ModuleReferenceTable::read(file, ne_header.module_references.value())?;
        
        let int_offset = lfanew + ne_header.import_name_table_offset.value() as u64;
        module_reference_table.read_names(file, int_offset)?;

        let et_offset = lfanew + ne_header.entry_table_offset.value() as u64;
        file.seek(SeekFrom::Start(et_offset))?;

        // replaced: read(x) -> read_sf(x)
        let entry_table = EntryTable::read_sf(file, ne_header.entry_table_length.value())?;
        
        let nnt_offset = ne_header.non_resident_names_table_offset.value() as u64;
        file.seek(SeekFrom::Start(nnt_offset))?;
        let nonresident_name_table = NonresidentNameTable::read(file)?;
        

        let mut relocs_per_segment = Vec::<RelocationTable>::new();
        
        for segment in &mut segment_entries {
            segment.read_data(file)?;
            if segment.header.flags & 0x0008 != 0 {  // must not be SEG_WITHIN_RELOCS
                let relocations = RelocationTable::read(file)?;
                relocs_per_segment.push(relocations);
            }
        }

        Ok(Self {
            dos_header: Box::new(dos_header),
            ne_header: Box::new(ne_header),
            segment_entries,
            resource_table,
            resident_name_table,
            module_reference_table,
            entry_table,
            nonresident_name_table,
            relocation_tables_per_segment: relocs_per_segment
        })
    }
    ///
    /// Writes read information of NE image in terminal.
    /// 
    pub(crate) fn describe(&self, show_data: bool, disassemble: bool) {
        let Self {
            ne_header,
            segment_entries,
            ..
        } = self;

        println!("File Type: Windows New Executable");
        println!("Header:");
        match ne_header.major_linker_version {
            0..3 => {
                error!("Module made by LINK.EXE {}.{}! Image's structures are unsupported!", ne_header.major_linker_version, ne_header.minor_linker_version);
            },
            _ => {
                println!("LINK.EXE version supported");
            }
        }
        println!(
            "\tLINK.EXE {}.{}",
            ne_header.major_linker_version, ne_header.minor_linker_version
        );
        print!("\tFlags: ");
        {
            let mut flag_found = false;
            for shift in 0..16 {
                let mask = 1 << shift;
                if (mask & ne_header.flags.value()) == 0 {
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
                error!("No program flags");
            }
        }
        println!();
        println!(
            "\tAuto-data segment: {}",
            ne_header.auto_data_segment_index.value()
        );
        println!(
            "\tInitial heap size: {}",
            ne_header.init_heap_size.value()
        );
        println!(
            "\tInitial stack size: {}",
            ne_header.init_stack_size.value()
        );
        println!(
            "\tEntry point (CS:IP): {:04X}:{:04X}",
            ne_header.entry_point.value() >> 16,
            ne_header.entry_point.value() & 0xFFFF
        );
        println!(
            "\tInitial stack (SS:SP): {:04X}:{:04X}",
            ne_header.init_stack.value() >> 16,
            ne_header.init_stack.value() & 0xFFFF
        );
        println!(
            "\tNumber of segments: {}",
            ne_header.segment_count.value()
        );
        println!(
            "\tNumber of referenced modules: {}",
            ne_header.module_references.value()
        );
        println!(
            "\tNumber of movable entry points: {}",
            ne_header.movable_entry_point_count.value()
        );
        println!(
            "\tNumber of file alignment shifts: {}",
            ne_header.file_alignment_shift_count.value()
        );
        println!(
            "\tNumber of resource table entries: {}",
            ne_header.resource_table_entries.value()
        );
        
        print!("\tTarget OS: ");
        match ne_header.target_os {
            0x0 => print!("Not specified"),
            0x1 => print!("OS/2"),
            0x2 => print!("Windows/286"),
            0x3 => print!("DOS 4.x"),
            0x4 => print!("Windows/386"),
            0x5 => print!("Borland OSS"),
            _ => print!("Unknown {:X}", ne_header.target_os)
        }

        println!();
        match ne_header.expected_win_ver[1] {
            0 => (),    
            _ => {
                println!(
                    "\tExpected Windows version: {}.{}",
                    ne_header.expected_win_ver[1], ne_header.expected_win_ver[0]
                );
            }
        }   

        for (i, segment) in segment_entries.iter().enumerate() {
            // SEGMENTS TABLE info
            println!("Segment #{}:", i + 1);
            println!("\tOffset on file: 0x{:04X}", segment.data_offset());
            println!("\tLength on file: 0x{:04X}", segment.data_length());
            println!("\tFlags: 0x{:04X}", segment.header.flags);
            println!("\tAllocation: 0x{:04X}", segment.min_alloc());

            // SEGMENT RELOCATIONS info
            if self.relocation_tables_per_segment.len() == 0 {
                println!("\tSEG_WITHIN_RELOCS");
                continue;
            }

            for (reloc_index, reloc) in self.relocation_tables_per_segment[i].entries.iter().enumerate() {
                println!("--------------------------------------------------");
                println!("\tRelocation #{}", reloc_index + 1);
                println!("\t\tATP: 0x{:2X}", reloc.address_type);
                println!("\t\tRTP: 0x{:2X}", reloc.reloc_type);
                println!("\t\tAdditive? {}", reloc.is_additive);
                println!("\t\tSegment offset: 0x{:X}", reloc.segment_offset);
                println!("--------------------------------------------------");

                match &reloc.target {
                    RelocationTarget::Internal(f) => {
                        println!("\t\tSEG_RELOC_INTERNAL_FIXES");
                        println!("\t\t#Segment {}", f.segment);
                        println!("\t\tOffset {:X}", f.offset_or_ordinal);
                        println!("\t\t.MOVEABLE? {}", f.is_movable);
                    },
                    RelocationTarget::ImportByOrdinal(o) => {
                        println!("SEG_RELOC_IMPORT_BY_ORDINAL");
                        println!("Procedure: @{}", o.ordinal);
                        println!("Module# {}", o.module_index);
                    },
                    RelocationTarget::ImportByName(n) => {
                        println!("SEG_RELOC_IMPORT_BY_NAME");
                        println!("Procedure name offset: {}", n.name_offset);
                        println!("Module# {}", n.module_index)
                    }
                }
                
                println!();
            }
        }

        if self.resident_name_table.entries.is_empty() {
            println!("Module name: <no entry>");
        } else {
            println!(
                "Module name: {}",
                String::from_utf8_lossy(&self.resident_name_table.entries[0].name)
            );
        }
        if self.nonresident_name_table.entries.is_empty() {
            println!("Module description: <no entry>");
        } else {
            println!(
                "Module description: {}",
                String::from_utf8_lossy(&self.nonresident_name_table.entries[0].name)
            );
        }
        
        if !self.resident_name_table.entries.len() > 1 {
            println!("Resident names:");
            for entry in &self.resident_name_table.entries[1..] {
                println!(
                    "\t{:3} {}",
                    entry.index,
                    String::from_utf8_lossy(&entry.name)
                );
            }
        }
        if !self.nonresident_name_table.entries.len() > 1 {
            println!("Nonresident names:");
            for entry in &self.nonresident_name_table.entries[1..] {
                println!(
                    "\t{:3} {}",
                    entry.index,
                    String::from_utf8_lossy(&entry.name)
                );
            }
        }

        println!("Module references:");
        for entry in &self.module_reference_table.entries {
            println!("\t{}", String::from_utf8_lossy(&entry.name));
        }

        for (i, entry) in self.entry_table.entries.iter().enumerate() {
            use self::entry_table::SegmentEntry::*;
            match entry {
                Unused => {
                    println!("Entry #{}: .UNUSED", i + 1);
                }
                Fixed(entry) => {
                    println!("Entry #{}: .FIXED", i + 1);
                    println!("\tSegment: {}", entry.segment);
                    println!("\tFlags: 0x{:02X}", entry.flags);
                    println!("\tOffset: 0x{:04X}", entry.offset);
                }
                Moveable(entry) => {
                    println!("Entry #{}: .MOVEABLE", i + 1);
                    println!("\tFlags: 0x{:02X}", entry.flags);
                    if entry.magic != *b"\xCD\x3F" { // movable entry must have INT 3Fh instruction.
                        error!(
                            "\t<Invalid magic>: {:02X} {:02X}",
                            entry.magic[0], entry.magic[1]
                        );
                    }
                    println!("\tSegment: 0x{:02X}", entry.segment);
                    println!("\tOffset: 0x{:04X}", entry.offset);
                }
            }
        }

        // TODO: .ITERATED segments .DATA segments
        if disassemble {
            for (segment_index, segment) in segment_entries.iter().enumerate() {
                // if segment has SEG_MASK flag (0x07), .CODE segments must be processed too.
                // see Microsoft Segmented EXE (New executable) Format sources or Wine-VDM
                // https://github.com/AlexeyTolstopyatov/old-executables-documentation

                match &segment.data {
                    Some(data) => {
                        let (segment_type, is_data) = match segment.header.flags & 0x0001 != 0 {
                            true => (".DATA", true),
                            false => (".CODE", false)
                        };
                        if segment.header.flags & 0x07 != 0 { // has mask
                            println!("\tSEG_HAS_MASK");
                        }
                        let (segment_compressed, is_iterated) = match segment.header.flags & 0x0002 != 0 {
                            true => {
                                (".ITERATED", true)
                            },
                            false => ("normal", false) // do nothing
                        };

                        println!("Segment #{} {} [{}]", segment_index + 1, segment_type, segment_compressed);
                        
                        define_disassemble(data, segment_type, is_data, is_iterated);
                    }
                    None => (),
                }
            }
        }
        if show_data {
            // HEXadecimal view for each segment
            for (i, segment) in segment_entries.iter().enumerate() {
                let segment_type = match segment.header.flags & 0x0001 != 0 {
                    true => ".DATA",
                    false => ".CODE"
                };
                if let Some(data) = &segment.data {
                    println!("Segment #{} {} view:", i + 1, segment_type);
                    for (i, chunk) in data.chunks(16).enumerate() {
                        print!("{:08X} ", i * 16);
                        for j in 0..16 {
                            if let Some(x) = chunk.get(j) {
                                print!(" {:02X}", x);
                            } else {
                                print!("   ");
                            }
                            if j == 7 {
                                print!(" ");
                            }
                        }
                        print!("  |");
                        for &byte in chunk {
                            if 0x20 <= byte && byte < 0x7F {
                                print!("{}", byte as char);
                            } else {
                                print!(".");
                            }
                        }
                        print!("|");
                        println!();
                    }
                    println!("{:08X}", (data.len() + 15) / 16 * 16);
                    println!();
                }
            }
        }
    }
}
///
/// Defines segment's storage type by flags in segment's header
/// and call disassemble procedure
/// 
fn define_disassemble(data: &Vec<u8>, segment_type: &'static str, is_data: bool, is_iterated: bool) {
    match is_data {
        true => println!("\tSkipped!"),
        false => {
            match !is_iterated {
                true => crate::x86::disassemble(data, false, segment_type),
                false => crate::x86::disassemble(&iter_segment_bytes(data), false, segment_type)
            }
        }
    }
}
///
/// If file segment has SEG_ITERATED flag,
/// it means that data compressed. 
/// 
/// Segmented EXE headedr Format doesn't tells: how actually compressed
/// This procedure is my suggestions how it may be. 
/// 
/// \param data -- compressed bytes slice
///
fn iter_segment_bytes(data: &[u8]) -> Vec<u8> {
    let iterations = u16::from_le_bytes([data[0], data[1]]);
    let data_size = u16::from_le_bytes([data[2], data[3]]);
    let raw_data = &data[4..4 + data_size as usize];
    
    raw_data.repeat(iterations as usize)
}