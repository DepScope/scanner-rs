//! # Scanner
//!
//! A tool to scan directories for npm package compatibility across various lock files.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path};
use std::process::Command;
use std::sync::Mutex;

use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde_json::Value;
use walkdir::WalkDir;

/// Command line arguments for the scanner
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to start scanning from
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Only check the root directory
    #[arg(long)]
    root_only: bool,

    /// Only list directories to be checked
    #[arg(long)]
    list_dirs: bool,

    /// Number of worker threads to use
    #[arg(short = 'j', long, default_value_t = num_cpus::get())]
    jobs: usize,

    /// Skip calling npm (fast)
    #[arg(long = "no-npm")]
    no_npm: bool,

    /// Verbose logging (debug)
    #[arg(short, long)]
    verbose: bool,
}

/// Preloaded lock file contents for a directory
struct Preload {
    /// Content of yarn.lock file
    yarn: Option<String>,
    /// Parsed package-lock.json content
    plock: Option<Value>,
    /// Content of pnpm-lock.yaml file
    pnpm: Option<String>,
    /// Content of DEPENDENCIES.json file
    deps: Option<String>,
}

fn find_dirs(root: &Path) -> Vec<String> {
    let patterns = vec![
        "package.json",
        "yarn.lock",
        "package-lock.json",
        "pnpm-lock.yaml",
        "pnpm-workspace.yaml",
        "DEPENDENCIES.json",
    ];
    let exclude_dirs = vec![".nx", "node_modules"];
    let mut dirs: HashSet<String> = HashSet::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().components().any(|c| exclude_dirs.contains(&c.as_os_str().to_str().unwrap_or(""))))
    {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap_or("");
            if patterns.contains(&file_name) {
                if let Some(parent) = entry.path().parent() {
                    let dir_str = parent.to_str().unwrap_or(".").to_string();
                    if !dir_str.contains("/node_modules/") && !dir_str.contains("/.nx/") {
                        if let Ok(abs_path) = fs::canonicalize(parent) {
                            if let Some(abs_str) = abs_path.to_str() {
                                dirs.insert(abs_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(abs_root) = fs::canonicalize(root) {
        if let Some(abs_str) = abs_root.to_str() {
            dirs.insert(abs_str.to_string());
        }
    }
    let mut sorted_dirs: Vec<String> = dirs.into_iter().collect();
    sorted_dirs.sort();
    sorted_dirs
}

#[allow(dead_code)]
fn parse_version(v: &str) -> Option<(i32, i32, i32)> {
    let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)").unwrap();
    re.captures(v).map(|cap| (
        cap[1].parse().unwrap(),
        cap[2].parse().unwrap(),
        cap[3].parse().unwrap(),
    ))
}

fn get_pkg_range(dirpath: &str, name: &str) -> String {
    let pj_path = Path::new(dirpath).join("package.json");
    if !pj_path.is_file() {
        return String::new();
    }
    let file = match File::open(pj_path) {
        Ok(f) => f,
        _ => return String::new(),
    };
    let data: Value = match serde_json::from_reader(file) {
        Ok(d) => d,
        _ => return String::new(),
    };
    for section in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(deps) = data.get(section).and_then(|d| d.as_object()) {
            if let Some(r) = deps.get(name).and_then(|r| r.as_str()) {
                return r.to_string();
            }
        }
    }
    String::new()
}

fn get_yarn_versions(name: &str, content: &str) -> HashSet<String> {
    let mut versions: HashSet<String> = HashSet::new();
    let record_re = Regex::new(r"\n\s*\n").unwrap();
    let records: Vec<&str> = record_re.split(content).collect();
    let ver_re = Regex::new(r#"version "(\d+\.\d+\.\d+)""#).unwrap();
    for rec in records {
        if rec.contains(&format!("{}@", name)) {
            if let Some(cap) = ver_re.captures(rec) {
                versions.insert(cap[1].to_string());
            }
        }
    }
    versions
}

fn get_package_lock_versions(name: &str, package_lock_json: &Value) -> HashSet<String> {
    let mut versions: HashSet<String> = HashSet::new();
    if let Some(deps) = package_lock_json.get("dependencies").and_then(|d| d.as_object()) {
        if let Some(v) = deps.get(name).and_then(|v| v.get("version")).and_then(|v| v.as_str()) {
            versions.insert(v.to_string());
        }
    }
    if let Some(packages) = package_lock_json.get("packages").and_then(|p| p.as_object()) {
        let key = format!("node_modules/{}", name);
        if let Some(v) = packages.get(&key).and_then(|v| v.get("version")).and_then(|v| v.as_str()) {
            versions.insert(v.to_string());
        }
    }
    if let Some(deps) = package_lock_json.get("dependencies").and_then(|d| d.as_object()) {
        for (_, v) in deps {
            if let Some(obj) = v.as_object() {
                if obj.get("version").is_some() {
                    walk_plock(obj, name, &mut versions);
                }
            }
        }
    }
    versions
}

fn walk_plock(obj: &serde_json::Map<String, Value>, name: &str, versions: &mut HashSet<String>) {
    if let Some(deps) = obj.get("dependencies").and_then(|d| d.as_object()) {
        for (k, v) in deps {
            if k == name {
                if let Some(ver) = v.get("version").and_then(|vv| vv.as_str()) {
                    versions.insert(ver.to_string());
                }
            }
            if let Some(sub_obj) = v.as_object() {
                walk_plock(sub_obj, name, versions);
            }
        }
    }
}

fn get_pnpm_versions(name: &str, content: &str) -> HashSet<String> {
    let mut versions: HashSet<String> = HashSet::new();
    let pattern = Regex::new(&format!(r"/{}/(\d+\.\d+\.\d+)", regex::escape(name))).unwrap();
    for cap in pattern.captures_iter(content) {
        versions.insert(cap[1].to_string());
    }
    let pattern2 = Regex::new(&format!(r#""{}@(\d+\.\d+\.\d+)""#, regex::escape(name))).unwrap();
    for cap in pattern2.captures_iter(content) {
        versions.insert(cap[1].to_string());
    }
    versions
}

fn get_dependencies_versions(name: &str, content: &str) -> HashSet<String> {
    let mut versions: HashSet<String> = HashSet::new();
    let pattern = Regex::new(&format!(r#""name"\s*:\s*"{}@(\d+\.\d+\.\d+)""#, regex::escape(name))).unwrap();
    for cap in pattern.captures_iter(content) {
        versions.insert(cap[1].to_string());
    }
    if let Ok(data) = serde_json::from_str::<Value>(content) {
        walk_deps(&data, name, &mut versions);
    }
    versions
}

fn walk_deps(obj: &Value, name: &str, versions: &mut HashSet<String>) {
    match obj {
        Value::Object(map) => {
            if let Some(nm) = map.get("name").and_then(|n| n.as_str()) {
                if nm.starts_with(&format!("{}@", name)) {
                    let parts: Vec<&str> = nm.split('@').collect();
                    if parts.len() == 2 && Regex::new(r"^\d+\.\d+\.\d+$").unwrap().is_match(parts[1]) {
                        versions.insert(parts[1].to_string());
                    }
                }
            }
            for (_, v) in map {
                walk_deps(v, name, versions);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                walk_deps(item, name, versions);
            }
        }
        _ => {}
    }
}

fn get_npm_versions(dirpath: &str, name: &str) -> HashSet<String> {
    let mut versions: HashSet<String> = HashSet::new();
    let output = match Command::new("npm")
        .args(["ls", "--json", name, "--depth=Infinity"])
        .current_dir(dirpath)
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        _ => return versions,
    };
    let output_str = match std::str::from_utf8(&output) {
        Ok(s) => s,
        _ => return versions,
    };
    let data: Value = match serde_json::from_str(output_str) {
        Ok(d) => d,
        _ => return versions,
    };
    walk_npm(&data, name, &mut versions);
    versions
}

fn walk_npm(obj: &Value, name: &str, versions: &mut HashSet<String>) {
    if let Value::Object(map) = obj {
        if let Some(deps) = map.get("dependencies").and_then(|d| d.as_object()) {
            for (k, v) in deps {
                if k == name {
                    if let Some(ver) = v.get("version").and_then(|vv| vv.as_str()) {
                        versions.insert(ver.to_string());
                    }
                }
                walk_npm(v, name, versions);
            }
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build_global()
        .unwrap();

    if args.verbose {
        eprintln!("[debug] Using {} threads", args.jobs);
    }

    println!("Checking for npm packages and lockfile/package.json/DEPENDENCIES.json compatibility in this project and subfolders...");

    let scan_path = Path::new(&args.dir);
    if !scan_path.exists() {
        eprintln!("[error] Directory does not exist: {}", args.dir);
        return Ok(());
    }

    let mut dirs = find_dirs(scan_path);
    if args.root_only {
        if let Ok(abs_path) = fs::canonicalize(scan_path) {
            if let Some(abs_str) = abs_path.to_str() {
                dirs = vec![abs_str.to_string()];
            }
        }
    }

    eprintln!("Directories to be checked:");
    for d in &dirs {
        eprintln!("  {}", d);
    }
    if args.list_dirs {
        return Ok(());
    }

    // Read packages from text file
    let packages_file = File::open("packages.txt")?;
    let packages: Vec<(String, String)> = BufReader::new(packages_file)
        .lines()
        .filter_map(|line| {
            if let Ok(l) = line {
                let parts: Vec<&str> = l.trim().split('@').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if packages.is_empty() {
        eprintln!("[error] No packages found in packages.txt");
        return Ok(());
    }

    if args.verbose {
        eprintln!("[debug] Loaded {} packages from packages.txt", packages.len());
    }

    // Preload lock files
    let mut preloads: HashMap<String, Preload> = HashMap::new();
    for d in &dirs {
        let mut preload = Preload {
            yarn: None,
            plock: None,
            pnpm: None,
            deps: None,
        };
        let dir_path = Path::new(d);
        if let Ok(content) = fs::read_to_string(dir_path.join("yarn.lock")) {
            preload.yarn = Some(content);
        }
        let plock_path = dir_path.join("package-lock.json");
        if plock_path.is_file() {
            if let Ok(file) = File::open(&plock_path) {
                if let Ok(value) = serde_json::from_reader(file) {
                    preload.plock = Some(value);
                }
            }
        }
        if let Ok(content) = fs::read_to_string(dir_path.join("pnpm-lock.yaml")) {
            preload.pnpm = Some(content);
        }
        if let Ok(content) = fs::read_to_string(dir_path.join("DEPENDENCIES.json")) {
            preload.deps = Some(content);
        }
        preloads.insert(d.clone(), preload);
    }

    if args.verbose {
        eprintln!("[debug] Preloaded lockfiles for {} directories", preloads.len());
    }

    // Prepare for parallel processing
    let rows_mutex: Mutex<Vec<(String, String, String, bool, bool)>> = Mutex::new(Vec::new());
    let found_mutex: Mutex<Vec<String>> = Mutex::new(Vec::new());

    dirs.par_iter().for_each(|d| {
        let preload = preloads.get(d).unwrap();
        for (name, version) in &packages {
            let rng = get_pkg_range(d, name);
            let mut versions_by_file: HashMap<String, HashSet<String>> = HashMap::new();

            if let Some(content) = &preload.yarn {
                let yv = get_yarn_versions(name, content);
                if !yv.is_empty() {
                    versions_by_file.insert("yarn.lock".to_string(), yv);
                }
            }
            if let Some(plock) = &preload.plock {
                let plv = get_package_lock_versions(name, plock);
                if !plv.is_empty() {
                    versions_by_file.insert("package-lock.json".to_string(), plv);
                }
            }
            if let Some(content) = &preload.pnpm {
                let pnv = get_pnpm_versions(name, content);
                if !pnv.is_empty() {
                    versions_by_file.insert("pnpm-lock.yaml".to_string(), pnv);
                }
            }
            if let Some(content) = &preload.deps {
                let dev = get_dependencies_versions(name, content);
                if !dev.is_empty() {
                    versions_by_file.insert("DEPENDENCIES.json".to_string(), dev);
                }
            }

            let mut nv: HashSet<String> = HashSet::new();
            if !args.no_npm {
                nv = get_npm_versions(d, name);
                if !nv.is_empty() {
                    versions_by_file.insert("npm_installed".to_string(), nv.clone());
                }
            }

            let mut all_versions: HashSet<String> = versions_by_file.values().flat_map(|s| s.clone()).collect();
            all_versions.extend(nv.iter().cloned());

            let match_package = !rng.is_empty() || !all_versions.is_empty();
            let match_version = all_versions.contains(version);

            rows_mutex.lock().unwrap().push((
                name.clone(),
                version.clone(),
                d.to_string(),
                match_package,
                match_version,
            ));

            if match_package && match_version {
                found_mutex.lock().unwrap().push(format!("{}:{}@{}", d, name, version));
            }
        }
    });

    // Sort and print found
    let mut found = found_mutex.into_inner().unwrap();
    found.sort();
    for item in found {
        println!("{}", item);
    }

    // Write CSV
    let mut csv_writer = csv::Writer::from_path("output.csv")?;
    csv_writer.write_record(&["package", "version", "location", "match_package", "match_version"])?;

    let mut rows = rows_mutex.into_inner().unwrap();
    rows.sort_by_key(|r| (r.0.clone(), r.1.clone(), r.2.clone()));
    for (pkg, ver, loc, mp, mv) in rows {
        csv_writer.write_record(&[
            pkg,
            ver,
            loc,
            mp.to_string(),
            mv.to_string(),
        ])?;
    }

    println!("Scan complete.");

    Ok(())
}
