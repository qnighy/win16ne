///
/// This module contains information entities for
/// Segment relocations. 
/// Mostly expected for importing procedures adresses
/// and importing procedure @ordinals
/// 
use std::io::{self, Read};

#[derive(Debug, Clone)]
pub enum RelocationTarget {
    Internal(InternalFixes),
    ImportByOrdinal(ImportByOrdinal), 
    ImportByName(ImportByName)
}

#[derive(Debug, Clone)]
pub struct RelocationEntry {
    pub address_type: u8,
    pub reloc_type: u8,
    pub is_additive: bool,
    pub segment_offset: u16,
    pub target: RelocationTarget,
}

#[derive(Debug, Clone)]
pub struct RelocationTable {
    pub entries: Vec<RelocationEntry>,
}
#[derive(Debug, Clone, Copy)]
pub struct InternalFixes {
    pub segment: u8,
    pub is_movable: bool,
    pub offset_or_ordinal: u16,
}
#[derive(Debug, Clone, Copy)]
pub struct ImportByOrdinal {
    pub module_index: u16,
    pub ordinal: u16,
}
#[derive(Debug, Copy, Clone)]
pub struct ImportByName {
    pub module_index: u16,
    pub name_offset: u16,
}

impl RelocationTable {
    /// Reads relocation table for a segment
    /// 
    /// According to MS-DOS Encyclopedia Appendix K:
    /// - First 2 bytes: number of relocation items
    /// - Each item is 8 bytes
    /// 
    /// \param r - reader reference
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut count_buf = [0; 2];
        r.read_exact(&mut count_buf)?;
        let count = u16::from_le_bytes(count_buf);
        
        let mut entries = Vec::with_capacity(count as usize);
        
        for _ in 0..count {
            let mut entry_buf = [0; 8];
            r.read_exact(&mut entry_buf)?;
            
            let address_type = entry_buf[0];
            let reloc_flags = entry_buf[1];
            let reloc_type = reloc_flags & 0x03;  // Lower 2 bits
            let is_additive = (reloc_flags & 0x04) != 0;  // Bit 2
            let segment_offset = u16::from_le_bytes([entry_buf[2], entry_buf[3]]);
            
            let target = match reloc_type {
                // Internal reference
                0x00 => {
                    let segment = entry_buf[4];
                    let is_movable = segment == 0xFF;
                    let offset_or_ordinal = u16::from_le_bytes([entry_buf[6], entry_buf[7]]);
                    
                    let internal_fix: InternalFixes = InternalFixes {
                        segment: segment,
                        is_movable: is_movable,
                        offset_or_ordinal: offset_or_ordinal
                    };

                    RelocationTarget::Internal(internal_fix)
                }
                // Imported by ordinal
                0x01 => {
                    let module_index = u16::from_le_bytes([entry_buf[4], entry_buf[5]]);
                    let ordinal = u16::from_le_bytes([entry_buf[6], entry_buf[7]]);
                    
                    let import_by_odrinal: ImportByOrdinal = ImportByOrdinal {
                        module_index,
                        ordinal,
                    };

                    RelocationTarget::ImportByOrdinal(import_by_odrinal) 
                }
                // Imported by name
                0x02 => {
                    let module_index = u16::from_le_bytes([entry_buf[4], entry_buf[5]]);
                    let name_offset = u16::from_le_bytes([entry_buf[6], entry_buf[7]]);
                    
                    let import_by_name: ImportByName = ImportByName {
                        module_index,
                        name_offset,
                    };

                    RelocationTarget::ImportByName(import_by_name) 
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid relocation type: 0x{:02X}", reloc_type)),
                    );
                }
            };
            
            entries.push(RelocationEntry {
                address_type,
                reloc_type,
                is_additive,
                segment_offset,
                target,
            });
        }
        
        Ok(Self { entries })
    }
}