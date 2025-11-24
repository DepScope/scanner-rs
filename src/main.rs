//! # Scanner
//!
//! A multi-language dependency scanner for Python, Node.js, and Rust ecosystems.

use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::Parser;
use rayon::prelude::*;

use scanner::indexer;
use scanner::models::{Ecosystem, ScanResult};
use scanner::parsers::ParserRegistry;
use scanner::parsers::manifest::*;
use scanner::parsers::lockfile::*;

/// Command line arguments for the scanner
#[derive(Parser, Debug)]
#[command(author, version, about = "Multi-language dependency scanner", long_about = None)]
struct Args {
    /// Directory to start scanning from
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Number of worker threads to use
    #[arg(short = 'j', long, default_value_t = num_cpus::get())]
    jobs: usize,

    /// Verbose logging (debug)
    #[arg(short, long)]
    verbose: bool,
    
    /// Filter by ecosystem (node, python, rust)
    #[arg(long)]
    ecosystem: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Configure thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build_global()
        .unwrap();

    if args.verbose {
        eprintln!("[debug] Using {} threads", args.jobs);
    }

    println!("Scanning for dependencies across Python, Node.js, and Rust ecosystems...");

    let scan_path = Path::new(&args.dir);
    if !scan_path.exists() {
        eprintln!("[error] Directory does not exist: {}", args.dir);
        return Ok(());
    }

    // Initialize parser registry
    let mut registry = ParserRegistry::new();
    
    // Register Node.js parsers
    registry.register(Arc::new(PackageJsonParser));
    registry.register(Arc::new(YarnLockParser));
    registry.register(Arc::new(PackageLockJsonParser));
    registry.register(Arc::new(PnpmLockParser));
    
    // Register Python parsers
    registry.register(Arc::new(PyprojectTomlParser));
    registry.register(Arc::new(RequirementsTxtParser));
    registry.register(Arc::new(PoetryLockParser));
    registry.register(Arc::new(UvLockParser));
    
    // Register Rust parsers
    registry.register(Arc::new(CargoTomlParser));
    registry.register(Arc::new(CargoLockParser));

    if args.verbose {
        eprintln!("[debug] Registered {} parsers", registry.registered_filenames().len());
    }

    // Discover files
    let exclude_dirs = vec!["node_modules", ".nx", "target", ".git", ".venv", "venv", "__pycache__"];
    let discovered_files = indexer::find_files(scan_path, &exclude_dirs);

    if args.verbose {
        eprintln!("[debug] Discovered {} files", discovered_files.len());
    }

    // Filter by ecosystem if specified
    let discovered_files: Vec<_> = if let Some(ref eco) = args.ecosystem {
        let filter_eco = match eco.as_str() {
            "node" => Ecosystem::Node,
            "python" => Ecosystem::Python,
            "rust" => Ecosystem::Rust,
            _ => {
                eprintln!("[error] Unknown ecosystem: {}. Use: node, python, or rust", eco);
                return Ok(());
            }
        };
        discovered_files.into_iter().filter(|f| f.ecosystem == filter_eco).collect()
    } else {
        discovered_files
    };

    println!("Found {} package files to parse", discovered_files.len());

    // Parse files in parallel
    let scan_result = Arc::new(Mutex::new(ScanResult::new()));
    
    discovered_files.par_iter().for_each(|file| {
        if let Some(parser) = registry.get_parser(&file.filename) {
            match std::fs::read_to_string(&file.path) {
                Ok(content) => {
                    match parser.parse(&content, &file.path) {
                        Ok(records) => {
                            if args.verbose && !records.is_empty() {
                                eprintln!("[debug] Parsed {} dependencies from {:?}", records.len(), file.path);
                            }
                            scan_result.lock().unwrap().add_all(records);
                        }
                        Err(e) => {
                            eprintln!("[warn] Failed to parse {:?}: {}", file.path, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[warn] Failed to read {:?}: {}", file.path, e);
                }
            }
        }
    });

    let mut result = Arc::try_unwrap(scan_result).unwrap().into_inner().unwrap();
    result.sort();

    println!("\nScan complete!");
    println!("Total dependencies found: {}", result.total_count());
    
    // Print statistics by ecosystem
    let stats = result.ecosystem_stats();
    for (ecosystem, count) in stats {
        println!("  {}: {} dependencies", ecosystem, count);
    }

    // Write CSV output
    scanner::output::write_csv(&result.dependencies, "output.csv")?;
    println!("\nResults written to output.csv");

    Ok(())
}


