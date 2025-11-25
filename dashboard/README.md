# Package Scanner Dashboard

Interactive Streamlit dashboard for visualizing npm package scan results.

## Features

### Main Overview Page

- 📊 Overview metrics (total packages, unique packages, locations, infected packages)
- 🔝 Top 20 most used packages visualization
- ⚠️ Infected packages analysis with charts and detailed tables
- 📈 Package version distribution analysis
- 🔍 Raw data viewer with CSV export

### Language-Specific Pages

- **📦 Node.js Packages**: Top 20 Node packages, version consistency analysis
- **🐍 Python Packages**: Top 20 Python packages, framework detection (Django, Flask, FastAPI, etc.)
- **🦀 Rust Packages**: Top 20 Rust crates, popular crate detection (Serde, Tokio, Clap, etc.)

## Installation

This project uses `uv` for dependency management:

```bash
# Dependencies are already installed if you used uv add
# Otherwise, sync the environment:
uv sync
```

## Usage

Run the dashboard from the `dashboard` directory:

```bash
uv run streamlit run app.py
```

Or specify a custom CSV file path:

```bash
uv run streamlit run app.py
```

Then use the sidebar in the web interface to change the CSV path (default: `../output.csv`).

The dashboard will open in your browser at `http://localhost:8501`.

## CSV Format

The dashboard expects a CSV file with the following columns:

- `package`: Package name
- `version`: Package version
- `location`: Directory location
- `match_package`: Matched infected package name (or "none")
- `match_version`: Matched infected package version (or "none")

## Visualizations

### Overview Page

- Total packages found
- Unique package count
- Number of locations scanned
- Count of infected packages
- Top 20 most frequently found packages (all types)
- Infected packages analysis with pie and bar charts
- Package version distribution analyzer

### Node.js Packages Page

- Top 20 most used Node.js packages
- Version consistency analysis
- Detailed package information with version breakdowns

### Python Packages Page

- Top 20 most used Python packages
- Version consistency analysis
- Framework detection (Django, Flask, FastAPI, Pandas, NumPy, etc.)
- Detailed package information

### Rust Packages Page

- Top 20 most used Rust crates
- Version consistency analysis
- Popular crate category detection (Serde, Tokio, CLI tools, HTTP frameworks, etc.)
- Category distribution visualization
