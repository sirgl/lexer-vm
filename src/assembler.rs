use opcodes::Opcode;
use std::collections::HashMap;


pub struct Assembler {
    buffer: Vec<u32>,
    cp_buffer: Vec<u32>,
    cp_value_to_index: HashMap<u32, u16>
}

fn is_valid_inline_code_point(code_point: u32) -> bool {
    return code_point & (0b1111 << 28) == 0;
}

pub type CodePointer = u16;
pub type PoolIndex = u16;

impl Assembler {
    pub fn new() -> Self {
        Assembler {
            buffer: Vec::new(),
            cp_buffer: Vec::new(),
            cp_value_to_index: HashMap::new()
        }
    }

    pub fn emit_instr(&mut self, opcode: Opcode, payload: u32) {
        self.buffer.push(((opcode as u32) << 28) | payload)
    }

    pub fn emit_char_imm(&mut self, ch: char) {
        self.emit_instr(Opcode::CharImm, ch as u32)
    }

    pub fn emit_char_cp(&mut self, ch: char) {
        let pool_index = self.get_pool_index(ch as u32);
        self.emit_instr(Opcode::CharCp, pool_index as u32)
    }

    pub fn emit_match(&mut self, token_type_index: u16) {
        self.emit_instr(Opcode::Match, token_type_index as u32)
    }

    pub fn emit_split(&mut self, then_instr_index: CodePointer, else_instr_index: CodePointer) {
        self.emit_binary_instr(Opcode::Split, then_instr_index, else_instr_index)
    }

    pub fn emit_jmp(&mut self, instr_index: CodePointer) {
        self.emit_instr(Opcode::Jmp, instr_index as u32)
    }

    // 14 bit on every operand
    fn emit_binary_instr(&mut self, opcode: Opcode, first: u16, second: u16) {
        // TODO check 14 bit
        let payload = ((first as u32) << 14) as u32 | (second as u32);
        self.emit_instr(opcode, payload)
    }

    pub fn get_pool_index(&mut self, value: u32) -> PoolIndex {
        match self.cp_value_to_index.get(&value).map(|s| *s) {
            None => {
                let cp_index = self.cp_buffer.len() as PoolIndex;
                self.cp_buffer.push(value);
                self.cp_value_to_index.insert(value, cp_index);
                // TODO check if there is enough place (only 14 bit)
                cp_index
            },
            Some(v) => v,
        }
    }

    pub fn finish(&mut self) -> ProgramData {
        let code = self.buffer.clone();
        let cp_buffer = self.cp_buffer.clone();
        self.buffer.clear();
        self.cp_buffer.clear();
        ProgramData::new(code, cp_buffer)
    }
}

#[derive(Debug)]
pub struct ProgramData {
    pub code: Vec<u32>,
    pub constant_pool: Vec<u32>,
}

impl ProgramData {
    pub fn new(code: Vec<u32>, constant_pool: Vec<u32>) -> Self {
        ProgramData { code, constant_pool }
    }
}