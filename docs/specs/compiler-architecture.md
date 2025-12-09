# RUX Compiler Architecture

## Overview

The RUX compiler transforms `.rsx` files into optimized binaries using a hybrid approach: a custom frontend for RUX-specific optimizations combined with LLVM as the backend for proven, high-quality code generation. This combines Svelte's compile-time optimizations with LLVM's excellent optimization passes and broad target support.

**Related Documentation**: 
- [Endpoint Compilers](endpoint-compilers.md) - Compilation targets, performance characteristics, and platform-specific compilation strategies
- [Syntax Analysis](syntax-analysis.md) - Analysis of rendering syntax alternatives and recommendations
- [SwiftUI Patterns Analysis](swiftui-patterns-analysis.md) - Detailed recommendations on SwiftUI patterns to adopt or avoid
- [Large Codebase Design](large-codebase-design.md) - Design philosophy for scalability and resource-constrained devices

## 1. Compiler Pipeline

### 1.1 Pipeline Stages

```
.rsx file
  ↓
[RUX Frontend Compiler - Built from scratch]
  ├─ Lexer (RUX built)
  ├─ Parser (RUX built)
  ├─ Type Checker (RUX built)
  └─ Analyzer (RUX built)
  ↓
[RUX Optimizer - RUX-specific optimizations]
  ├─ Component inlining
  ├─ Signal optimization
  ├─ Virtual tree optimization
  └─ UI-specific optimizations
  ↓
[RUX Code Generator - Built from scratch]
  ↓
[LLVM IR Generator]
  ↓
[LLVM Backend]
  ├─ LLVM optimization passes
  ├─ Target code generation
  └─ Binary output
  ↓
Multiple targets:
  ├─ Native binary (desktop/mobile)
  ├─ WASM module (web)
  └─ Static library (platform interop)
```

**Compiler Strategy**: RUX uses a hybrid approach combining custom frontend with LLVM backend:
- **RUX Frontend**: Built from scratch - handles parsing, type checking, and RUX-specific optimizations
- **LLVM Backend**: Proven, mature code generation with excellent optimization quality
- **Best of Both Worlds**: RUX-specific optimizations + LLVM's battle-tested code generation
- **Broad Target Support**: LLVM provides support for all major platforms and architectures

See [Endpoint Compilers](endpoint-compilers.md) for details on each compilation target and their performance characteristics.

### 1.2 Stage Overview

1. **Lexing**: Convert source to tokens (RUX built lexer)
2. **Parsing**: Build abstract syntax tree (RUX built parser)
3. **Analysis**: Type checking, dependency analysis (RUX built analyzer)
4. **RUX-Specific Optimization**: Component inlining, signal optimization, virtual tree optimization (RUX built optimizer)
5. **Code Generation**: Generate LLVM IR from RUX AST (RUX built code generator)
6. **LLVM Optimization**: LLVM optimization passes (inlining, loop optimization, vectorization, etc.)
7. **LLVM Code Generation**: Target-specific code generation and binary output (LLVM backend)

**Hybrid Approach**: RUX frontend handles domain-specific optimizations, LLVM backend handles proven low-level optimizations and code generation.

## 2. Parser for .rsx Syntax

### 2.1 Lexer

Tokenizing `.rsx` source code.

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Identifiers
    Ident(String),
    
    // Keywords
    Fn, If, Else, For, In, Let, Mut, Return,
    
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    
    // Operators
    Plus, Minus, Star, Slash, Eq, EqEq, Ne, Lt, Gt,
    
    // Punctuation
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Semicolon, Colon, Dot, Arrow,
    
    // JSX
    JSXOpen, JSXClose, JSXSelfClose,
    JSXOpenTag(String), JSXCloseTag(String),
    
    // Special
    Eof,
}
```

### 2.2 Parser

Recursive descent parser for `.rsx` syntax.

```rust
struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn parse_component(&mut self) -> Result<Component> {
        self.expect(Token::Fn)?;
        let name = self.parse_identifier()?;
        let params = self.parse_params()?;
        self.expect(Token::Arrow)?;
        let return_type = self.parse_type()?;
        let body = self.parse_block()?;
        
        Ok(Component {
            name,
            params,
            return_type,
            body,
        })
    }
    
    fn parse_jsx_element(&mut self) -> Result<JSXElement> {
        self.expect(Token::JSXOpen)?;
        let tag = self.parse_jsx_tag()?;
        let props = self.parse_jsx_props()?;
        
        if self.check(Token::JSXSelfClose) {
            self.advance();
            return Ok(JSXElement::SelfClosing { tag, props });
        }
        
        self.expect(Token::GT)?;
        let children = self.parse_jsx_children()?;
        self.expect(Token::JSXClose)?;
        self.expect(Token::JSXCloseTag(tag.clone()))?;
        self.expect(Token::GT)?;
        
        Ok(JSXElement::WithChildren { tag, props, children })
    }
}
```

### 2.3 Error Recovery

Error recovery for better error messages.

```rust
impl Parser {
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            
            match self.peek().token_type {
                TokenType::Fn | TokenType::Let | TokenType::If => return,
                _ => {}
            }
            
            self.advance();
        }
    }
}
```

## 3. AST Transformation Pipeline

### 3.1 AST Structure

Abstract syntax tree representation.

```rust
#[derive(Debug, Clone)]
enum Expr {
    Literal(Literal),
    Variable(String),
    Binary { left: Box<Expr>, op: Op, right: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    JSXElement(JSXElement),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone)]
enum Stmt {
    Let { name: String, value: Expr },
    Expr(Expr),
    Return(Option<Expr>),
    If { condition: Expr, then: Box<Stmt>, else_: Option<Box<Stmt>> },
    For { var: String, iter: Expr, body: Box<Stmt> },
}
```

### 3.2 Transformations

AST transformations for optimization.

```rust
trait Transform {
    fn transform(&self, ast: &mut AST) -> Result<()>;
}

struct InlineConstants;
struct DeadCodeElimination;
struct HoistStaticElements;
struct FlattenNestedComponents;
```

### 3.3 Visitor Pattern

Visitor pattern for AST traversal.

```rust
trait Visitor {
    fn visit_expr(&mut self, expr: &mut Expr) -> Result<()>;
    fn visit_stmt(&mut self, stmt: &mut Stmt) -> Result<()>;
    fn visit_component(&mut self, component: &mut Component) -> Result<()>;
}
```

## 4. Compile-Time Optimizations

### 4.1 Dead Code Elimination

Removing unused code.

```rust
struct DeadCodeEliminator {
    used_symbols: HashSet<String>,
}

impl Visitor for DeadCodeEliminator {
    fn visit_stmt(&mut self, stmt: &mut Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, .. } => {
                if !self.used_symbols.contains(name) {
                    *stmt = Stmt::Nop; // Mark for removal
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 4.2 Constant Folding

Evaluating constants at compile time.

```rust
fn fold_constants(expr: &mut Expr) -> Result<()> {
    match expr {
        Expr::Binary { left, op, right } => {
            if let (Expr::Literal(l), Expr::Literal(r)) = (&**left, &**right) {
                if let Some(result) = evaluate_op(l, op, r) {
                    *expr = Expr::Literal(result);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
```

### 4.3 Inlining

Inlining small functions and components.

```rust
struct Inliner {
    inline_threshold: usize,
}

impl Inliner {
    fn should_inline(&self, component: &Component) -> bool {
        component.body.size() < self.inline_threshold
    }
    
    fn inline_component(&mut self, call: &mut JSXElement, component: &Component) {
        // Replace component call with component body
    }
}
```

### 4.4 Template Pre-Compilation

Pre-compiling templates to reduce runtime work.

```rust
fn precompile_template(template: &JSXElement) -> CompiledTemplate {
    let mut instructions = Vec::new();
    
    compile_element(template, &mut instructions);
    
    CompiledTemplate { instructions }
}
```

## 5. Dependency Collection

### 5.1 Dependency Analysis

Analyzing component and signal dependencies.

```rust
struct DependencyAnalyzer {
    dependencies: HashMap<String, HashSet<String>>,
    signals: HashSet<String>,
    components: HashSet<String>,
}

impl DependencyAnalyzer {
    fn analyze(&mut self, ast: &AST) {
        self.traverse(ast);
        self.build_dependency_graph();
    }
    
    fn build_dependency_graph(&mut self) {
        // Build graph of dependencies
    }
}
```

### 5.2 Reactive Dependency Tracking

Tracking reactive dependencies for fine-grained updates.

```rust
fn track_reactive_dependencies(expr: &Expr) -> HashSet<SignalId> {
    let mut signals = HashSet::new();
    
    match expr {
        Expr::Call { callee, .. } if is_signal_call(callee) => {
            signals.insert(extract_signal_id(callee));
        }
        Expr::Binary { left, right, .. } => {
            signals.extend(track_reactive_dependencies(left));
            signals.extend(track_reactive_dependencies(right));
        }
        _ => {}
    }
    
    signals
}
```

## 6. Code Generation

### 6.1 Rust Code Generation

Generating Rust code from AST.

```rust
struct CodeGenerator {
    output: String,
    indent: usize,
}

impl CodeGenerator {
    fn generate_component(&mut self, component: &Component) {
        writeln!(self.output, "fn {}(", component.name);
        self.generate_params(&component.params);
        writeln!(self.output, ") -> Element {{");
        self.indent += 1;
        self.generate_body(&component.body);
        self.indent -= 1;
        writeln!(self.output, "}}");
    }
    
    fn generate_jsx(&mut self, element: &JSXElement) {
        match element {
            JSXElement::WithChildren { tag, props, children } => {
                writeln!(self.output, "{}::new()", tag);
                self.generate_props(props);
                for child in children {
                    self.generate_expr(child);
                }
            }
            _ => {}
        }
    }
}
```

### 6.2 Optimization in Codegen

Applying optimizations during code generation.

```rust
impl CodeGenerator {
    fn generate_optimized(&mut self, ast: &OptimizedAST) {
        // Generate code with optimizations applied
        match ast {
            OptimizedAST::InlinedComponent { body, .. } => {
                self.generate_body(body); // Direct generation, no function call
            }
            _ => {}
        }
    }
}
```

## 7. Incremental Compilation

### 7.1 Change Detection

Detecting changes for incremental compilation.

```rust
struct IncrementalCompiler {
    file_hashes: HashMap<PathBuf, u64>,
    dependency_graph: DependencyGraph,
}

impl IncrementalCompiler {
    fn compile_incremental(&mut self, changed_files: &[PathBuf]) -> Result<()> {
        let affected = self.find_affected_files(changed_files);
        
        for file in affected {
            self.compile_file(&file)?;
        }
        
        Ok(())
    }
    
    fn find_affected_files(&self, changed: &[PathBuf]) -> Vec<PathBuf> {
        // Find all files that depend on changed files
        self.dependency_graph.transitive_deps(changed)
    }
}
```

### 7.2 Caching

Caching compilation results.

```rust
struct CompilationCache {
    cache: HashMap<PathBuf, CachedResult>,
}

struct CachedResult {
    hash: u64,
    output: Vec<u8>,
    dependencies: Vec<PathBuf>,
}
```

## 8. Error Messages

### 8.1 Rust-Style Error Messages

High-quality error messages inspired by Rust.

```rust
struct ErrorReporter {
    source: SourceMap,
}

impl ErrorReporter {
    fn report_error(&self, error: &CompileError) {
        eprintln!("error[E{:04}]: {}", error.code, error.message);
        eprintln!("  --> {}:{}:{}", error.file, error.line, error.column);
        eprintln!("   |");
        eprintln!("{} | {}", error.line, error.source_line);
        eprintln!("   | {}^", " ".repeat(error.column), error.message);
        eprintln!("   |");
        eprintln!("   = help: {}", error.help);
    }
}
```

### 8.2 Suggestions

Providing helpful suggestions.

```rust
fn suggest_fix(error: &CompileError) -> Option<String> {
    match error.kind {
        ErrorKind::UnknownComponent { name } => {
            let suggestions = find_similar_components(name);
            Some(format!("Did you mean: {}?", suggestions.join(", ")))
        }
        ErrorKind::TypeMismatch { expected, found } => {
            Some(format!("Expected {}, found {}", expected, found))
        }
        _ => None
    }
}
```

## 9. Macro Expansion

### 9.1 Macro Processing

Processing declarative and procedural macros.

```rust
struct MacroExpander {
    macro_registry: HashMap<String, MacroDefinition>,
}

impl MacroExpander {
    fn expand_macros(&mut self, ast: &mut AST) -> Result<()> {
        self.visit_ast(ast, |node| {
            if let Some(macro_def) = self.find_macro(node) {
                let expanded = self.expand_macro(macro_def, node)?;
                *node = expanded;
            }
        })
    }
}
```

## 10. Type Checking

### 10.1 Type Inference

Inferring types for expressions.

```rust
struct TypeChecker {
    type_env: TypeEnvironment,
}

impl TypeChecker {
    fn infer_type(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Literal(l) => Ok(self.type_of_literal(l)),
            Expr::Variable(name) => self.type_env.lookup(name),
            Expr::Binary { left, op, right } => {
                let left_type = self.infer_type(left)?;
                let right_type = self.infer_type(right)?;
                self.unify(&left_type, &right_type)?;
                self.type_of_op(op, &left_type)
            }
            _ => Err(TypeError::CannotInfer)
        }
    }
}
```

### 10.2 Type Unification

Unifying types for type checking.

```rust
fn unify(&mut self, t1: &Type, t2: &Type) -> Result<()> {
    match (t1, t2) {
        (Type::Var(v), t) | (t, Type::Var(v)) => {
            self.type_env.bind(*v, t.clone());
            Ok(())
        }
        (Type::Function { params: p1, ret: r1 }, 
         Type::Function { params: p2, ret: r2 }) => {
            if p1.len() != p2.len() {
                return Err(TypeError::ArityMismatch);
            }
            for (a, b) in p1.iter().zip(p2.iter()) {
                self.unify(a, b)?;
            }
            self.unify(r1, r2)
        }
        (a, b) if a == b => Ok(()),
        _ => Err(TypeError::TypeMismatch { expected: t1.clone(), found: t2.clone() })
    }
}
```

## 11. Build Integration

### 11.1 Cargo Integration

Integrating with Cargo build system.

```rust
// In Cargo.toml
[build-dependencies]
rux-compiler = "0.1"

// In build.rs
fn main() {
    rux_compiler::compile_rsx_files("src/**/*.rsx");
}
```

### 11.2 Watch Mode

File watching for development.

```rust
struct Watcher {
    compiler: Compiler,
    watcher: FileWatcher,
}

impl Watcher {
    fn watch(&mut self) -> Result<()> {
        self.watcher.watch("src", |event| {
            if let Event::Modify(path) = event {
                self.compiler.compile_file(&path)?;
            }
        })
    }
}
```

## 12. LLVM Backend Integration

### 12.1 LLVM IR Generation

The RUX compiler generates LLVM IR (Intermediate Representation) from the optimized RUX AST:

```rust
struct LLVMIRGenerator {
    context: llvm::Context,
    module: llvm::Module,
    builder: llvm::IRBuilder,
}

impl LLVMIRGenerator {
    fn generate(&mut self, ast: &OptimizedAST) -> Result<llvm::Module> {
        // Generate LLVM IR from RUX AST
        self.visit_ast(ast);
        Ok(self.module.clone())
    }
    
    fn generate_function(&mut self, component: &Component) {
        // Generate LLVM function from component
        let func_type = self.create_function_type(component);
        let func = self.module.add_function(component.name, func_type);
        // ... generate function body
    }
}
```

### 12.2 LLVM Optimization Pipeline

LLVM applies proven optimization passes to the generated IR:

```rust
fn optimize_with_llvm(module: &mut llvm::Module) -> Result<()> {
    let pass_manager = llvm::PassManager::new();
    
    // Standard optimization passes
    pass_manager.add_instruction_combining_pass();
    pass_manager.add_reassociate_pass();
    pass_manager.add_gvn_pass();
    pass_manager.add_cfg_simplification_pass();
    pass_manager.add_dead_store_elimination_pass();
    
    // Advanced optimizations
    pass_manager.add_loop_vectorize_pass();
    pass_manager.add_slp_vectorize_pass();
    pass_manager.add_aggressive_dce_pass();
    
    // Run optimizations
    pass_manager.run(module);
    
    Ok(())
}
```

### 12.3 Target Code Generation

LLVM generates target-specific code for all supported platforms:

- **x86/x86_64**: Intel and AMD processors
- **ARM**: 32-bit and 64-bit ARM processors
- **RISC-V**: RISC-V architecture
- **WebAssembly**: WASM binary format
- **MIPS, PowerPC**: Additional architectures

### 12.4 RUX-Specific LLVM Passes (Optional)

Custom LLVM passes can be added for RUX-specific optimizations:

```rust
struct ComponentInliningPass;

impl llvm::FunctionPass for ComponentInliningPass {
    fn run_on_function(&mut self, func: &llvm::Function) -> bool {
        // RUX-specific: Inline small components
        // ... implementation
        true
    }
}

struct SignalOptimizationPass;

impl llvm::FunctionPass for SignalOptimizationPass {
    fn run_on_function(&mut self, func: &llvm::Function) -> bool {
        // RUX-specific: Optimize signal dependency tracking
        // ... implementation
        true
    }
}
```

## 13. Endpoint Compilation

The RUX compiler uses LLVM to generate code for multiple targets:

- **Native binaries**: Desktop and mobile platforms (fastest performance)
- **WASM modules**: Web platform (near-native performance)
- **Static libraries**: Platform interop (Android JNI, iOS FFI)

For detailed information on compilation targets, performance characteristics, build configurations, and platform-specific integration, see [Endpoint Compilers](endpoint-compilers.md).

### 13.1 Target Selection

The compilation target is determined by:
1. LLVM target triple specification
2. Feature flags in `Cargo.toml`
3. Platform-specific code via `#[cfg(...)]` attributes

### 13.2 Code Generation for Multiple Targets

LLVM handles target-specific code generation automatically:

```rust
// LLVM automatically generates platform-specific code
let target_machine = llvm::TargetMachine::new(
    target_triple,  // e.g., "x86_64-unknown-linux-gnu"
    cpu,            // e.g., "x86-64"
    features,       // e.g., "+avx2"
    opt_level,      // Optimization level
    reloc_model,    // Relocation model
    code_model,     // Code model
)?;

target_machine.emit_to_file(module, output_path, FileType::Object)?;
```

## 13. Future Considerations

- Parallel compilation
- LSP integration for real-time errors
- Source maps for debugging
- Hot module replacement
- Tree shaking
- Code splitting

