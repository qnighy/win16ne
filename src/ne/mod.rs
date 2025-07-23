use log::debug;
use std::io::{self, Read, Seek, SeekFrom};

use self::entry_table::EntryTable;
use self::header::NeHeader;
use self::module_reference_table::ModuleReferenceTable;
use self::nonresident_name_table::NonresidentNameTable;
use self::resident_name_table::ResidentNameTable;
use self::resource_table::NeResourceTable;
use self::segment_table::NeSegment;
use crate::mz::DosHeader;

pub mod entry_table;
pub mod header;
pub mod module_reference_table;
pub mod nonresident_name_table;
pub mod resident_name_table;
pub mod resource_table;
pub mod segment_table;

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
}

impl NeExecutable {
    ///
    /// Just reads NE image structures
    /// 
    pub fn read<R: Read + Seek>(file: &mut R) -> io::Result<Self> {
        let dos_header = DosHeader::read(file)?;
        debug!("dos_header = {:?}", dos_header);
        dos_header.check_magic()?;

        let lfanew = dos_header.lfanew.value() as u64;

        file.seek(SeekFrom::Start(lfanew))?;

        let ne_header = NeHeader::read(file)?;
        debug!("ne_header = {:#?}", ne_header);
        ne_header.check_magic()?;

        file.seek(SeekFrom::Start(
            lfanew + ne_header.segment_table_offset.value() as u64,
        ))?;

        let mut segment_entries = (0..ne_header.segment_count.value())
            .map(|_| NeSegment::read(file, ne_header.file_alignment_shift_count.value()))
            .collect::<Result<Vec<_>, _>>()?;
        debug!("segment_entries = {:#?}", segment_entries);

        let rt_offset = lfanew + ne_header.resource_table_offset.value() as u64;
        file.seek(SeekFrom::Start(rt_offset))?;
        let resource_table = if ne_header.resource_table_entries.value() == 0xFFFF {
            NeResourceTable::read_variadic(file)?
        } else {
            NeResourceTable::read(file, ne_header.resource_table_entries.value())?
        };
        debug!("resource_table = {:#?}", resource_table);

        let rnt_offset = lfanew + ne_header.resident_names_table_offset.value() as u64;
        file.seek(SeekFrom::Start(rnt_offset))?;
        let resident_name_table = ResidentNameTable::read(file)?;
        debug!("resident_name_table = {:#?}", resident_name_table);

        let mrt_offset = lfanew + ne_header.module_reference_table_offset.value() as u64;
        file.seek(SeekFrom::Start(mrt_offset))?;
        let mut module_reference_table =
            ModuleReferenceTable::read(file, ne_header.module_references.value())?;
        debug!("module_reference_table = {:#?}", module_reference_table);

        let int_offset = lfanew + ne_header.import_name_table_offset.value() as u64;
        module_reference_table.read_names(file, int_offset)?;

        let et_offset = lfanew + ne_header.entry_table_offset.value() as u64;
        file.seek(SeekFrom::Start(et_offset))?;

        // replaced: read(x) -> read_sf(x)
        let entry_table = EntryTable::read_sf(file, ne_header.entry_table_offset.value(), ne_header.entry_table_length.value())?;
        debug!("entry_table = {:#?}", entry_table);

        let nnt_offset = ne_header.non_resident_names_table_offset.value() as u64;
        file.seek(SeekFrom::Start(nnt_offset))?;
        let nonresident_name_table = NonresidentNameTable::read(file)?;
        debug!("nonresident_name_table = {:#?}", nonresident_name_table);

        for segment in &mut segment_entries {
            segment.read_data(file)?;
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
        println!(
            "\tLinker version: {}.{}",
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
                print!("0");
            }
        }
        println!();
        println!(
            "    Auto-data segment: {}",
            ne_header.auto_data_segment_index.value()
        );
        println!(
            "    Initial heap size: {}",
            ne_header.init_heap_size.value()
        );
        println!(
            "    Initial stack size: {}",
            ne_header.init_stack_size.value()
        );
        println!(
            "    Entry point (CS:IP): {:04X}:{:04X}",
            ne_header.entry_point.value() >> 16,
            ne_header.entry_point.value() & 0xFFFF
        );
        println!(
            "    Initial stack (SS:SP): {:04X}:{:04X}",
            ne_header.init_stack.value() >> 16,
            ne_header.init_stack.value() & 0xFFFF
        );
        println!(
            "    Number of segments: {}",
            ne_header.segment_count.value()
        );
        println!(
            "    Number of referenced modules: {}",
            ne_header.module_references.value()
        );
        println!(
            "    Number of movable entry points: {}",
            ne_header.movable_entry_point_count.value()
        );
        println!(
            "    Number of file alignment shifts: {}",
            ne_header.file_alignment_shift_count.value()
        );
        println!(
            "    Number of resource table entries: {}",
            ne_header.resource_table_entries.value()
        );
        print!("    Target os: ");

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
            println!("Segment #{}:", i);
            println!("\tOffset on file: 0x{:04X}", segment.data_offset());
            println!("\tLength on file: 0x{:04X}", segment.data_length());
            println!("\tFlags: 0x{:04X}", segment.header.flags);
            println!("\tAllocation: 0x{:04X}", segment.min_alloc());
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
        if !self.resident_name_table.entries.is_empty() {
            println!("Resident names:");
            for entry in &self.resident_name_table.entries[1..] {
                println!(
                    "\t{:3} {}",
                    entry.index,
                    String::from_utf8_lossy(&entry.name)
                );
            }
        }
        if !self.nonresident_name_table.entries.is_empty() {
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
                    println!("Entry #{}: unused", i + 1);
                }
                Fixed(entry) => {
                    println!("Entry #{}: fixed", i + 1);
                    println!("\tSegment: {}", entry.segment);
                    println!("\tFlags: 0x{:02X}", entry.flags);
                    println!("\tOffset: 0x{:04X}", entry.offset);
                }
                Moveable(entry) => {
                    println!("Entry #{}: moveable", i + 1);
                    println!("\tFlags: 0x{:02X}", entry.flags);
                    if entry.magic != *b"\xCD\x3F" { // movable entry must have INT 3Fh instruction.
                        println!(
                            "\t<Invalid magic>: {:02X} {:02X}",
                            entry.magic[0], entry.magic[1]
                        );
                    }
                    println!("\tSegment: 0x{:02X}", entry.segment);
                    println!("\tOffset: 0x{:04X}", entry.offset);
                }
            }
        }

        for (_i, segment) in segment_entries.iter().enumerate() {
            if !disassemble || (segment.header.flags & 7) != 0 {
                continue;
            }
            if let Some(data) = &segment.data {
                crate::x86::disassemble(data, false);
            }
        }

        for (i, segment) in segment_entries.iter().enumerate() {
            if !show_data {
                continue;
            }
            if let Some(data) = &segment.data {
                println!("Segment #{} data:", i);
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
