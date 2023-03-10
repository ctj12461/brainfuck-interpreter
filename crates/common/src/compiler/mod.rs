mod instruction;
mod lexer;
mod parser;

pub use instruction::{Instruction, InstructionList};
use lexer::build_token_list;
use parser::Parser;
pub use parser::{AddUntilZeroArg, ParseError};

pub type Result<T> = std::result::Result<T, ParseError>;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, code: &str) -> Result<InstructionList> {
        let token_list = build_token_list(code);
        let parser = Parser::new();
        let syntax_tree = parser.parse(token_list)?;
        let instruction_list = InstructionList::compile(syntax_tree);
        Ok(instruction_list)
    }
}
