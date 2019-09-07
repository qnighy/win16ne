use std::convert::TryInto;
use std::io::{self, Read};

#[derive(Debug, Clone)]
pub struct NeResourceTable {
    pub header: NeResourceTableHeader,
    pub resource_types: Vec<NeResourceType>,
}
impl NeResourceTable {
    pub fn read<R: Read>(r: &mut R, num_entries: u16) -> io::Result<Self> {
        let header = NeResourceTableHeader::read(r)?;
        let resource_types = (0..num_entries)
            .map(|_| NeResourceType::read(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            header,
            resource_types,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NeResourceTableHeader {
    pub alignment_shift_count: u16,
}
impl NeResourceTableHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let alignment_shift_count = {
            let mut data = [0; 2];
            r.read_exact(&mut data)?;
            u16::from_le_bytes(data)
        };
        Ok(Self {
            alignment_shift_count,
        })
    }
}

#[derive(Debug, Clone)]
pub struct NeResourceType {
    pub header: NeResourceTypeHeader,
    pub resources: Vec<NeResource>,
}
impl NeResourceType {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let header = NeResourceTypeHeader::read(r)?;
        let resources = (0..header.num_resources)
            .map(|_| NeResource::read(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { header, resources })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NeResourceTypeHeader {
    pub type_id: u16,
    pub num_resources: u16,
    pub res: [u16; 2],
}
impl NeResourceTypeHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x8];
        r.read_exact(&mut buf)?;
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap());

        Ok(Self {
            type_id: get_u16(0),
            num_resources: get_u16(2),
            res: [get_u16(4), get_u16(6)],
        })
    }
}

#[derive(Debug, Clone)]
pub struct NeResource {
    pub header: NeResourceHeader,
}
impl NeResource {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(Self {
            header: NeResourceHeader::read(r)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NeResourceHeader {
    pub data_offset_shifted: u16,
    pub data_length: u16,
    pub flags: u16,
    pub resource_id: u16,
    pub res: [u16; 2],
}
impl NeResourceHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0xC];
        r.read_exact(&mut buf)?;
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap());

        Ok(Self {
            data_offset_shifted: get_u16(0),
            data_length: get_u16(2),
            flags: get_u16(4),
            resource_id: get_u16(6),
            res: [get_u16(8), get_u16(10)],
        })
    }
}
