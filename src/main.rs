//! # Scanner
//!
//! A multi-language dependency scanner for Python, Node.js, and Rust ecosystems.

use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::Parser;
use rayon::prelude::*;

use scanner::analyzer::{
    ApplicationLinker, Classifier, InfectedPackageFilter, TreeBuilder, VersionMatcher,
};
use scanner::indexer;
use scanner::models::{Ecosystem, InstalledPackage, ScanResult};
use scanner::output::{
    write_applications_json_with_security, write_classified_csv_with_security,
    write_trees_json_with_security,
};
use scanner::parsers::lockfile::*;
use scanner::parsers::manifest::*;
use scanner::parsers::{NodeModulesParser, ParserRegistry, SitePackagesParser};

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

    /// Scan mode: full, installed-only, declared-only
    #[arg(long, default_value = "full")]
    scan_mode: String,

    /// Output format: csv, json
    #[arg(long, default_value = "csv")]
    format: String,

    /// Include installation directories in traversal
    #[arg(long)]
    include_install_dirs: bool,

    /// Infected package list file (CSV format: package,version1 | version2)
    #[arg(long)]
    infected_list: Option<String>,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,
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
        eprintln!("[debug] Scan mode: {}", args.scan_mode);
        eprintln!("[debug] Output format: {}", args.format);
    }

    println!("Scanning for dependencies across Python, Node.js, and Rust ecosystems...");

    let scan_path = Path::new(&args.dir);
    if !scan_path.exists() {
        eprintln!("[error] Directory does not exist: {}", args.dir);
        return Ok(());
    }

    // Determine scan mode
    let scan_installed = args.scan_mode == "full" || args.scan_mode == "installed-only";
    let scan_declared = args.scan_mode == "full" || args.scan_mode == "declared-only";

    if !scan_installed && !scan_declared {
        eprintln!(
            "[error] Invalid scan mode: {}. Use: full, installed-only, or declared-only",
            args.scan_mode
        );
        return Ok(());
    }

    // Validate output format
    if args.format != "csv" && args.format != "json" {
        eprintln!("[error] Invalid format: {}. Use: csv or json", args.format);
        return Ok(());
    }

    // Determine output file
    let output_file = args.output.unwrap_or_else(|| {
        if args.format == "json" {
            "output.json".to_string()
        } else {
            "output.csv".to_string()
        }
    });

    // Initialize parser registry for declared dependencies
    let mut registry = ParserRegistry::new();

    if scan_declared {
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
            eprintln!(
                "[debug] Registered {} parsers",
                registry.registered_filenames().len()
            );
        }
    }

    // Discover files
    let mut exclude_dirs = vec![".nx", "target", ".git", "__pycache__"];

    // Conditionally exclude installation directories from declared dependency scanning
    // Note: We still want to find manifests/lockfiles in venvs, so we only exclude
    // the actual package directories (node_modules, site-packages)
    if !args.include_install_dirs {
        exclude_dirs.extend(vec!["node_modules", "site-packages", "dist-packages"]);
    }

    let discovered_files = if scan_declared {
        // Determine scan mode enum
        let mode = match args.scan_mode.as_str() {
            "full" => indexer::ScanMode::Full,
            "installed-only" => indexer::ScanMode::InstalledOnly,
            "declared-only" => indexer::ScanMode::DeclaredOnly,
            _ => indexer::ScanMode::Full,
        };

        indexer::find_files_with_mode(scan_path, &exclude_dirs, mode, args.include_install_dirs)
    } else {
        vec![]
    };

    if args.verbose {
        eprintln!(
            "[debug] Discovered {} manifest/lockfiles",
            discovered_files.len()
        );
    }

    // Filter by ecosystem if specified
    let discovered_files: Vec<_> = if let Some(ref eco) = args.ecosystem {
        let filter_eco = match eco.as_str() {
            "node" => Ecosystem::Node,
            "python" => Ecosystem::Python,
            "rust" => Ecosystem::Rust,
            _ => {
                eprintln!(
                    "[error] Unknown ecosystem: {}. Use: node, python, or rust",
                    eco
                );
                return Ok(());
            }
        };
        discovered_files
            .into_iter()
            .filter(|f| f.ecosystem == filter_eco)
            .collect()
    } else {
        discovered_files
    };

    // Parse declared dependencies
    let dependency_records = if scan_declared {
        println!("Found {} package files to parse", discovered_files.len());
        let scan_result = Arc::new(Mutex::new(ScanResult::new()));

        discovered_files.par_iter().for_each(|file| {
            if let Some(parser) = registry.get_parser(&file.filename) {
                match std::fs::read_to_string(&file.path) {
                    Ok(content) => match parser.parse(&content, &file.path) {
                        Ok(records) => {
                            if args.verbose && !records.is_empty() {
                                eprintln!(
                                    "[debug] Parsed {} dependencies from {:?}",
                                    records.len(),
                                    file.path
                                );
                            }
                            scan_result.lock().unwrap().add_all(records);
                        }
                        Err(e) => {
                            eprintln!("[warn] Failed to parse {:?}: {}", file.path, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("[warn] Failed to read {:?}: {}", file.path, e);
                    }
                }
            }
        });

        let result = Arc::try_unwrap(scan_result).unwrap().into_inner().unwrap();
        result.dependencies
    } else {
        vec![]
    };

    // Scan for installed packages
    let installed_packages = if scan_installed {
        println!("Scanning for installed packages...");
        let installed = Arc::new(Mutex::new(Vec::<InstalledPackage>::new()));

        // Find installation directories
        let install_dirs = indexer::install_dirs::find_all_install_dirs(scan_path, &[]);

        if args.verbose {
            eprintln!(
                "[debug] Found {} installation directories",
                install_dirs.len()
            );
        }

        // Parse installed packages in parallel
        install_dirs
            .par_iter()
            .for_each(|install_dir| match install_dir.dir_type {
                indexer::install_dirs::InstallDirType::NodeModules => {
                    let parser = NodeModulesParser;
                    match parser.parse_installed(&install_dir.path) {
                        Ok(packages) => {
                            if args.verbose && !packages.is_empty() {
                                eprintln!(
                                    "[debug] Found {} installed packages in {:?}",
                                    packages.len(),
                                    install_dir.path
                                );
                            }
                            installed.lock().unwrap().extend(packages);
                        }
                        Err(e) => {
                            eprintln!("[warn] Failed to parse {:?}: {}", install_dir.path, e);
                        }
                    }
                }
                indexer::install_dirs::InstallDirType::SitePackages
                | indexer::install_dirs::InstallDirType::DistPackages
                | indexer::install_dirs::InstallDirType::VirtualEnv => {
                    let parser = SitePackagesParser;
                    match parser.parse_installed(&install_dir.path) {
                        Ok(packages) => {
                            if args.verbose && !packages.is_empty() {
                                eprintln!(
                                    "[debug] Found {} installed packages in {:?}",
                                    packages.len(),
                                    install_dir.path
                                );
                            }
                            installed.lock().unwrap().extend(packages);
                        }
                        Err(e) => {
                            eprintln!("[warn] Failed to parse {:?}: {}", install_dir.path, e);
                        }
                    }
                }
            });

        Arc::try_unwrap(installed).unwrap().into_inner().unwrap()
    } else {
        vec![]
    };

    println!("Found {} installed packages", installed_packages.len());

    // Classify dependencies
    let classifier = Classifier::new();
    let mut classified = classifier.classify(dependency_records, installed_packages);

    if args.verbose {
        eprintln!(
            "[debug] Classified {} unique dependencies",
            classified.len()
        );
    }

    // Detect version mismatches
    let version_matcher = VersionMatcher::new();
    for dep in &mut classified {
        if let (Some(has_ver), Some(should_ver)) = (
            dep.get_version(scanner::models::Classification::Has),
            dep.get_version(scanner::models::Classification::Should),
        ) {
            dep.has_version_mismatch = version_matcher.detect_version_mismatch(has_ver, should_ver);
        }

        if let (Some(should_ver), Some(can_range)) = (
            dep.get_version(scanner::models::Classification::Should),
            dep.get_version(scanner::models::Classification::Can),
        ) {
            dep.has_constraint_violation =
                version_matcher.detect_constraint_violation(should_ver, can_range, dep.ecosystem);
        }
    }

    // Load infected package list if provided
    let infected_filter = if let Some(infected_file) = &args.infected_list {
        println!("Loading infected package list from {}...", infected_file);
        let mut filter = InfectedPackageFilter::new();
        match filter.load_from_csv(Path::new(infected_file)) {
            Ok(_) => {
                println!("Loaded {} infected packages", filter.count());

                // Count infected dependencies
                let infected_count = classified.iter().filter(|d| filter.is_infected(d)).count();
                let match_package_count = classified
                    .iter()
                    .filter(|d| {
                        matches!(
                            filter.get_security_status(d),
                            scanner::analyzer::SecurityStatus::MatchPackage
                        )
                    })
                    .count();

                println!("Found {} infected dependencies", infected_count);
                if match_package_count > 0 {
                    println!(
                        "Found {} dependencies with matching package names (different versions)",
                        match_package_count
                    );
                }

                Some(filter)
            }
            Err(e) => {
                eprintln!("[error] Failed to load infected package list: {}", e);
                return Ok(());
            }
        }
    } else {
        None
    };

    // Link to applications
    let linker = ApplicationLinker::new();
    let applications = linker.link_to_applications(classified.clone());

    if args.verbose {
        eprintln!(
            "[debug] Linked dependencies to {} applications",
            applications.len()
        );
    }

    println!("\nScan complete!");
    println!("Total unique dependencies: {}", classified.len());
    println!("Applications found: {}", applications.len());

    // Write output
    match args.format.as_str() {
        "csv" => {
            write_classified_csv_with_security(
                &classified,
                infected_filter.as_ref(),
                &output_file,
            )?;
            println!("\nResults written to {}", output_file);
        }
        "json" => {
            if args.scan_mode == "full" {
                // Build dependency trees for full scan
                let tree_builder = TreeBuilder::new();
                let trees = tree_builder.build_trees(applications.clone());
                write_trees_json_with_security(trees, infected_filter.as_ref(), &output_file)?;
                println!("\nDependency trees written to {}", output_file);
            } else {
                // Just write applications without trees
                write_applications_json_with_security(
                    applications,
                    infected_filter.as_ref(),
                    &output_file,
                )?;
                println!("\nResults written to {}", output_file);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
