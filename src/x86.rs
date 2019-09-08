use std::fmt;

pub fn disassemble(code: &[u8]) {
    let mut insts = Vec::new();
    let mut pos = 0;
    while pos < code.len() {
        let mut inst = eat(&code[pos..]).unwrap_or_else(|| Inst {
            pos: 0,
            kind: InstKind::Unknown(code[pos]),
        });
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

fn eat(code: &[u8]) -> Option<Inst> {
    if (0x40..0x60).contains(&code[0]) {
        use self::RegisterOpKind::*;
        let kind = [Inc, Dec, Push, Pop][(code[0] >> 3) as usize & 3];
        let register = GeneralRegister::from_id(code[0] & 7);
        Some(Inst {
            pos: 0,
            kind: InstKind::RegisterOp { kind, register },
        })
    } else if (0..0x40).contains(&code[0]) && (code[0] & 4) == 0 {
        if code.len() < 2 {
            return None;
        }
        use self::RegMemOpKind::*;
        let kind = [Add, Or, Adc, Sbb, Add, Sub, Xor, Cmp][(code[0] >> 3) as usize & 3];
        let inverse = (code[0] & 2) != 0;
        let wide = (code[0] & 1) != 0;
        Some(Inst {
            pos: 0,
            kind: InstKind::RegMemOp {
                kind,
                inverse,
                wide,
                modrm: code[1],
            },
        })
    } else if (0x88..0x8C).contains(&code[0]) {
        if code.len() < 2 {
            return None;
        }
        let kind = RegMemOpKind::Mov;
        let inverse = (code[0] & 2) != 0;
        let wide = (code[0] & 1) != 0;
        Some(Inst {
            pos: 0,
            kind: InstKind::RegMemOp {
                kind,
                inverse,
                wide,
                modrm: code[1],
            },
        })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Inst {
    pub pos: usize,
    pub kind: InstKind,
}

impl Inst {
    pub fn len(&self) -> usize {
        use self::InstKind::*;
        match self.kind {
            RegisterOp { .. } => 1,
            RegMemOp { .. } => 2,
            Unknown(_) => 1,
        }
    }
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, Clone)]
pub enum InstKind {
    RegisterOp {
        kind: RegisterOpKind,
        register: GeneralRegister,
    },
    RegMemOp {
        kind: RegMemOpKind,
        inverse: bool,
        wide: bool,
        modrm: u8,
    },
    Unknown(u8),
}

impl fmt::Display for InstKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::InstKind::*;
        match self {
            RegMemOp {
                kind,
                inverse,
                wide,
                modrm,
            } => {
                if *inverse {
                    write!(f, "{} ..", kind.name())
                } else {
                    write!(f, "{} ..", kind.name())
                }
            }
            RegisterOp { kind, register } => write!(f, "{} %{}", kind.name(), register.name16()),
            Unknown(byte) => write!(f, "unknown{:02X}", *byte),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RegMemOpKind {
    Add,
    Or,
    Adc,
    Sbb,
    And,
    Sub,
    Xor,
    Cmp,
    Mov,
}

impl RegMemOpKind {
    pub fn name(&self) -> &'static str {
        use self::RegMemOpKind::*;
        match self {
            Add => "add",
            Or => "or",
            Adc => "adc",
            Sbb => "sbb",
            And => "and",
            Sub => "sub",
            Xor => "xor",
            Cmp => "cmp",
            Mov => "mov",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterOpKind {
    Inc,
    Dec,
    Push,
    Pop,
}

impl RegisterOpKind {
    pub fn name(&self) -> &'static str {
        use self::RegisterOpKind::*;
        match self {
            Inc => "inc",
            Dec => "dec",
            Push => "push",
            Pop => "pop",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GeneralRegister {
    Ax,
    Cx,
    Dx,
    Bx,
    Sp,
    Bp,
    Si,
    Di,
}

impl GeneralRegister {
    pub fn from_id(value: u8) -> Self {
        use self::GeneralRegister::*;
        [Ax, Cx, Dx, Bx, Sp, Bp, Si, Di][value as usize]
    }

    pub fn name16(&self) -> &'static str {
        use self::GeneralRegister::*;
        match self {
            Ax => "ax",
            Cx => "cx",
            Dx => "dx",
            Bx => "bx",
            Sp => "sp",
            Bp => "bp",
            Si => "si",
            Di => "di",
        }
    }
}

fn split233(byte: u8) -> (u8, u8, u8) {
    (byte >> 6, (byte >> 3) & 7, byte & 7)
}
