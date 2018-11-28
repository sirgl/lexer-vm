
#[derive(Debug)]
pub enum Opcode {
    CharImm = 0,
    CharCp = 1,
    Match = 2,
    Split = 3,
    Jmp = 4,
    Any = 5,
    RangeImm = 6,
    Range = 7,
    Invert = 8,
    SplitMany = 9,
    OuterLexer = 10,
    Noop = 11
}

impl Opcode {
    fn from_instruction(instruction: u32) -> Opcode {
        return Opcode::from(instruction >> 28)
    }
}

impl From<u32> for Opcode {
    fn from(value: u32) -> Self {
        match value {
            0 => Opcode::CharImm,
            1 => Opcode::CharCp,
            2 => Opcode::Match,
            3 => Opcode::Split,
            4 => Opcode::Jmp,
            5 => Opcode::Any,
            6 => Opcode::RangeImm,
            7 => Opcode::Range,
            8 => Opcode::Invert,
            9 => Opcode::SplitMany,
            10 => Opcode::OuterLexer,
            11 => Opcode::Noop,
            _ => panic!("Bad opcode")
        }
    }
}
