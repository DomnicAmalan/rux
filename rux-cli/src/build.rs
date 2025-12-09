use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};

pub struct BuildSystem;

impl BuildSystem {
    pub fn new() -> Self {
        Self
    }
    
    pub fn build_web(&self, out_dir: &Path, release: bool) -> Result<()> {
        println!("Building for web target...");
        
        // 1. Find all .rsx files
        let rsx_files = Self::find_rsx_files(Path::new("src"))?;
        println!("Found {} .rsx files", rsx_files.len());
        
        // 2. Compile .rsx files to Rust
        let mut compiler = rux_compiler::Compiler::new();
        let mut generated_rust = String::new();
        
        generated_rust.push_str("// Auto-generated from .rsx files\n");
        generated_rust.push_str("use rux_core::virtual_tree::{VirtualNode, NodeType, PropValue};\n");
        generated_rust.push_str("use std::collections::HashMap;\n\n");
        
        for rsx_file in &rsx_files {
            println!("Compiling {:?}...", rsx_file);
            match compiler.compile_file(rsx_file) {
                Ok(ast) => {
                    let mut codegen = rux_compiler::CodeGenerator::new();
                    match codegen.generate_rust_code(&ast) {
                        Ok(rust_code) => {
                            generated_rust.push_str(&format!("// From {:?}\n", rsx_file));
                            generated_rust.push_str(&rust_code);
                            generated_rust.push_str("\n\n");
                        }
                        Err(e) => {
                            eprintln!("Error generating code for {:?}: {}", rsx_file, e);
                            return Err(e.into());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error compiling {:?}: {}", rsx_file, e);
                    return Err(e.into());
                }
            }
        }
        
        // 3. Write generated Rust code
        std::fs::create_dir_all(out_dir)?;
        let generated_path = out_dir.join("generated.rs");
        std::fs::write(&generated_path, generated_rust)?;
        println!("Generated Rust code: {:?}", generated_path);
        
        // 4. Compile to WASM (would use wasm-pack in real implementation)
        println!("WASM compilation would happen here (requires wasm-pack)");
        
        // 5. Generate HTML entry point
        let html = Self::generate_html();
        let html_path = out_dir.join("index.html");
        std::fs::write(&html_path, html)?;
        println!("Generated HTML: {:?}", html_path);
        
        Ok(())
    }
    
    pub fn build_desktop(&self, out_dir: &Path, release: bool) -> Result<()> {
        println!("Building for desktop target...");
        
        // 1. Find all .rsx files
        let rsx_files = Self::find_rsx_files(Path::new("src"))?;
        println!("Found {} .rsx files", rsx_files.len());
        
        // 2. Compile .rsx files to Rust
        let mut compiler = rux_compiler::Compiler::new();
        let mut generated_rust = String::new();
        
        generated_rust.push_str("// Auto-generated from .rsx files\n");
        generated_rust.push_str("use rux_core::virtual_tree::{VirtualNode, NodeType, PropValue};\n");
        generated_rust.push_str("use std::collections::HashMap;\n\n");
        
        for rsx_file in &rsx_files {
            println!("Compiling {:?}...", rsx_file);
            match compiler.compile_file(rsx_file) {
                Ok(ast) => {
                    let mut codegen = rux_compiler::CodeGenerator::new();
                    match codegen.generate_rust_code(&ast) {
                        Ok(rust_code) => {
                            generated_rust.push_str(&format!("// From {:?}\n", rsx_file));
                            generated_rust.push_str(&rust_code);
                            generated_rust.push_str("\n\n");
                        }
                        Err(e) => {
                            eprintln!("Error generating code for {:?}: {}", rsx_file, e);
                            return Err(e.into());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error compiling {:?}: {}", rsx_file, e);
                    return Err(e.into());
                }
            }
        }
        
        // 3. Write generated Rust code
        std::fs::create_dir_all(out_dir)?;
        let generated_path = out_dir.join("generated.rs");
        std::fs::write(&generated_path, generated_rust)?;
        println!("Generated Rust code: {:?}", generated_path);
        
        // 4. Compile to native binary (would use cargo build in real implementation)
        println!("Native compilation would happen here (requires cargo build)");
        
        Ok(())
    }
    
    fn find_rsx_files(dir: &Path) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;
        
        let mut files = Vec::new();
        for entry in WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rsx") {
                files.push(path.to_path_buf());
            }
        }
        Ok(files)
    }
    
    fn generate_html() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RUX App</title>
</head>
<body>
    <div id="root"></div>
    <script type="module">
        import init from './rux_web.js';
        init().then(() => {
            console.log('RUX app loaded');
        });
    </script>
</body>
</html>"#.to_string()
    }
}

impl Default for BuildSystem {
    fn default() -> Self {
        Self::new()
    }
}
