use vm::Vm;
use ast::Expr;
use assembler::Assembler;
use assembler::ProgramData;

struct Compiler {
    asm: Assembler
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { asm: Assembler::new() }
    }

    pub fn compile(&mut self, expr: &Expr) -> Vm {
        let program_data = self.get_result();
        Vm::new(program_data.code, program_data.constant_pool)
    }

    pub fn generate(&mut self, expr: &Expr) {
        match expr {
            Expr::Single { ch } => {
                self.asm.emit_char_imm(*ch);
            },
            Expr::Range { from, to } => {},
            Expr::OrTable { .. } => {unimplemented!()},
            Expr::Or { left, right } => {
                let (left_patch, right_patch) = self.asm.emit_split(0, 0);
                let left_target = self.asm.next_code_position();
                self.generate(left);
                let right_target = self.asm.next_code_position();
                self.generate(right);
                self.asm.patch_target(&left_patch, left_target);
                self.asm.patch_target(&right_patch, right_target);
            },
            Expr::Seq { exprs } => {
                for e in exprs {
                    self.generate(e);
                }
            },
        }
    }

    pub fn get_result(&mut self) -> ProgramData {
        self.asm.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use disasm::decode;
    use disasm::Instruction;

    #[test]
    fn single_char() {
        let mut compiler = Compiler::new();
        let expr = Expr::Seq { exprs: vec![
            Expr::Single { ch: 'a' },
            Expr::Single { ch: 'b' },
            Expr::Or {
                left: Box::new(Expr::Single { ch: 'c' }),
                right: Box::new(Expr::Single { ch: 'd' }),
            }
        ] };
        compiler.generate(&expr);
        let prog_data = compiler.get_result();
        let instructions: Vec<Instruction> = prog_data.code.iter()
            .map(|instr| decode(*instr))
            .collect();
        eprintln!("instructions = {:?}", instructions);

    }
}