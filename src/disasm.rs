use assembler::ProgramData;
use opcodes::Opcode;
use assembler::PoolIndex;
use assembler::CodePointer;
use std::char::from_u32;
use std::fmt;
use std::fmt::Formatter;

pub fn decode<'a>(code: u32) -> Instruction {
    let opcode = Opcode::from(code >> 28);
    let payload = trim_tag(code);
    match opcode {
        Opcode::CharImm => Instruction::CharImm { ch: from_u32(payload).unwrap() },
        Opcode::CharCp => Instruction::CharCp { ch_index: payload as PoolIndex },
        Opcode::Match => Instruction::Match { token_type_index: payload as u16 },
        Opcode::Split => {
            let (left, right) = decode_binary(payload);
            Instruction::Split { then_instr_index: left, else_instr_index: right }
        },
        Opcode::Jmp => Instruction::Jmp { instr_index: payload as CodePointer },
        Opcode::SplitMany => Instruction::SplitMany  { table_index: payload as u16 },
        _ => unimplemented!("code not implemented yet")
    }
}

fn decode_binary(payload: u32) -> (u16, u16) {
    let mask = 0b0011_1111_1111_1111;
    let left = (payload >> 14) & mask;
    let right = payload & mask;
    (left as u16, right as u16)
}

fn trim_tag(tagged: u32) -> u32 {
    let mask = !(0b1111 << 28);
    return tagged & mask;
}


#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    CharImm { ch: char },
    CharCp { ch_index: PoolIndex },
    /// token_type_index is 
    Match { token_type_index: u16 },
    Split { then_instr_index: CodePointer, else_instr_index: CodePointer },
    SplitMany { table_index: u16 },
    Jmp { instr_index: CodePointer },
}

impl fmt::Display for Instruction {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> fmt::Result {
        match self {
            Instruction::CharImm { ch } => { write!(f, "char_imm ch: {}", ch) }
            Instruction::CharCp { ch_index } => { write!(f, "char_cp ch_index: {}", ch_index) }
            Instruction::Match { token_type_index } => { write!(f, "match token_type_index: {}", token_type_index) }
            Instruction::Split { then_instr_index, else_instr_index } =>
                { write!(f, "split then_instr_index: {} else_instr_index: {}, ", then_instr_index, else_instr_index) }
            Instruction::Jmp { instr_index } => { write!(f, "jmp instr_index: {}", instr_index) }
            Instruction::SplitMany { table_index } => { write!(f, "split_many table_index: {}", table_index) }
        }
    }
}