use assembler::CodePointer;
use opcodes::Opcode;
use disasm::decode;
use disasm::Instruction;
use std::iter;
use std::borrow::Borrow;
use std::mem::swap;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::Debug;
use bit_set::BitSet;


pub struct Vm {
    code: Vec<u32>,
    constant_pool: Vec<u32>,
    current_threads: BitSet,
    next_threads: BitSet,
    token_types_count: usize
}

impl Vm {
    pub fn new(code: Vec<u32>, constant_pool: Vec<u32>, token_types_count: usize) -> Self {
        let code_len = code.len();
        let mut current_threads = BitSet::with_capacity(token_types_count);
        current_threads.insert(0);
        Vm {
            code,
            constant_pool,
            current_threads,
            next_threads: BitSet::with_capacity(token_types_count),
            token_types_count,
        }
    }
}

impl Vm {
    pub fn tokenize(&mut self, text: &str) -> Vec<TokenRaw> {
        let mut tokens: Vec<TokenRaw> = Vec::new();
        let code = &self.code;
        let mut token_start: u32 = 0;
        let mut token_end: u32 = 0;
        let mut error_mode = false;
        let mut max_matched_token_index = 0;
        let mut matched_indices = BitSet::with_capacity(self.token_types_count);

        for ch in text.chars() {
            // has a match on this iteration
            let mut has_match = false;
            for cp_index in self.current_threads.iter() {
                let instruction = code[cp_index];
                match decode(instruction) {
                        Instruction::CharImm { ch: instr_ch } => {
                            if instr_ch == ch {
                                self.next_threads.insert(cp_index + 1);
                            }
                        }
                        Instruction::CharCp { .. } => unimplemented!(),
                        Instruction::Match { token_type_index } => {
                            if !has_match {
                                matched_indices.clear();
                                has_match = true;
                            }
                        }
                        Instruction::Split { then_instr_index, else_instr_index } => {
                            self.next_threads.insert(then_instr_index as usize);
                            self.next_threads.insert(else_instr_index as usize);
                        }
                        Instruction::Jmp { instr_index } => {
                            self.next_threads.insert(instr_index as usize);
                        }
                }
            }
            if !has_match {
                if matched_indices.is_empty() {
                    // error path
                    error_mode = true;
                } else {
                    // on previous iteration was last match
                    let token = TokenRaw::new(
                        token_end - token_start,
                        max_matched_token_index);
                    // TODO we skipping one char, forbid it
                    tokens.push(token);
                    token_start = token_end
                }
            }
            token_end += ch.len_utf8() as u32;
            swap(&mut self.current_threads, &mut self.next_threads);
        }
        // end token is always of index 1
        tokens.push(TokenRaw::new(0, 1));

        tokens
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct TokenRaw {
    length: u32,
    token_type_index: u16,
}

impl TokenRaw {
    pub fn new(length: u32, token_type_index: u16) -> Self {
        TokenRaw { length, token_type_index }
    }
}

impl Iterator for Vm {
    type Item = TokenRaw;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        Option::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assembler::Assembler;

    #[test]
    fn single_char() {
//        let mut asm = Assembler::new();
//        asm.emit_char_imm('a');
//        asm.emit_match(2);
//        let program_data = asm.finish();
//        let mut vm = Vm::new(program_data.code, program_data.constant_pool);
//        let tokens = vm.tokenize("a");
//        assert_eq!(vec![
//            TokenRaw::new(1, 2),
//            TokenRaw::new(0, 0)
//        ], tokens);
    }
}
