# RUX Build System & Tooling

## Overview

RUX provides a comprehensive build system integrated with Cargo, featuring hot module replacement, fast bundling, and excellent developer experience.

## 1. Cargo Integration

### 1.1 Cargo.toml Configuration

RUX integrates seamlessly with Cargo.

```toml
[package]
name = "rux-app"
version = "0.1.0"
edition = "2021"

[dependencies]
rux = "0.1"

[build-dependencies]
rux-compiler = "0.1"

[package.metadata.rux]
entry = "src/main.rsx"
out_dir = "dist"
```

### 1.2 Build Script

Automatic compilation of `.rsx` files.

```rust
// build.rs
fn main() {
    rux_compiler::compile_rsx_files("src/**/*.rsx");
}
```

### 1.3 Cargo Features

Feature flags for different targets.

```toml
[features]
default = ["web"]
web = ["rux/web"]
desktop = ["rux/desktop"]
mobile = ["rux/mobile"]
embedded = ["rux/embedded"]
```

## 2. Hot Module Replacement (HMR)

### 2.1 HMR Architecture

Lightning-fast HMR inspired by Vite.

```rust
struct HMRServer {
    ws_server: WebSocketServer,
    file_watcher: FileWatcher,
    module_graph: ModuleGraph,
}

impl HMRServer {
    fn handle_file_change(&mut self, path: &Path) {
        // Find affected modules
        let affected = self.module_graph.find_dependents(path);
        
        // Send update to client
        for module in affected {
            self.send_update(module);
        }
    }
}
```

### 2.2 Module Graph Reloading

Efficient module graph updates.

```rust
struct ModuleGraph {
    modules: HashMap<PathBuf, Module>,
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

impl ModuleGraph {
    fn update_module(&mut self, path: &Path, new_content: &str) {
        // Parse new content
        let module = parse_module(new_content);
        
        // Update dependencies
        self.update_dependencies(path, &module);
        
        // Invalidate dependents
        self.invalidate_dependents(path);
    }
}
```

### 2.3 State Preservation

Preserving component state during HMR.

```rust
struct HMRState {
    component_states: HashMap<ComponentId, StateSnapshot>,
}

impl HMRState {
    fn preserve_state(&mut self, component: &Component) {
        let snapshot = component.snapshot_state();
        self.component_states.insert(component.id, snapshot);
    }
    
    fn restore_state(&mut self, component: &mut Component) {
        if let Some(snapshot) = self.component_states.get(&component.id) {
            component.restore_state(snapshot);
        }
    }
}
```

## 3. Dependency Graph Caching

### 3.1 Dependency Graph

Building and caching dependency graph.

```rust
struct DependencyGraph {
    graph: petgraph::Graph<Module, Dependency>,
    cache: HashMap<PathBuf, ModuleHash>,
}

impl DependencyGraph {
    fn build(&mut self, entry: &Path) -> Result<()> {
        // Build graph from entry point
        self.traverse(entry)?;
        
        // Cache graph
        self.cache_graph()?;
        
        Ok(())
    }
    
    fn is_stale(&self, path: &Path) -> bool {
        let current_hash = hash_file(path)?;
        self.cache.get(path) != Some(&current_hash)
    }
}
```

### 3.2 Incremental Compilation

Only compiling changed files.

```rust
struct IncrementalCompiler {
    cache: CompilationCache,
    dependency_graph: DependencyGraph,
}

impl IncrementalCompiler {
    fn compile_incremental(&mut self, changed: &[PathBuf]) -> Result<()> {
        // Find affected modules
        let affected = self.dependency_graph.transitive_deps(changed);
        
        // Compile only affected
        for path in affected {
            self.compile_file(&path)?;
        }
        
        Ok(())
    }
}
```

## 4. Multi-Threaded Bundling

### 4.1 Parallel Compilation

Compiling modules in parallel.

```rust
use rayon::prelude::*;

fn compile_parallel(files: &[PathBuf]) -> Result<()> {
    files.par_iter()
        .map(|file| compile_file(file))
        .collect::<Result<Vec<_>>>()?;
    
    Ok(())
}
```

### 4.2 Task Scheduling

Efficient task scheduling for bundling.

```rust
struct Bundler {
    task_queue: VecDeque<BundleTask>,
    workers: Vec<Worker>,
}

impl Bundler {
    fn bundle(&mut self, entry: &Path) -> Result<Bundle> {
        // Create tasks
        let tasks = self.create_tasks(entry)?;
        
        // Schedule tasks
        for task in tasks {
            self.schedule_task(task);
        }
        
        // Wait for completion
        self.wait_for_completion()?;
        
        // Combine results
        self.combine_bundles()
    }
}
```

## 5. Dev Server Architecture

### 5.1 Dev Server

Development server with HMR.

```rust
struct DevServer {
    http_server: HttpServer,
    ws_server: WebSocketServer,
    file_server: FileServer,
    hmr: HMRServer,
}

impl DevServer {
    fn start(&mut self) -> Result<()> {
        // Start HTTP server
        self.http_server.start("localhost:3000")?;
        
        // Start WebSocket for HMR
        self.ws_server.start("localhost:3001")?;
        
        // Watch files
        self.watch_files()?;
        
        Ok(())
    }
}
```

### 5.2 Request Handling

Handling various request types.

```rust
impl DevServer {
    fn handle_request(&mut self, req: Request) -> Response {
        match req.path() {
            "/" => self.serve_index(),
            path if path.ends_with(".rsx") => self.compile_and_serve(path),
            path if path.starts_with("/hmr") => self.handle_hmr(req),
            _ => self.serve_static(req.path()),
        }
    }
}
```

### 5.3 Source Maps

Source maps for debugging.

```rust
struct SourceMapGenerator {
    mappings: Vec<Mapping>,
}

impl SourceMapGenerator {
    fn generate(&self, output: &str, source: &str) -> String {
        // Generate source map
        format!(r#"{{
            "version": 3,
            "sources": ["{}"],
            "mappings": "{}"
        }}"#, source, self.encode_mappings())
    }
}
```

## 6. Formatter and Linter

### 6.1 RUX Formatter

Automatic code formatting.

```rust
struct RuxFormatter {
    config: FormatConfig,
}

impl RuxFormatter {
    fn format(&self, source: &str) -> Result<String> {
        // Parse
        let ast = parse(source)?;
        
        // Format
        let formatted = self.format_ast(&ast)?;
        
        Ok(formatted)
    }
}
```

### 6.2 Clippy-Style Linter

Linting with helpful suggestions.

```rust
struct RuxLinter {
    rules: Vec<Box<dyn LintRule>>,
}

impl RuxLinter {
    fn lint(&self, ast: &AST) -> Vec<LintWarning> {
        let mut warnings = Vec::new();
        
        for rule in &self.rules {
            warnings.extend(rule.check(ast));
        }
        
        warnings
    }
}
```

### 6.3 Lint Rules

Common lint rules.

```rust
struct UnusedVariableRule;
struct MissingKeyRule;
struct PropTypeMismatchRule;
struct PerformanceRule;
```

## 7. Test Runner Integration

### 7.1 Test Framework

Integrated test runner.

```rust
#[cfg(test)]
mod tests {
    use rux::test::*;
    
    #[rux_test]
    fn test_component() {
        let component = render(<MyComponent />);
        assert_eq!(component.text(), "Hello");
    }
}
```

### 7.2 Component Testing

Testing utilities for components.

```rust
mod test {
    pub fn render(component: Element) -> TestRenderer {
        TestRenderer::new(component)
    }
    
    pub fn fire_event(element: &TestElement, event: Event) {
        // Fire event
    }
    
    pub fn query_selector(renderer: &TestRenderer, selector: &str) -> Option<TestElement> {
        // Query element
    }
}
```

### 7.3 Snapshot Testing

Snapshot testing for UI.

```rust
#[rux_test]
fn test_component_snapshot() {
    let component = render(<MyComponent />);
    assert_snapshot!(component.html());
}
```

## 8. Build Modes

### 8.1 Development Mode

Optimized for development.

```bash
rux dev
# - Fast compilation
# - HMR enabled
# - Source maps
# - Verbose errors
```

### 8.2 Production Mode

Optimized for production.

```bash
rux build --release
# - Full optimizations
# - Code minification
# - Tree shaking
# - Dead code elimination
```

### 8.3 Profile Mode

Performance profiling.

```bash
rux build --profile
# - Performance instrumentation
# - Debug symbols
# - Profiling data
```

## 9. Bundle Optimization

### 9.1 Tree Shaking

Removing unused code.

```rust
struct TreeShaker {
    used_symbols: HashSet<String>,
}

impl TreeShaker {
    fn shake(&mut self, bundle: &mut Bundle) {
        // Mark used symbols
        self.mark_used(bundle.entry);
        
        // Remove unused
        bundle.remove_unused(&self.used_symbols);
    }
}
```

### 9.2 Code Splitting

Splitting code into chunks.

```rust
struct CodeSplitter {
    chunk_size: usize,
}

impl CodeSplitter {
    fn split(&self, bundle: Bundle) -> Vec<Chunk> {
        // Split into chunks
        bundle.split_by_size(self.chunk_size)
    }
}
```

### 9.3 Minification

Minifying output code.

```rust
struct Minifier;

impl Minifier {
    fn minify(&self, code: &str) -> String {
        // Remove whitespace
        // Shorten identifiers
        // Optimize expressions
        code.trim().replace("  ", " ")
    }
}
```

## 10. Plugin System

### 10.1 Plugin Interface

Plugin system for extensibility.

```rust
trait RuxPlugin {
    fn name(&self) -> &str;
    fn transform(&self, ast: &mut AST) -> Result<()>;
    fn generate(&self, code: &mut String) -> Result<()>;
}
```

### 10.2 Built-in Plugins

Common plugins included.

```rust
struct TypeScriptPlugin;
struct SassPlugin;
struct ImageOptimizerPlugin;
```

## 11. Future Considerations

- Parallel test execution
- Coverage reporting
- Performance budgets
- Bundle analysis
- Custom plugin marketplace

