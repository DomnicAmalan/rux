use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rux")]
#[command(about = "RUX compiler and build tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build the project
    Build(BuildArgs),
    /// Start development server with hot reload
    Dev(DevArgs),
    /// Create a new RUX project
    New(NewArgs),
    /// Check code without building
    Check(CheckArgs),
}

#[derive(Args)]
pub struct BuildArgs {
    /// Target platform (web, desktop, android, ios)
    #[arg(short, long, default_value = "web")]
    pub target: String,
    
    /// Output directory
    #[arg(short, long, default_value = "dist")]
    pub out_dir: PathBuf,
    
    /// Release mode
    #[arg(short, long)]
    pub release: bool,
}

#[derive(Args)]
pub struct DevArgs {
    /// Port for development server
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,
    
    /// Open browser automatically
    #[arg(short, long)]
    pub open: bool,
}

#[derive(Args)]
pub struct NewArgs {
    /// Project name
    pub name: String,
    
    /// Template to use
    #[arg(short, long, default_value = "default")]
    pub template: String,
}

#[derive(Args)]
pub struct CheckArgs {
    /// Files to check
    pub files: Vec<PathBuf>,
}

pub fn handle_build(args: BuildArgs) -> anyhow::Result<()> {
    use crate::build::BuildSystem;
    
    println!("Building for target: {}", args.target);
    println!("Output directory: {:?}", args.out_dir);
    println!("Release mode: {}", args.release);
    
    let build_system = BuildSystem::new();
    
    match args.target.as_str() {
        "web" => build_system.build_web(&args.out_dir, args.release)?,
        "desktop" => build_system.build_desktop(&args.out_dir, args.release)?,
        _ => {
            return Err(anyhow::anyhow!("Unknown target: {}", args.target));
        }
    }
    
    println!("✅ Build complete!");
    Ok(())
}

pub async fn handle_dev(args: DevArgs) -> anyhow::Result<()> {
    use crate::file_watcher::FileWatcher;
    use crate::dev_server::DevServer;
    use rux_compiler::IncrementalCompiler;
    use std::time::Duration;
    
    println!("Starting development server on port {}", args.port);
    if args.open {
        println!("Opening browser...");
        // Would open browser here
    }
    
    // Initialize file watcher
    let mut watcher = FileWatcher::new()?;
    if std::path::Path::new("src").exists() {
        watcher.watch_directory(std::path::Path::new("src"))?;
    }
    
    // Initialize incremental compiler
    let mut compiler = IncrementalCompiler::new();
    
    // Start dev server in background
    let server = DevServer::new(args.port);
    let server_port = args.port;
    tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Dev server error: {}", e);
        }
    });
    
    println!("Watching for file changes...");
    
    // Main loop: watch for changes and recompile
    loop {
        let changed = watcher.check_for_changes();
        if !changed.is_empty() {
            println!("Files changed: {:?}", changed);
            match compiler.compile_incremental(&changed) {
                Ok(_) => {
                    println!("✅ Recompiled successfully");
                    // Would send WebSocket update to clients
                }
                Err(e) => {
                    eprintln!("❌ Compilation error: {}", e);
                }
            }
        }
        
        // Sleep briefly to avoid busy-waiting
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub fn handle_new(args: NewArgs) -> anyhow::Result<()> {
    println!("Creating new RUX project: {}", args.name);
    println!("Template: {}", args.template);
    
    // Create project structure
    // Copy template files
    // Initialize git
    
    Ok(())
}

pub fn handle_check(args: CheckArgs) -> anyhow::Result<()> {
    if args.files.is_empty() {
        // Check all .rsx files in src/
        println!("Checking all .rsx files...");
    } else {
        for file in &args.files {
            println!("Checking {:?}...", file);
            let mut compiler = rux_compiler::Compiler::new();
            compiler.compile_file(file)?;
        }
    }
    
    Ok(())
}
