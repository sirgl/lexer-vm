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

pub struct PatchMarker {
    position: CodePointer,
    is_first: bool
}

pub struct SplitManyMarker {
    position: CodePointer
}

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

    pub fn emit_range_imm(&mut self, from: char, to: char) {
        // TODO assert, that from and to fits into 14 bits
        self.emit_binary_instr(Opcode::RangeImm, from as u16, to as u16)
    }

    pub fn emit_split(&mut self, then_instr_index: CodePointer, else_instr_index: CodePointer) -> (PatchMarker, PatchMarker) {
        self.emit_binary_instr(Opcode::Split, then_instr_index, else_instr_index);
        let position = self.last_code_position();
        return (PatchMarker { position, is_first: false }, PatchMarker { position, is_first: true })
    }

    pub fn emit_jmp(&mut self, instr_index: CodePointer) {
        self.emit_instr(Opcode::Jmp, instr_index as u32)
    }

    pub fn emit_split_many(&mut self) -> SplitManyMarker {
        let table_marker = SplitManyMarker { position: self.next_code_position() };
        self.emit_instr(Opcode::SplitMany, 0);
        table_marker
    }

    pub fn patch_split_many(&mut self, marker: &SplitManyMarker, table: Vec<CodePointer>) {
        let table_index = self.cp_buffer.len();
        let old_instruction = self.buffer[marker.position as usize];
        self.buffer[marker.position as usize] = (old_instruction & (0b1111_1111_1111_1111 << 16)) | (table_index as u32);
        self.cp_buffer.extend(table.iter().map(|el| *el as u32));
    }

    pub fn next_code_position(&self) -> CodePointer {
        self.buffer.len() as CodePointer
    }

    fn last_code_position(&self) -> CodePointer {
        (self.buffer.len() - 1) as CodePointer
    }

    pub fn patch_target(&mut self, patch_marker: &PatchMarker, new_pos: CodePointer) {
        let instruction = self.buffer[patch_marker.position as usize];
        if patch_marker.is_first {
            let mask = !0b11_1111_1111_1111;
            let new_instruction = (instruction & mask) | new_pos as u32;
            self.buffer[patch_marker.position as usize] = new_instruction;
        } else {
            let mask = !(0b11_1111_1111_1111 << 14);
            let new_instruction = (instruction & mask) | ((new_pos as u32) << 14);
            self.buffer[patch_marker.position as usize] = new_instruction;
        }
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