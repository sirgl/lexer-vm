use assembler::CodePointer;
use opcodes::Opcode;
use disasm::decode;
use disasm::Instruction;
use std::{
    fmt::Debug,
    fmt::Formatter,
    fmt,
    mem::swap,
    borrow::Borrow,
    iter,
    cmp::max
};
use bit_set::BitSet;


pub struct Vm {
    code: Vec<u32>,
    constant_pool: Vec<u32>,
    current_threads: BitSet,
    next_threads: BitSet,
    token_types_count: usize,
    max_token_index: u16,
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
            max_token_index: 0,
        }
    }
}

struct MatchResult {
    has_match: bool,
    some_threads_succeed: bool,
}

impl MatchResult {
    pub fn new(has_match: bool, some_threads_succeed: bool) -> Self {
        MatchResult { has_match, some_threads_succeed }
    }
}

const END_TOKEN_INDEX: u16 = 1;
const ERROR_TOKEN_INDEX: u16 = 0;

impl Vm {
    pub fn tokenize(&mut self, text: &str) -> Vec<TokenRaw> {
        let mut tokens: Vec<TokenRaw> = Vec::new();
        let mut token_start: u32 = 0;
        let mut token_end: u32 = 0;
        let mut error_mode = false;
        let mut matched_indices = BitSet::with_capacity(self.token_types_count);

        for ch in text.chars() {
            // has a match on this iteration
            let mut match_res = self.match_char(&mut matched_indices, Option::Some(ch));
            if !match_res.some_threads_succeed {
                if matched_indices.is_empty() {
                    // error path
                    error_mode = true;
                } else {
                    // on previous iteration was last match
                    let token = TokenRaw::new(
                        token_end - token_start,
                        self.max_token_index);
                    matched_indices.clear();
                    self.match_char(&mut matched_indices, Option::Some(ch));
                    tokens.push(token);
                    token_start = token_end
                }
            } else {
                if error_mode {
                    error_mode = false;
                    tokens.push(TokenRaw::new(
                        token_end - token_start,
                        ERROR_TOKEN_INDEX,
                    ));
                }
            }
            token_end += ch.len_utf8() as u32;
            swap(&mut self.current_threads, &mut self.next_threads);
        }
        // tail handling
        self.match_char(&mut matched_indices, Option::None);
        if token_start != token_end && !matched_indices.is_empty() {
            let token_len = token_end - token_start;
            let token_type_index = if error_mode {
                ERROR_TOKEN_INDEX
            } else {
                self.max_token_index
            };
            tokens.push(TokenRaw::new(token_len, token_type_index));

        }
        // end token is always of index 1
        tokens.push(TokenRaw::new(0, END_TOKEN_INDEX));
        tokens
    }


    fn match_char(&mut self, matched_indices: &mut BitSet<u32>, ch: Option<char>) -> MatchResult {
        let code = &self.code;
        let mut has_match = false;
        let mut all_failed = true;
        for code_pointer in self.current_threads.iter() {
            let instruction = code[code_pointer];
            match decode(instruction) {
                Instruction::CharImm { ch: instr_ch } => {
                    if ch.is_some() && instr_ch == ch.unwrap() {
                        self.next_threads.insert(code_pointer + 1);
                        all_failed = false;
                    }
                }
                Instruction::CharCp { .. } => unimplemented!(),
                Instruction::Match { token_type_index } => {
                    if !has_match {
                        matched_indices.clear();
                        has_match = true;
                    }
                    matched_indices.insert(token_type_index as usize);
                    self.max_token_index = std::cmp::max(self.max_token_index, token_type_index)
                }
                Instruction::Split { then_instr_index, else_instr_index } => {
                    let i = then_instr_index as usize;
                    self.next_threads.insert(i);
                    let i1 = else_instr_index as usize;
                    self.next_threads.insert(i1);
                    all_failed = false;
                }
                Instruction::Jmp { instr_index } => {
                    self.next_threads.insert(instr_index as usize);
                    all_failed = false;
                }
            }
        }
        return MatchResult::new(has_match, !all_failed);
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
    use assembler::ProgramData;

//    #[test]
//    fn single_char() {
//        let mut asm = Assembler::new();
//        asm.emit_char_imm('a');
//        asm.emit_match(2);
//        test_vm(asm.finish(), 3, "a", vec![
//            TokenRaw::new(1, 2),
//            TokenRaw::new(0, END_TOKEN_INDEX)
//        ])
//    }
//
//    #[test]
//    fn two_chars() {
//        let mut asm = Assembler::new();
//        asm.emit_char_imm('a');
//        asm.emit_char_imm('a');
//        asm.emit_match(2);
//        test_vm(asm.finish(), 3, "aa", vec![
//            TokenRaw::new(2, 2),
//            TokenRaw::new(0, END_TOKEN_INDEX)
//        ])
//    }
//
//    #[test]
//    fn lex_loop() {
//        let mut asm = Assembler::new();
//        asm.emit_char_imm('a');
//        asm.emit_split(0, 2);
//        asm.emit_match(2);
//        test_vm(asm.finish(), 3, "aaaa", vec![
//            TokenRaw::new(4, 2),
//            TokenRaw::new(0, END_TOKEN_INDEX)
//        ])
//    }

    #[test]
    fn lex_two_tokens() {
        let mut asm = Assembler::new();
        // a | b regex code
        asm.emit_split(1, 3);
        asm.emit_char_imm('a');
        asm.emit_match(2);
        asm.emit_char_imm('b');
        asm.emit_match(3);
        test_vm(asm.finish(), 4, "ab", vec![
            TokenRaw::new(1, 2),
            TokenRaw::new(1, 3),
            TokenRaw::new(0, END_TOKEN_INDEX)
        ])
    }

    fn test_vm(program_data: ProgramData, token_types_count: usize, text: &str, expected_tokens: Vec<TokenRaw>) {
        let mut vm = Vm::new(program_data.code, program_data.constant_pool, 3);
        let tokens = vm.tokenize(text);
        assert_eq!(expected_tokens, tokens);
    }
}
