use std::io::{self, Read};

///
/// This table contains a list of ASCII strings. 
/// The first string is the module description from the module definition file. 
/// 
/// The other strings are the names of all exported functions listed in the module
/// definition file that have ordinal numbers associated with them.
/// (Exported functions without ordinal numbers in the module definition
/// file are listed in the resident names table.)
///
/// Each string is prefaced by a single byte indicating the number of
/// characters in the string and is followed by a word (2 bytes)
/// referencing a member of the entry table, beginning at 1. The word that
/// follows the module description string is 0. (Offsets are from the
/// beginning of the table.)
///
#[derive(Debug, Clone)]
pub struct NonresidentNameTable {
    pub entries: Vec<NonresidentNameEntry>,
}

impl NonresidentNameTable {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut entries = Vec::new();
        while let Some(entry) = NonresidentNameEntry::read(r)? {
            entries.push(entry);
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone)]
pub struct NonresidentNameEntry {
    pub name: Vec<u8>,
    pub index: u16,
}

impl NonresidentNameEntry {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Option<Self>> {
        let len = {
            let mut len = 0;
            r.read_exact(std::slice::from_mut(&mut len))?;
            len
        };
        if len == 0 {
            return Ok(None);
        }
        let name = {
            let mut name = vec![0; len as usize];
            r.read_exact(&mut name)?;
            name
        };
        let index = {
            let mut buf = [0; 2];
            r.read_exact(&mut buf)?;
            u16::from_le_bytes(buf)
        };
        Ok(Some(Self { name, index }))
    }
}
