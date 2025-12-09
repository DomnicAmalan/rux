use crate::ast::AST;
use crate::errors::{Error, Result};
use crate::lexer::{Lexer, TokenWithSpan};
use crate::parser::Parser;
use crate::type_checker::TypeChecker;
use crate::optimizer::Optimizer;
use crate::analyzer::DependencyAnalyzer;
use crate::codegen::CodeGenerator;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

pub struct Compiler {
    source_map: HashMap<PathBuf, String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            source_map: HashMap::new(),
        }
    }
    
    pub fn compile_file(&mut self, path: &Path) -> Result<AST> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| Error::parser(
                format!("Failed to read file: {}", e),
                String::new(),
                (0, 0).into(),
            ))?;
        
        self.source_map.insert(path.to_path_buf(), source.clone());
        
        // Lex
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()?;
        
        // Parse
        let mut parser = Parser::new(tokens, source);
        let mut ast = parser.parse()?;
        
        // Type check
        let mut type_checker = TypeChecker::new();
        type_checker.check(&ast)?;
        
        // Analyze dependencies
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.analyze(&ast)?;
        
        // Optimize
        let optimizer = Optimizer::new();
        optimizer.optimize(&mut ast)?;
        
        Ok(ast)
    }
    
    pub fn compile_string(&mut self, source: &str, filename: &str) -> Result<AST> {
        // Lex
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        
        // Parse
        let mut parser = Parser::new(tokens, source.to_string());
        let mut ast = parser.parse()?;
        
        // Type check
        let mut type_checker = TypeChecker::new();
        type_checker.check(&ast)?;
        
        // Analyze dependencies
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.analyze(&ast)?;
        
        // Optimize
        let optimizer = Optimizer::new();
        optimizer.optimize(&mut ast)?;
        
        Ok(ast)
    }
    
    pub fn compile_string_to_rust(&mut self, source: &str, filename: &str) -> Result<String> {
        let ast = self.compile_string(source, filename)?;
        let mut codegen = CodeGenerator::new();
        codegen.generate_rust_code(&ast)
    }
}

pub struct IncrementalCompiler {
    compiler: Compiler,
    file_hashes: HashMap<PathBuf, u64>,
    dependency_graph: HashMap<PathBuf, Vec<PathBuf>>,
}

impl IncrementalCompiler {
    pub fn new() -> Self {
        Self {
            compiler: Compiler::new(),
            file_hashes: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }
    
    pub fn compile_incremental(&mut self, changed_files: &[PathBuf]) -> Result<()> {
        let affected = self.find_affected_files(changed_files);
        
        for file in affected {
            self.compiler.compile_file(&file)?;
            // Update hash
            let hash = self.compute_file_hash(&file)?;
            self.file_hashes.insert(file, hash);
        }
        
        Ok(())
    }
    
    fn find_affected_files(&self, changed: &[PathBuf]) -> Vec<PathBuf> {
        let mut affected = changed.to_vec();
        let mut to_check = changed.to_vec();
        
        while let Some(file) = to_check.pop() {
            if let Some(dependents) = self.dependency_graph.get(&file) {
                for dependent in dependents {
                    if !affected.contains(dependent) {
                        affected.push(dependent.clone());
                        to_check.push(dependent.clone());
                    }
                }
            }
        }
        
        affected
    }
    
    fn compute_file_hash(&self, path: &Path) -> Result<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::parser(
                format!("Failed to read file: {}", e),
                String::new(),
                (0, 0).into(),
            ))?;
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(hasher.finish())
    }
}
