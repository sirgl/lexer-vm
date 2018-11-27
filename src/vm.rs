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
    cmp::max,
};
use bit_set::BitSet;


pub struct Vm {
    code: Vec<u32>,
    constant_pool: Vec<u32>,
    // TODO put threads to LexerSession
    current_threads: BitSet,
    next_threads: BitSet,
    token_types_count: usize,
}


impl Vm {
    pub fn new(code: Vec<u32>, constant_pool: Vec<u32>, token_types_count: usize) -> Self {
        let code_len = code.len();
        let mut current_threads = BitSet::with_capacity(token_types_count);
        Vm {
            code,
            constant_pool,
            current_threads,
            next_threads: BitSet::with_capacity(token_types_count),
            token_types_count,
        }
    }
}


struct LexingSession<'a, 'b> {
    vm: &'a mut Vm,
    token_start: u32,
    text: &'b str,
    position: usize,
    is_end: bool,
}


#[derive(Copy, Clone)]
struct BestToken {
    token_index: u16,
    end_position: usize,
}

impl BestToken {
    pub fn new(token_index: u16, end_position: usize) -> Self {
        BestToken { token_index, end_position }
    }
}

impl<'a, 'b> LexingSession<'a, 'b> {
    pub fn new(vm: &'a mut Vm, text: &'b str) -> Self {
        LexingSession {
            vm,
            token_start: 0,
            text,
            position: 0,
            is_end: false
        }
    }

    fn token_len(&self) -> u32 {
        return self.position as u32 - self.token_start;
    }
}

impl<'a, 'b> Iterator for LexingSession<'a, 'b> {
    type Item = TokenRaw;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.is_end {
            return Option::None;
        }
        let mut best : Option<BestToken> = Option::None;
        let mut error_mode = false;
        self.vm.add_thread(0, false);
        let (_, text) = self.text.split_at(self.position);
        if self.position == self.text.len() {
            self.is_end = true;
            return Some(TokenRaw::new(0, END_TOKEN_INDEX));
        }
        eprintln!("Lexing at position {:?}", self.position);
        let mut result = Option::None;
        for ch in text.chars() {
            eprintln!("ch = {:?}", ch);
            eprintln!("self.current_threads = {:?}", self.vm.current_threads);
            let match_res = self.vm.match_char(ch);
            // trying update BestToken
            if let Some(new_max_index) = match_res.max_matched_token_index {
                if let Some(max_length_token) = best {
                    if max_length_token.token_index <= new_max_index {
                        let new_max_len_info = BestToken::new(new_max_index, self.position);
                        best = Some(new_max_len_info)
                    }
                } else {
                    let info = BestToken::new(new_max_index, self.position);
                    best = Option::Some(info);
                }
            }
            if match_res.all_threads_failed {
                if !error_mode {
                    if let Some(best_val) = best {
                        result = Option::Some(TokenRaw::new(self.token_len(), best_val.token_index));
                    } else {
                        error_mode = true;
                    };
                }
            } else {
                // something matched?
                // TODO actually, returning back from error mode must happen, when something really matched
                if error_mode {
                    result = Option::Some(TokenRaw::new(self.token_len(), ERROR_TOKEN_INDEX));
                    error_mode = false;
                }
            }
            self.position += ch.len_utf8();
            if result.is_some() {
                self.token_start = self.position as u32;
                break;
            }
            self.vm.current_threads.clear();
            swap(&mut self.vm.current_threads, &mut self.vm.next_threads);
        }
        if self.position != self.token_start as usize {
            let pos = self.position as u32;
            let ts = self.token_start;
            self.token_start = pos;
            if let Some(best_val) = best {
                result = Option::Some(TokenRaw::new(self.token_len(), best_val.token_index));
            } else {
                result = Option::Some(TokenRaw::new(self.token_len(), ERROR_TOKEN_INDEX))
            };
//            self.position = self.token_start as usize;
        }
        result
    }
}


struct MatchResult {
    all_threads_failed: bool,
    max_matched_token_index: Option<u16>,
}

impl MatchResult {
    pub fn new(all_threads_failed: bool, max_matched_token_index: Option<u16>) -> Self {
        MatchResult { all_threads_failed, max_matched_token_index }
    }
}

const END_TOKEN_INDEX: u16 = 1;
const ERROR_TOKEN_INDEX: u16 = 0;

impl Vm {
    /// handles all not immediately advancing instructions
    fn add_thread(&mut self, pc: CodePointer, to_next: bool) {
        let instruction = self.code[pc as usize];
        match decode(instruction) {
            Instruction::Split { then_instr_index, else_instr_index } => {
                self.add_thread(then_instr_index, to_next);
                self.add_thread(else_instr_index, to_next);
            }
            Instruction::Jmp { instr_index } => self.add_thread(instr_index, to_next),
            _ => {
                if to_next {
                    self.next_threads.insert(pc as usize);
                } else {
                    self.current_threads.insert(pc as usize);
                }
            }
        }
    }

    fn match_char(&mut self, ch: char) -> MatchResult {
        let mut all_failed = true;
        let mut next: Option<CodePointer> = Option::None;
        let mut max_matched_token_index = Option::None;
        for code_pointer in self.current_threads.iter() {
            let instruction = self.code[code_pointer];
            match decode(instruction) {
                // must handle here only strictly advancing operations
                Instruction::CharImm { ch: instr_ch } => {
                    if instr_ch == ch {
                        let new_code_pointer = (code_pointer + 1) as CodePointer;
                        next = Option::Some(new_code_pointer);
                        all_failed = false;
                    }
                }
                Instruction::Match { token_type_index } => {
                    max_matched_token_index = max_matched_token_index
                        .map(|value| std::cmp::max(value, token_type_index));
                }
                _ => {}
            }
        }
        match next {
            Some(cp) => self.add_thread(cp, true),
            None => {}
        }
        return MatchResult::new(all_failed, max_matched_token_index);
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

    #[test]
    fn single_char() {
        let mut asm = Assembler::new();
        asm.emit_char_imm('a');
        asm.emit_match(2);
        test_vm(asm.finish(), "a", vec![
            TokenRaw::new(1, 2),
            TokenRaw::new(0, END_TOKEN_INDEX)
        ])
    }

    #[test]
    fn two_chars() {
        let mut asm = Assembler::new();
        asm.emit_char_imm('a');
        asm.emit_char_imm('a');
        asm.emit_match(2);
        test_vm(asm.finish(), "aa", vec![
            TokenRaw::new(2, 2),
            TokenRaw::new(0, END_TOKEN_INDEX)
        ])
    }

    #[test]
    fn lex_loop() {
        let mut asm = Assembler::new();
        asm.emit_char_imm('a');
        asm.emit_split(0, 2);
        asm.emit_match(2);
        test_vm(asm.finish(), "aaaa", vec![
            TokenRaw::new(4, 2),
            TokenRaw::new(0, END_TOKEN_INDEX)
        ])
    }

    #[test]
    fn lex_two_tokens() {
        let mut asm = Assembler::new();
        // a | b regex code
        asm.emit_split(1, 3);
        asm.emit_char_imm('a');
        asm.emit_match(2);
        asm.emit_char_imm('b');
        asm.emit_match(3);
        test_vm(asm.finish(), "ab", vec![
            TokenRaw::new(1, 2),
            TokenRaw::new(1, 3),
            TokenRaw::new(0, END_TOKEN_INDEX)
        ])
    }

    fn test_vm(program_data: ProgramData, text: &str, expected_tokens: Vec<TokenRaw>) {
        let mut vm = Vm::new(program_data.code, program_data.constant_pool, 3);
        let tokens: Vec<TokenRaw> = LexingSession::new(&mut vm, text).take(4).collect();
//        let tokens = vm.tokenize(text);
        assert_eq!(expected_tokens, tokens);
    }
}
