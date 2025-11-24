### SemVer Crates (General & Node-Specific)

| Category | Crate | Latest Version (as of Nov 2025) | Downloads/Month | Last Updated | Why Best? |
|----------|--------|-------------------------------|-----------------|--------------|-----------|
| **General SemVer** | `semver` | 1.0.24 | 15M+ | Oct 2025 | Official Cargo spec implementation; full SemVer 2.0 support (parse, compare, ranges like `^`, `~`); fast, mature, no Node quirks. |
| **Node/NPM SemVer** | `node-semver` | 0.9.1 | 500K+ | Sep 2025 | Exact match for npm/yarn/pnpm/Bun behavior (e.g., pre-release handling in `^0.x`); pixel-perfect with Node.js semver lib. |

- **General Use**: `semver` for most cases; add to `Cargo.toml`: `cargo add semver`.
- **Node Compatibility**: `node-semver` if parsing npm-style versions; `cargo add node-semver`.

### Parsing Manifest Files (Declared Dependencies)

| Language/Tool | File | Best Crate | Latest Version | Downloads/Month | Last Updated | Notes |
|---------------|------|------------|----------------|-----------------|--------------|-------|
| **Python** | `requirements.txt` | `requirements` | 0.1.3 | 10K+ | Aug 2025 | Dedicated parser; handles ranges, URLs, extras. |
| **Python** | `pyproject.toml` (Poetry/uv) | `pyproject-toml` | 0.10.1 | 50K+ | Oct 2025 | Full PEP 517/621 support; covers Poetry `[tool.poetry]` and uv sections. |
| **Node/TS** | `package.json` | `serde_json` (with custom struct) | 1.0.128 | 100M+ | Nov 2025 | Standard JSON; define `#[derive(Deserialize)] struct PackageJson { dependencies: HashMap<String, String> }`. No dedicated crate needed. |
| **Node/TS** | `tsconfig.json` | `tsconfig` | 0.1.0 | 5K+ | 2021 (stale) | Parses TS configs; handles `extends`. **Best alternative**: `serde_json` for basic use. |
| **Rust** | `Cargo.toml` | `cargo_toml` | 0.21.3 | 2M+ | Nov 2025 | Full manifest support; extracts deps, features, edition. |

- **No Crate Cases**: For `tsconfig.json`, use `serde_json` as it's plain JSON—simple and actively maintained.

### Parsing Lockfiles (Installed/Resolved Versions)

| Language/Tool | File | Best Crate | Latest Version | Downloads/Month | Last Updated | Notes |
|---------------|------|------------|----------------|-----------------|--------------|-------|
| **Python (pip)** | `pip freeze` output | None | N/A | N/A | N/A | **Best way**: Custom line parser with `nom` or regex (e.g., split on `==`); format is `pkg==ver`. |
| **Python (Poetry)** | `poetry.lock` | `serde_toml` (with custom struct) | 0.9.8 (toml) | 50M+ | Nov 2025 | TOML-based; parse into `struct Lock { package: Vec<Package> }`. No dedicated crate. |
| **Python (uv)** | `uv.lock` | None | N/A | N/A | N/A | **Best way**: Custom TOML parser via `toml` crate; uv.lock is TOML with resolved deps. |
| **Node (npm)** | `package-lock.json` | `package-lock-json-parser` | 0.3.0 | 1K+ | May 2023 (stale) | Handles v1-v3; extracts resolved versions. **Best alternative**: `serde_json`. |
| **Node (Yarn)** | `yarn.lock` | `yarn-lock-parser` | 0.11.0 | 4K+ | Jul 2025 | Supports v1/v2; nom-based for custom format. |
| **Node (pnpm)** | `pnpm-lock.yaml` | `serde_yaml` (with custom struct) | 0.9.34 | 10M+ | Oct 2025 | YAML-based; parse into `struct Lock { packages: HashMap<String, Entry> }`. No dedicated crate. |
| **Node (Bun)** | `bun.lockb` / `bun.lock` | None | N/A | N/A | N/A | **Best way**: Run `bun ./bun.lockb` to export as yarn.lock, then use `yarn-lock-parser`. For text `bun.lock` (JSONC), use `serde_json`. |
| **Rust** | `Cargo.lock` | `cargo_lock` | 11.0.0 | 100K+ | Jan 2025 | Supports v1-v4; optional dep tree graph via petgraph. |

- **No Crate Cases**:
  - `pip freeze`: Simple line-by-line parsing (e.g., `lines.iter().map(|l| l.split("==").collect::<Vec<_>>())`).
  - `poetry.lock` / `uv.lock`: Use `toml::from_str` with structs.
  - `pnpm-lock.yaml`: Use `serde_yaml::from_str`.
  - `bun.lockb`: External conversion to text format.

### Other Common Formats You Might Have Forgotten

Based on ecosystem scans, here are notable omissions from your list (focusing on Python/Node/Rust; excluding niche like R's `DESCRIPTION` or Go's `go.mod`):

- **Python**: `Pipfile` / `Pipfile.lock` (Pipenv)—parse with `toml` crate (TOML format).
- **Python**: `environment.yml` (Conda)—YAML; use `serde_yaml`.
- **Node**: `npm-shrinkwrap.json` (legacy npm)—JSON; `serde_json`.
- **Node**: `yarn-error.log` (debug)—text; custom parser.
- **Rust**: `.cargo/config.toml` (Cargo config)—TOML; `toml` crate.
- **Cross-Language**: `pubspec.yaml` / `pubspec.lock` (Dart/Flutter)—YAML; `serde_yaml`. (If expanding beyond Python/Node/Rust.)

All crates are from crates.io; "good support/actuality" prioritizes recent updates (2025), high downloads, and active maintenance. For missing ones, generic parsers like `serde_*` (JSON/TOML/YAML) are efficient fallbacks. If building a multi-tool, combine with `anyhow` for errors.
