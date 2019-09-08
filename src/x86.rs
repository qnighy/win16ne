use std::fmt;

pub fn disassemble(code: &[u8], is_32: bool) {
    let mut insts = Vec::new();
    let mut pos = 0;
    while pos < code.len() {
        let mut inst = eat(&code[pos..], is_32).unwrap_or_else(|_| gen_invalid(code[pos]));
        inst.pos = pos;
        pos += inst.len();
        insts.push(inst);
    }

    println!("0000:0000 <.text>:");
    for inst in &insts {
        let pos = inst.pos;
        let len = inst.len();
        for skip in 0..((len + 6) / 7) {
            print!("{:4X}:   ", pos + skip);
            for i in 0..7 {
                if i < len {
                    print!("{:02X} ", code[pos + skip + i]);
                } else {
                    print!("   ");
                }
            }
            if skip == 0 {
                println!("   {}", inst);
            }
        }
    }
}

fn eat(code: &[u8], is_32c: bool) -> Result<Inst, EatError> {
    let mut eater = SimpleEater::new(code);
    let inst_prefix = eater.next_if(|b| b == 0xF0 || b == 0xF2 || b == 0xF3);
    let addr_prefix = eater.next_if(|b| b == 0x67);
    let size_prefix = eater.next_if(|b| b == 0x66);
    let segm_prefix =
        eater.next_if(|b| [0x26, 0x2E, 0x36, 0x3E, 0x64, 0x65].iter().any(|&x| b == x));

    let opcode = eater.next()?;

    const OPCODE_VALIDITY_MAP: [u32; 8] = [
        0b11111111_11111111_11111111_11111111,
        0b10111111_10111111_10111111_10111111,
        0b11111111_11111111_11111111_11111111,
        0b11111111_11111111_11111111_00001111,
        0b11111111_11111111_11111111_11111011,
        0b11111111_11111111_11111111_11111111,
        0b11111111_10111111_11111111_11111111,
        0b11111111_11110000_11111111_11111111,
    ];
    if !lookup_byte(&OPCODE_VALIDITY_MAP, opcode) {
        return Err(EatError);
    }

    let opcode2 = if opcode == 0x0F {
        Some(eater.next()?)
    } else {
        None
    };

    const OPCODE2_VALIDITY_MAP: [u32; 8] = [
        0b00000000_00000000_00000000_01001111,
        0b00000000_00000000_00000000_01011111,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00000000_00000000_00000000,
        0b11111111_11111111_11111111_11111111,
        0b11111100_11111100_10111011_00111011,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00000000_00000000_00000000,
    ];
    if let Some(opcode2) = opcode2 {
        if !lookup_byte(&OPCODE2_VALIDITY_MAP, opcode2) {
            return Err(EatError);
        }
    }

    const HAS_MODRM: [u32; 8] = [
        0b00111111_00111111_00111111_00111111,
        0b00111111_00111111_00111111_00111111,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00000000_11111010_00001100,
        0b00000000_00000000_11111111_11111011,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00001111_00000000_11110011,
        0b11000000_11000000_11110000_00000000,
    ];
    const HAS_MODRM2: [u32; 8] = [
        0b00000000_00000000_00000000_00001100,
        0b00000000_00000000_00000000_01011111,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00000000_00000000_00000000,
        0b11111100_11111100_10111011_00111000,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00000000_00000000_00000000,
    ];
    let has_modrm = if let Some(opcode2) = opcode2 {
        lookup_byte(&HAS_MODRM2, opcode2)
    } else {
        lookup_byte(&HAS_MODRM, opcode)
    };
    let modrm = if has_modrm { Some(eater.next()?) } else { None };

    let is_32a = is_32c ^ addr_prefix.is_some();
    let is_32d = is_32c ^ size_prefix.is_some();

    let has_sib = if let Some(modrm) = modrm {
        is_32a && (modrm & 56) == 32 && (modrm & 192) != 192
    } else {
        false
    };
    let sib = if has_sib { Some(eater.next()?) } else { None };

    let disp_size = if let Some(modrm) = modrm {
        let (mod_, _, rm) = split233(modrm);
        if is_32a {
            if mod_ == 1 {
                ImmediateSize::Byte
            } else if mod_ == 2 {
                ImmediateSize::DWord
            } else if mod_ == 0 && rm == 5 {
                ImmediateSize::DWord
            } else {
                ImmediateSize::None
            }
        } else {
            if mod_ == 1 {
                ImmediateSize::Byte
            } else if mod_ == 2 {
                ImmediateSize::Word
            } else if mod_ == 0 && rm == 6 {
                ImmediateSize::Word
            } else {
                ImmediateSize::None
            }
        }
    } else {
        ImmediateSize::None
    };
    let disp = match disp_size {
        ImmediateSize::None => Immediate::None,
        ImmediateSize::Byte => Immediate::Byte(eater.next()?),
        ImmediateSize::Word => Immediate::Word(u16::from_le_bytes([eater.next()?, eater.next()?])),
        ImmediateSize::DWord => Immediate::DWord(u32::from_le_bytes([
            eater.next()?,
            eater.next()?,
            eater.next()?,
            eater.next()?,
        ])),
    };

    const IMMEDIATE_MAP: [u32; 8] = [
        0b00110000_00110000_00110000_00110000,
        0b00110000_00110000_00110000_00110000,
        0b00000000_00000000_00000000_00000000,
        0b11111111_11111111_00001111_00000000,
        0b00000000_00000000_00000000_00001011,
        0b11111111_11111111_00000011_00000000,
        0b00000000_00000000_00100101_11000111,
        0b00000000_11000000_00001111_11111111,
    ];
    const IMMEDIATE_BYTE_MAP: [u32; 8] = [
        0b00010000_00010000_00010000_00010000,
        0b00010000_00010000_00010000_00010000,
        0b00000000_00000000_00000000_00000000,
        0b11111111_11111111_00000101_00000000,
        0b00000000_00000000_00000000_00001001,
        0b00000000_11111111_00000001_00000000,
        0b00000000_00000000_00100001_01000001,
        0b00000000_00000000_00001000_11111111,
    ];
    const IMMEDIATE_WIDE_MAP: [u32; 8] = [
        0b00100000_00100000_00100000_00100000,
        0b00100000_00100000_00100000_00100000,
        0b00000000_00000000_00000000_00000000,
        0b00000000_00000000_00001010_00000000,
        0b00000000_00000000_00000000_00000010,
        0b11111111_00000000_00000010_00000000,
        0b00000000_00000000_00000000_10000010,
        0b00000000_00000000_00000011_00000000,
    ];

    let immediate_size = if !lookup_byte(&IMMEDIATE_MAP, opcode) {
        ImmediateSize::None
    } else if lookup_byte(&IMMEDIATE_BYTE_MAP, opcode) {
        ImmediateSize::Byte
    } else if lookup_byte(&IMMEDIATE_WIDE_MAP, opcode) {
        if is_32d {
            ImmediateSize::DWord
        } else {
            ImmediateSize::Word
        }
    } else if opcode == 0xC2 || opcode == 0xCA {
        ImmediateSize::Word
    } else {
        // TODO: EA, F6, F7
        ImmediateSize::None
    };
    let imm = match immediate_size {
        ImmediateSize::None => Immediate::None,
        ImmediateSize::Byte => Immediate::Byte(eater.next()?),
        ImmediateSize::Word => Immediate::Word(u16::from_le_bytes([eater.next()?, eater.next()?])),
        ImmediateSize::DWord => Immediate::DWord(u32::from_le_bytes([
            eater.next()?,
            eater.next()?,
            eater.next()?,
            eater.next()?,
        ])),
    };

    Ok(Inst {
        pos: 0,
        is_invalid: false,
        is_32c,
        inst_prefix,
        addr_prefix,
        size_prefix,
        segm_prefix,
        opcode,
        opcode2,
        modrm,
        sib,
        displacement: disp,
        immediate: imm,
    })
}

fn gen_invalid(byte: u8) -> Inst {
    Inst {
        pos: 0,
        is_invalid: true,
        is_32c: false,
        inst_prefix: None,
        addr_prefix: None,
        size_prefix: None,
        segm_prefix: None,
        opcode: byte,
        opcode2: None,
        modrm: None,
        sib: None,
        displacement: Immediate::None,
        immediate: Immediate::None,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Inst {
    pub pos: usize,
    pub is_invalid: bool,
    pub is_32c: bool,
    pub inst_prefix: Option<u8>,
    pub addr_prefix: Option<u8>,
    pub size_prefix: Option<u8>,
    pub segm_prefix: Option<u8>,
    pub opcode: u8,
    pub opcode2: Option<u8>,
    pub modrm: Option<u8>,
    pub sib: Option<u8>,
    pub displacement: Immediate,
    pub immediate: Immediate,
}

impl Inst {
    fn len(&self) -> usize {
        self.inst_prefix.is_some() as usize
            + self.addr_prefix.is_some() as usize
            + self.size_prefix.is_some() as usize
            + self.segm_prefix.is_some() as usize
            + 1
            + self.opcode2.is_some() as usize
            + self.modrm.is_some() as usize
            + self.sib.is_some() as usize
            + self.displacement.len()
            + self.immediate.len()
    }

    fn is_32a(&self) -> bool {
        self.is_32c ^ self.addr_prefix.is_some()
    }

    fn is_32d(&self) -> bool {
        self.is_32c ^ self.size_prefix.is_some()
    }

    fn rm_name(&self, wide: bool) -> RmDisp {
        RmDisp {
            wide,
            is_32a: self.is_32a(),
            is_32d: self.is_32d(),
            disp: self.displacement,
            modrm: self.modrm.unwrap_or(0),
        }
    }

    fn reg_name(&self, wide: bool) -> &'static str {
        let (_, reg, _) = split233(self.modrm.unwrap_or(0));
        regname(reg, self.is_32d(), wide)
    }
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_invalid {
            return write!(f, "<invalid>");
        }
        match self.opcode {
            opcode if (0..0x40).contains(&opcode) && opcode & 7 < 6 => {
                let opname =
                    ["add", "or", "adc", "sbb", "and", "sub", "xor", "cmp"][(opcode >> 3) as usize];
                if opcode & 4 == 0 {
                    let wide = opcode & 1 != 0;
                    let reg = self.reg_name(wide);
                    let rm = self.rm_name(wide);
                    if opcode & 2 == 0 {
                        write!(f, "{} %{}, {}", opname, reg, rm)
                    } else {
                        write!(f, "{} {}, %{}", opname, rm, reg)
                    }
                } else {
                    write!(f, "{} ...", opname)
                }
            }
            0x55 => write!(f, "nop"),
            0x80 | 0x81 | 0x83 => {
                let (_, subop, _) = split233(self.modrm.unwrap_or(0));
                let opname =
                    ["add", "or", "adc", "sbb", "and", "sub", "xor", "cmp"][subop as usize];
                let imm = SignedImmDisp(self.immediate);
                let rm = self.rm_name(self.opcode != 0x80);
                write!(f, "{} {}, {}", opname, imm, rm)
            }
            opcode if (0x88..0x8C).contains(&opcode) => {
                let wide = opcode & 1 != 0;
                let reg = self.reg_name(wide);
                let rm = self.rm_name(wide);
                if opcode & 2 == 0 {
                    write!(f, "mov %{}, {}", reg, rm)
                } else {
                    write!(f, "mov {}, %{}", rm, reg)
                }
            }
            _ => write!(f, "..."),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImmediateSize {
    None,
    Byte,
    Word,
    DWord,
}

#[derive(Debug, Clone, Copy)]
pub enum Immediate {
    None,
    Byte(u8),
    Word(u16),
    DWord(u32),
}

impl Immediate {
    fn len(&self) -> usize {
        use self::Immediate::*;
        match self {
            None => 0,
            Byte(_) => 1,
            Word(_) => 2,
            DWord(_) => 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EatError;

struct SimpleEater<'a> {
    code: &'a [u8],
    pos: usize,
}

impl<'a> SimpleEater<'a> {
    fn new(code: &'a [u8]) -> Self {
        Self { code, pos: 0 }
    }
    fn next_if<F>(&mut self, f: F) -> Option<u8>
    where
        F: FnOnce(u8) -> bool,
    {
        if self.pos < self.code.len() && f(self.code[self.pos]) {
            let ret = self.code[self.pos];
            self.pos += 1;
            Some(ret)
        } else {
            None
        }
    }
    fn next(&mut self) -> Result<u8, EatError> {
        if self.pos < self.code.len() {
            let ret = self.code[self.pos];
            self.pos += 1;
            Ok(ret)
        } else {
            Err(EatError)
        }
    }
    fn peek(&self) -> Option<u8> {
        if self.pos < self.code.len() {
            Some(self.code[self.pos])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RmDisp {
    is_32a: bool,
    is_32d: bool,
    wide: bool,
    modrm: u8,
    disp: Immediate,
}

impl fmt::Display for RmDisp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (mod_, _, rm) = split233(self.modrm);
        if mod_ == 3 {
            write!(f, "%{}", regname(rm, self.is_32d, self.wide))
        } else if self.is_32a {
            write!(f, "...")
        } else {
            if mod_ == 0 && rm == 6 {
                write!(f, "{}", DispDisp(self.disp))
            } else if mod_ == 0 {
                write!(f, "{}", RM16_TABLE[rm as usize])
            } else {
                write!(f, "{}{}", DispDisp(self.disp), RM16_TABLE[rm as usize])
            }
        }
    }
}
const RM16_TABLE: [&str; 8] = [
    "(%bx,%si)",
    "(%bx,%di)",
    "(%bp,%si)",
    "(%bp,%di)",
    "(%si)",
    "(%di)",
    "(%bp)",
    "(%bx)",
];

fn regname(id: u8, is_32d: bool, wide: bool) -> &'static str {
    if !wide {
        ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"][id as usize]
    } else if !is_32d {
        ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"][id as usize]
    } else {
        ["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"][id as usize]
    }
}

#[derive(Debug, Clone, Copy)]
struct SignedImmDisp(Immediate);

impl fmt::Display for SignedImmDisp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Immediate::*;
        match self.0 {
            None => Ok(()),
            Byte(x) => write!(f, "${:#x}", x as i8),
            Word(x) => write!(f, "${:#x}", x as i16),
            DWord(x) => write!(f, "${:#x}", x as i32),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DispDisp(Immediate);

impl fmt::Display for DispDisp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Immediate::*;
        match self.0 {
            None => Ok(()),
            Byte(x) => write!(f, "{:#x}", x as i8),
            Word(x) => write!(f, "{:#x}", x as i16),
            DWord(x) => write!(f, "{:#x}", x as i32),
        }
    }
}

fn lookup_byte(table: &[u32; 8], byte: u8) -> bool {
    (table[(byte >> 5) as usize] >> (byte & 31)) & 1 != 0
}

fn split233(byte: u8) -> (u8, u8, u8) {
    (byte >> 6, (byte >> 3) & 7, byte & 7)
}
