# Infected Package Detection Feature

## Overview

The scanner now supports detection of infected packages (ransomware/worm) using a CSV-based list format that supports multiple versions per package.

## Changes Made

### 1. Renamed from "Vulnerability" to "Infected Package"

- Changed terminology from "vulnerability" to "infected package" to better reflect the use case (ransomware/worm detection)
- Updated all related code, documentation, and CLI flags

### 2. CSV Format Support

The infected package list now uses CSV format instead of simple text format:

**Old Format (text):**

```
react@18.2.0
lodash@4.17.21
```

**New Format (CSV with multiple versions):**

```csv
webpack-loader-httpfile,0.2.1
wenk,1.0.9 | 1.0.10
zapier-async-storage,1.0.3 | 1.0.2 | 1.0.1
zapier-platform-cli,18.0.4 | 18.0.3 | 18.0.2
```

### 3. Security Status Column

Added a new `security` column to CSV output with three possible values:

- **NONE**: Package is not in the infected list
- **MATCH_PACKAGE**: Package name matches but version is different (not infected)
- **INFECTED**: Package name and version match the infected list

### 4. Updated Components

#### `src/analyzer/vuln_filter.rs`

- Renamed to handle "infected packages" instead of "vulnerabilities"
- Changed `VulnerabilityFilter` to `InfectedPackageFilter`
- Changed `VulnerablePackage` to `InfectedPackage`
- Added `SecurityStatus` enum with NONE, MATCH_PACKAGE, INFECTED variants
- Implemented CSV parsing with pipe-separated versions
- Added `get_security_status()` method to determine security status

#### `src/output/csv_writer.rs`

- Added `security` column to CSV output
- Created `write_classified_csv_with_security()` function
- Security status is calculated for all dependencies when infected list is provided

#### `src/main.rs`

- Changed `--vuln-list` flag to `--infected-list`
- Updated to use `InfectedPackageFilter` and `load_from_csv()`
- Added reporting of both infected and matching package counts

## Usage Examples

### Basic Infected Package Scan

```bash
# Create infected package list
cat > infected.csv << EOF
webpack-loader-httpfile,0.2.1
zapier-async-storage,1.0.3 | 1.0.2 | 1.0.1
lodash,4.17.21
EOF

# Run scan
scanner --infected-list infected.csv --output results.csv
```

### CSV Output Example

```csv
package_name,ecosystem,...,security
lodash,node,...,INFECTED
react,node,...,MATCH_PACKAGE
typescript,node,...,NONE
```

### JSON Output Example

```json
{
  "applications": [
    {
      "name": "myapp",
      "dependencies": [
        {
          "name": "lodash",
          "security": "INFECTED",
          ...
        },
        {
          "name": "react",
          "security": "MATCH_PACKAGE",
          ...
        },
        {
          "name": "typescript",
          "security": "NONE",
          ...
        }
      ]
    }
  ]
}
```

### Interpreting Results

1. **INFECTED**: This package version is known to be infected - immediate action required
2. **MATCH_PACKAGE**: Package name matches infected list but version differs - review recommended
3. **NONE**: Package is not in the infected list - no known issues

## CSV Format Specification

### Format

```
package_name,version1 | version2 | version3
```

### Rules

- First column: Package name (required)
- Second column: One or more versions separated by ` | ` (pipe with spaces)
- Comments: Lines starting with `#` are ignored
- Empty lines are ignored

### Example

```csv
# Infected packages detected in supply chain attack
webpack-loader-httpfile,0.2.1
wellness-expert-ng-gallery,5.1.1
wenk,1.0.9 | 1.0.10
zapier-async-storage,1.0.3 | 1.0.2 | 1.0.1
zapier-platform-cli,18.0.4 | 18.0.3 | 18.0.2
```

## Benefits

1. **Multiple Versions**: Support for tracking multiple infected versions of the same package
2. **Clear Status**: Three-tier security status (NONE/MATCH_PACKAGE/INFECTED)
3. **CSV Format**: Easy to maintain and integrate with other tools
4. **Comprehensive Reporting**: Shows all dependencies with their security status
5. **Priority Sorting**: Infected packages are sorted by priority (HAS > SHOULD > CAN)
6. **JSON Support**: Security status is included in JSON output for programmatic analysis

## Testing

All tests pass with the new functionality:

- CSV parsing with multiple versions
- Security status detection
- Filtering and sorting
- Integration with main scan flow

## Sample File

A sample infected packages list is provided at `examples/infected-packages-sample.csv`.

## Migration from Old Format

If you have an old vulnerability list in the format `package@version`, you can convert it to CSV:

```bash
# Old format
cat old_vulns.txt
react@18.2.0
lodash@4.17.21

# Convert to new CSV format
awk -F'@' '{print $1","$2}' old_vulns.txt > infected.csv

# Result
cat infected.csv
react,18.2.0
lodash,4.17.21
```

## Future Enhancements

Potential improvements:

1. Support for version ranges in infected list (e.g., `>=1.0.0,<2.0.0`)
2. Integration with public infected package databases
3. Automatic updates of infected package lists
4. Severity levels (critical, high, medium, low)
5. Remediation suggestions
