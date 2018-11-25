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


#[derive(Clone)]
struct Thread {
    /// Layout:  alive flag | inverted flag | pc |
    storage: u16
}

impl Debug for Thread {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Thread alive: {}, inverted: {}, pc: {}", self.is_alive(), self.is_inverted(), self.code_pointer())
    }
}

impl fmt::Display for Thread {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Thread alive: {}, inverted: {}, pc: {}", self.is_alive(), self.is_inverted(), self.code_pointer())
    }
}

impl Thread {
    /// Creates dead, not inverted thread
    pub fn new(pc: CodePointer) -> Self {
        Thread { storage: pc }
    }

    fn is_alive(&self) -> bool {
        return 0b1000_0000_0000_0000 & self.storage != 0;
    }

    fn set_alive(&mut self) {
        self.storage |= 0b1000_0000_0000_0000
    }

    fn code_pointer(&self) -> CodePointer {
        return 0b0011_1111_1111_1111 & self.storage;
    }

    fn set_code_pointer(&mut self, cp: CodePointer) {
        self.storage = (0b1100_0000_0000_0000 & self.storage) | cp
    }

    fn run(&mut self, cp: CodePointer) {
        self.set_alive();
        self.set_code_pointer(cp);
    }

    fn is_inverted(&self) -> bool {
        return 0b0100_0000_0000_0000 & self.storage != 0;
    }
}

pub struct Vm {
    code: Vec<u32>,
    constant_pool: Vec<u32>,
    current_threads: Vec<Thread>,
    next_threads: Vec<Thread>,
}

impl Vm {
    pub fn new(code: Vec<u32>, constant_pool: Vec<u32>) -> Self {
        let code_len = code.len();
        let mut current_threads = Vm::create_threads(code_len);
        current_threads[0].set_alive();
        Vm {
            code,
            constant_pool,
            current_threads,
            next_threads: Vm::create_threads(code_len),
        }
    }

    fn create_threads(size: usize) -> Vec<Thread> {
        let threads: Vec<Thread> = iter::repeat(Thread::new(0)).take(size).collect();
        threads
    }
}

impl Vm {
    pub fn tokenize(&mut self, text: &str) -> Vec<TokenRaw> {
        let mut tokens: Vec<TokenRaw> = Vec::new();
        let code = &self.code;
        let mut token_start: u32 = 0;
        let mut token_end: u32 = 0;
        for ch in text.chars() {
            token_end += ch.len_utf8() as u32;
            {
                let threads: &Vec<Thread> = &self.current_threads;
                let mut is_match = false;
                eprintln!("current threads = {:?}", threads);
                for thread in threads {
                    eprintln!("thread = {}", thread);
                    if !thread.is_alive() {
                        continue;
                    }
                    let code_pointer: CodePointer = thread.code_pointer();
                    eprintln!("code_pointer = {:?}", code_pointer);
                    let instruction = code[code_pointer as usize];
                    match decode(instruction) {
                        Instruction::CharImm { ch: instr_ch } => {
                            if instr_ch == ch {
                                self.next_threads[(code_pointer + 1) as usize]
                                    .run(code_pointer + 1);
                            }
                        }
                        Instruction::CharCp { .. } => unimplemented!(),
                        Instruction::Match { token_type_index } => {
                            let token = TokenRaw { token_type_index, length: token_end - token_start };
                            is_match = true;
                            // TODO check, that it is not error token
                            tokens.push(token);
                            token_start = token_end
                        }
                        Instruction::Split { then_instr_index, else_instr_index } => {
                            self.next_threads[thread.storage as usize]
                                .run(then_instr_index);
                            self.next_threads[thread.storage as usize]
                                .run(else_instr_index);
                        }
                        Instruction::Jmp { instr_index } => {
                            self.next_threads[thread.storage as usize]
                                .run(instr_index);
                        }
                    }
                }
                if !is_match && !self.next_threads.iter().any(|el| el.is_alive()) {
                    panic!("error!")
                }
            }
            swap(&mut self.current_threads, &mut self.next_threads);
            // TODO not clear, but kill all threads
//            self.next_threads.clear();
        }
        // end token is always of index 0
        tokens.push(TokenRaw::new(0, 0));

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
        let mut asm = Assembler::new();
        asm.emit_char_imm('a');
        asm.emit_match(2);
        let program_data = asm.finish();
        let mut vm = Vm::new(program_data.code, program_data.constant_pool);
        let tokens = vm.tokenize("a");
        assert_eq!(vec![
            TokenRaw::new(1, 2),
            TokenRaw::new(0, 0)
        ], tokens);
    }
}
