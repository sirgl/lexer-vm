//#![feature(test)]

extern crate core;
extern crate bit_set;

mod vm;
mod opcodes;
mod assembler;
mod disasm;

use vm::Vm;
use assembler::Assembler;
use opcodes::Opcode;
use disasm::decode;
use std::iter::Map;
use disasm::Instruction;

fn main() {
    println!("Hello, world!");
    let mut assembler = Assembler::new();
    assembler.emit_instr(Opcode::CharCp, 12);
    assembler.emit_char_imm('a');
    assembler.emit_char_cp('b');
    assembler.emit_match(2);
    assembler.emit_split(1, 2);
    let data = assembler.finish();
    let instructions = data.code.into_iter().map(|instr| decode(instr));
//    eprintln!("vec = {:?}", &data);
    let vec: Vec<Instruction> = instructions.collect();
//    println!("{}", vec);
    for x in vec {
        println!("{}", x);
    }
//    eprintln!("instructions = {}", vec);
}


//extern crate test;
//
//pub fn add_two(a: i32) -> i32 {
//    a + 2
//}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use test::Bencher;
//
//    #[test]
//    fn it_works() {
//        assert_eq!(4, add_two(2));
//    }
//
//    #[bench]
//    fn bench_add_two(b: &mut Bencher) {
//        b.iter(|| {
//            let mut x = 12;
//            while x < 10000 {
//                x += 23;
//                eprintln!("x = {:?}", x);
//                x -= 22;
//            }
//        });
//    }
//}