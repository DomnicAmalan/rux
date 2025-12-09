pub mod lexer;
pub mod parser;
pub mod ast;
pub mod errors;
pub mod type_checker;
pub mod optimizer;
pub mod analyzer;
pub mod driver;
pub mod codegen;

pub use errors::{Error, Result};
pub use lexer::{Lexer, Token, TokenWithSpan};
pub use parser::Parser;
pub use ast::AST;
pub use type_checker::TypeChecker;
pub use optimizer::Optimizer;
pub use analyzer::DependencyAnalyzer;
pub use driver::{Compiler, IncrementalCompiler};
pub use codegen::CodeGenerator;
