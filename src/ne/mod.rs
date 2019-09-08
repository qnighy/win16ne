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
    pub fn read<R: Read + Seek>(file: &mut R) -> io::Result<Self> {
        let dos_header = DosHeader::read(file)?;
        debug!("dos_header = {:?}", dos_header);
        dos_header.check_magic()?;

        file.seek(SeekFrom::Start(dos_header.lfanew as u64))?;

        let ne_header = NeHeader::read(file)?;
        debug!("ne_header = {:#?}", ne_header);
        ne_header.check_magic()?;

        file.seek(SeekFrom::Start(
            dos_header.lfanew as u64 + ne_header.segment_table_offset as u64,
        ))?;

        let mut segment_entries = (0..ne_header.segment_count)
            .map(|_| NeSegment::read(file, ne_header.file_alignment_shift_count))
            .collect::<Result<Vec<_>, _>>()?;
        debug!("segment_entries = {:#?}", segment_entries);

        let rt_offset = dos_header.lfanew as u64 + ne_header.resource_table_offset as u64;
        file.seek(SeekFrom::Start(rt_offset))?;
        let resource_table = NeResourceTable::read(file, ne_header.resource_table_entries)?;
        debug!("resource_table = {:#?}", resource_table);

        let rnt_offset = dos_header.lfanew as u64 + ne_header.resident_names_table_offset as u64;
        file.seek(SeekFrom::Start(rnt_offset))?;
        let resident_name_table = ResidentNameTable::read(file)?;
        debug!("resident_name_table = {:#?}", resident_name_table);

        let mrt_offset = dos_header.lfanew as u64 + ne_header.module_reference_table_offset as u64;
        file.seek(SeekFrom::Start(mrt_offset))?;
        let mut module_reference_table =
            ModuleReferenceTable::read(file, ne_header.module_references)?;
        debug!("module_reference_table = {:#?}", module_reference_table);

        let int_offset = dos_header.lfanew as u64 + ne_header.import_name_table_offset as u64;
        module_reference_table.read_names(file, int_offset)?;

        let et_offset = dos_header.lfanew as u64 + ne_header.entry_table_offset as u64;
        let entry_table = EntryTable::read(file, et_offset, ne_header.entry_table_length)?;
        debug!("entry_table = {:#?}", entry_table);

        let nnt_offset = ne_header.non_resident_names_table_offset as u64;
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

    pub(crate) fn describe(&self, show_data: bool) {
        let Self {
            ne_header,
            segment_entries,
            ..
        } = self;

        println!("File Type: Windows New Executable");
        println!("Header:");
        println!(
            "    Linker version: {}.{}",
            ne_header.major_linker_version, ne_header.minor_linker_version
        );
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
        println!(
            "    Auto-data segment: {}",
            ne_header.auto_data_segment_index
        );
        println!("    Initial heap size: {}", ne_header.init_heap_size);
        println!("    Initial stack size: {}", ne_header.init_stack_size);
        println!(
            "    Entry point (CS:IP): {:04X}:{:04X}",
            ne_header.entry_point >> 16,
            ne_header.entry_point & 0xFFFF
        );
        println!(
            "    Initial stack (SS:SP): {:04X}:{:04X}",
            ne_header.init_stack >> 16,
            ne_header.init_stack & 0xFFFF
        );
        println!("    Number of segments: {}", ne_header.segment_count);
        println!(
            "    Number of referenced modules: {}",
            ne_header.module_references
        );
        println!(
            "    Number of movable entry points: {}",
            ne_header.movable_entry_point_count
        );
        println!(
            "    Number of file alignment shifts: {}",
            ne_header.file_alignment_shift_count
        );
        println!(
            "    Number of resource table entries: {}",
            ne_header.resource_table_entries
        );
        print!("    Target os: ");
        if ne_header.target_os == 2 {
            print!("Windows");
        } else {
            print!("Unknown ({})", ne_header.target_os);
        }
        println!();
        println!(
            "    Expected Windows version: {}.{}",
            ne_header.expected_win_ver[1], ne_header.expected_win_ver[0]
        );

        for (i, segment) in segment_entries.iter().enumerate() {
            println!("Segment #{}:", i);
            println!("    Offset on file: 0x{:04X}", segment.data_offset());
            println!("    Length on file: 0x{:04X}", segment.data_length());
            println!("    Flags: 0x{:04X}", segment.header.flags);
            println!("    Allocation: 0x{:04X}", segment.min_alloc());
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
                    "    {:3} {}",
                    entry.index,
                    String::from_utf8_lossy(&entry.name)
                );
            }
        }
        if !self.nonresident_name_table.entries.is_empty() {
            println!("Nonresident names:");
            for entry in &self.nonresident_name_table.entries[1..] {
                println!(
                    "    {:3} {}",
                    entry.index,
                    String::from_utf8_lossy(&entry.name)
                );
            }
        }

        println!("Module references:");
        for entry in &self.module_reference_table.entries {
            println!("    {}", String::from_utf8_lossy(&entry.name));
        }

        for (i, bundle) in self.entry_table.bundles.iter().enumerate() {
            use self::entry_table::EntryBundle::*;
            match bundle {
                Unused => {
                    println!("Bundle #{}: unused", i);
                }
                Fixed(bundle) => {
                    println!("Bundle #{}: fixed({})", i, bundle.segment);
                    for (j, entry) in bundle.entries.iter().enumerate() {
                        println!("    Entry #{}:", j);
                        println!("        Flags: 0x{:02X}", entry.flags);
                        println!("        Offset: 0x{:04X}", entry.offset);
                    }
                }
                Moveable(bundle) => {
                    println!("Bundle #{}: moveable", i);
                    for (j, entry) in bundle.entries.iter().enumerate() {
                        println!("    Entry #{}:", j);
                        println!("        Flags: 0x{:02X}", entry.flags);
                        if entry.magic != *b"\xCD\x3F" {
                            println!(
                                "        <Invalid magic>: {:02X} {:02X}",
                                entry.magic[0], entry.magic[1]
                            );
                        }
                        println!("        Segment: 0x{:02X}", entry.segment);
                        println!("        Offset: 0x{:04X}", entry.offset);
                    }
                }
            }
        }

        if !show_data {
            return;
        }

        for (i, segment) in segment_entries.iter().enumerate() {
            println!("Segment #{} data:", i);
            for (i, chunk) in segment.data.chunks(16).enumerate() {
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
            println!("{:08X}", (segment.data.len() + 15) / 16 * 16);
            println!();
        }
    }
}
