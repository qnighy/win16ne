use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct ModuleReferenceTable {
    pub entries: Vec<ModuleReferenceEntry>,
}

impl ModuleReferenceTable {
    pub fn read<R: Read>(r: &mut R, num: u16) -> io::Result<Self> {
        let entries = (0..num)
            .map(|_| ModuleReferenceEntry::read(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { entries })
    }

    pub fn read_names<R: Read + Seek>(&mut self, r: &mut R, offset: u64) -> io::Result<()> {
        for entry in &mut self.entries {
            entry.read_name(r, offset)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModuleReferenceEntry {
    pub header: ModuleReferenceEntryHeader,
    pub name: Vec<u8>,
}

impl ModuleReferenceEntry {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(Self {
            header: ModuleReferenceEntryHeader::read(r)?,
            name: Vec::new(),
        })
    }

    pub fn read_name<R: Read + Seek>(&mut self, r: &mut R, offset: u64) -> io::Result<()> {
        r.seek(SeekFrom::Start(offset + self.header.offset as u64))?;
        let len = {
            let mut len = 0;
            r.read_exact(std::slice::from_mut(&mut len))?;
            len
        };
        self.name.resize(len as usize, 0);
        r.read_exact(&mut self.name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModuleReferenceEntryHeader {
    pub offset: u16,
}

impl ModuleReferenceEntryHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let offset = {
            let mut buf = [0; 2];
            r.read_exact(&mut buf)?;
            u16::from_le_bytes(buf)
        };
        Ok(Self { offset })
    }
}
