use vm::Vm;
use ast::Expr;
use assembler::Assembler;
use assembler::ProgramData;
use ast::LexerDefinition;
use ast::TokenDefinition;

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

    pub fn compile_lexer(&mut self, lexer_definition: &LexerDefinition) -> Vm {
        self.generate_lexer(lexer_definition);
        self.get_vm()
    }

    pub fn get_vm(&mut self) -> Vm {
        let program_data = self.get_prog_data();
        Vm::new(program_data.code, program_data.constant_pool)
    }

    pub fn get_prog_data(&mut self) -> ProgramData {
        self.asm.finish()
    }

    pub fn generate_lexer(&mut self, definition: &LexerDefinition) {
        let indices: Vec<u16> = definition.tokens.iter()
            .map(|el| el.index)
            .collect();
        let expressions : Vec<Expr> = definition.tokens.iter()
            .map(|el| el.expr.clone())
            .collect();
        self.generate_or(&expressions, Some(indices))
    }

    pub fn generate_token_expr(&mut self, definition: &TokenDefinition) {
        self.generate(&definition.expr);
        self.asm.emit_match(definition.index)
    }

    pub fn generate(&mut self, expr: &Expr) {
        match expr {
            Expr::Single { ch } => {
                self.asm.emit_char_imm(*ch);
            },
            Expr::Range { from, to } => {
                self.asm.emit_range_imm(*from, *to);
            },
            Expr::Or { variants } => {
                self.generate_or(variants, None)
            },
            Expr::Seq { exprs } => {
                for e in exprs {
                    self.generate(e);
                }
            },
            Expr::Loop { expr } => {
                let next_position = self.asm.next_code_position();
                let (_, forward) = self.asm.emit_split(next_position, 0);
//                self.generate(expr);
//                self.asm.emit_split();
//                self.asm.emit_noop()
                // TODO
            }
        }
    }

    fn generate_or(&mut self, variants: &Vec<Expr>, type_indices: Option<Vec<u16>>) -> () {
        match variants.len() {
            1 => {
                self.generate(&variants[0]);
                if let Some(ref indices) = type_indices {
                    self.asm.emit_match(indices[0])
                }
            }
            2 => {
                let left = &variants[0];
                let right = &variants[1];
                self.generate_split(left, right, type_indices)
            }
            _ => {
                let mut positions = Vec::new();
                let marker = self.asm.emit_split_many();
                for (index, variant) in variants.iter().enumerate() {
                    positions.push(self.asm.next_code_position());
                    self.generate(variant);
                    if let Some(ref indices) = type_indices {
                        self.asm.emit_match(indices[index])
                    }
                }
                self.asm.patch_split_many(&marker, positions);
            }
        }
    }

    fn generate_split(&mut self, left: &Expr, right: &Expr, type_indices: Option<Vec<u16>>) {
        let (left_patch, right_patch) = self.asm.emit_split(0, 0);
        let left_target = self.asm.next_code_position();
        self.generate(left);
        if let Some(ref indices) = type_indices {
            self.asm.emit_match(indices[0])
        }
        let right_target = self.asm.next_code_position();
        self.generate(right);
        if let Some(ref indices) = type_indices {
            self.asm.emit_match(indices[1])
        }
        self.asm.patch_target(&left_patch, left_target);
        self.asm.patch_target(&right_patch, right_target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use disasm::decode;
    use disasm::Instruction;
    use disasm::Instruction::*;


    #[test]
    fn single_char() {
        let mut compiler = Compiler::new();
        let expr = Expr::Seq { exprs: vec![
            Expr::Single { ch: 'a' },
            Expr::Single { ch: 'b' },
            Expr::Or {
                variants: vec![
                    Expr::Single { ch: 'c' },
                    Expr::Single { ch: 'd' }
                ]
            }
        ] };
        let lexer_definition = LexerDefinition {
            tokens: vec![
                TokenDefinition {
                    expr,
                    index: 2,
                    name: "foo".to_string()
                }
            ]
        };
        check_compiler(&mut compiler, &lexer_definition, vec![
            CharImm { ch: 'a' },
            CharImm { ch: 'b' },
            Split { then_instr_index: 3, else_instr_index: 4 },
            CharImm { ch: 'c' },
            CharImm { ch: 'd' },
            Match { token_type_index: 2 }
        ], vec![]);
    }

    #[test]
    fn range() {
        let mut compiler = Compiler::new();
        let expr = Expr::Range {
            from: 'a',
            to: 'z',
        };
        let lexer_definition = LexerDefinition {
            tokens: vec![
                TokenDefinition {
                    expr,
                    index: 2,
                    name: "foo".to_string()
                }
            ]
        };
        check_compiler(&mut compiler, &lexer_definition, vec![
            RangeImm { from: 'a', to: 'z' },
            Match { token_type_index: 2 }
        ], vec![]);
    }

    #[test]
    fn multiple_tokens(){
        let mut compiler = Compiler::new();
        let first = Expr::Seq { exprs: vec![
            Expr::Single { ch: 'a' },
            Expr::Single { ch: 'b' },
        ]};
        let second = Expr::Seq { exprs: vec![
            Expr::Single { ch: 'c' },
            Expr::Single { ch: 'd' },
        ]};
        let lexer_definition = LexerDefinition {
            tokens: vec![
                TokenDefinition {
                    expr: first,
                    index: 2,
                    name: "foo".to_string()
                },
                TokenDefinition {
                    expr: second,
                    index: 3,
                    name: "bar".to_string()
                }
            ]
        };
        check_compiler(&mut compiler, &lexer_definition, vec![
            Split { then_instr_index: 1, else_instr_index: 4 },
            CharImm { ch: 'a' },
            CharImm { ch: 'b' },
            Match { token_type_index: 2 },
            CharImm { ch: 'c' },
            CharImm { ch: 'd' },
            Match { token_type_index: 3 }
        ], vec![]);
    }

    fn check_compiler(compiler: &mut Compiler, lexer_definition: &LexerDefinition, expected: Vec<Instruction>, pool: Vec<u32>) {
        compiler.generate_lexer(&lexer_definition);
        let prog_data = compiler.get_prog_data();
        let instructions: Vec<Instruction> = prog_data.code.iter()
            .map(|instr| decode(*instr))
            .collect();
        assert_eq!(expected, instructions);
        assert_eq!(pool, prog_data.constant_pool);
    }
}