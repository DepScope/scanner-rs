# Supply Chain Security Scanning Guide

## Overview

This guide covers how to use Scanner to detect infected packages from supply chain attacks, with a focus on the Shai Hulud attack that compromised hundreds of npm packages.

## What is Shai Hulud?

Shai Hulud was a sophisticated supply chain attack that compromised over 400 npm packages by injecting malicious code. The attack targeted developers and their systems by:

- Stealing environment variables and credentials
- Exfiltrating sensitive data
- Establishing persistence mechanisms
- Potentially deploying ransomware/worm capabilities

## Quick Start: Scanning for Shai Hulud

### 1. Basic System Scan

Scan your entire home directory for infected packages:

```bash
scanner --infected-list shai-hulud2.csv --dir ~ --output shai-hulud-scan.csv
```

### 2. Scan Specific Projects

Scan a specific project directory:

```bash
scanner --infected-list shai-hulud2.csv --dir /path/to/project --output project-scan.csv
```

### 3. Verbose Scan with Progress

See detailed progress during scanning:

```bash
scanner --infected-list shai-hulud2.csv --dir ~ --verbose --output shai-hulud-scan.csv
```

### 4. JSON Output for Automation

Generate JSON output for programmatic analysis:

```bash
scanner --infected-list shai-hulud2.csv --dir ~ --format json --output shai-hulud-scan.json
```

## Understanding the Output

### Security Status Values

The scanner reports three security statuses:

1. **INFECTED** - Package name and version match the infected list
   - **Action Required**: Immediate removal and remediation
   - **Risk Level**: CRITICAL

2. **MATCH_PACKAGE** - Package name matches but version differs
   - **Action Required**: Review and verify version is safe
   - **Risk Level**: MEDIUM

3. **NONE** - Package not in infected list
   - **Action Required**: None
   - **Risk Level**: LOW

### CSV Output Format

```csv
package_name,ecosystem,application_name,application_root,has_version,has_path,should_version,should_path,can_version,can_path,version_mismatch,constraint_violation,parent_package,is_direct,dependency_count,security
@posthog/icons,node,myapp,/home/user/myapp,0.36.1,/home/user/myapp/node_modules/@posthog/icons,0.36.1,/home/user/myapp/package-lock.json,^0.36.0,/home/user/myapp/package.json,false,false,,true,0,INFECTED
react,node,myapp,/home/user/myapp,18.2.0,/home/user/myapp/node_modules/react,18.2.0,/home/user/myapp/package-lock.json,^18.0.0,/home/user/myapp/package.json,false,false,,true,2,NONE
```

### Key Columns for Analysis

- **package_name**: Name of the package
- **has_version**: Actually installed version (most critical)
- **has_path**: Where the package is installed
- **application_root**: Which project uses this package
- **security**: INFECTED, MATCH_PACKAGE, or NONE

## Analyzing Results

### 1. Filter for Infected Packages

```bash
# Show only INFECTED packages
grep "INFECTED" shai-hulud-scan.csv

# Count infected packages
grep -c "INFECTED" shai-hulud-scan.csv
```

### 2. Filter by Classification Priority

The scanner prioritizes by classification:

- **HAS** (installed) - Highest priority, actually present on system
- **SHOULD** (locked) - Medium priority, intended to be installed
- **CAN** (declared) - Lower priority, allowed by version range

```bash
# Show packages that are actually installed (HAS)
awk -F',' '$5 != "" {print $1,$5,$16}' shai-hulud-scan.csv | grep INFECTED
```

### 3. Group by Application

```bash
# Show which applications have infected packages
awk -F',' '$16 == "INFECTED" {print $3,$1,$5}' shai-hulud-scan.csv | sort -u
```

### 4. Find All Affected Paths

```bash
# List all installation paths for infected packages
awk -F',' '$16 == "INFECTED" && $6 != "" {print $6}' shai-hulud-scan.csv
```

## Remediation Steps

### For INFECTED Packages

1. **Immediate Actions**:

   ```bash
   # Stop any running applications using the infected package
   # Document the infected package details
   grep "INFECTED" shai-hulud-scan.csv > infected-packages.txt
   ```

2. **Remove Infected Packages**:

   ```bash
   # For Node.js projects
   cd /path/to/project
   rm -rf node_modules
   rm package-lock.json

   # Update package.json to exclude infected versions
   # Then reinstall
   npm install
   ```

3. **Verify Removal**:

   ```bash
   # Scan again to confirm removal
   scanner --infected-list shai-hulud2.csv --dir /path/to/project --output verification-scan.csv
   grep "INFECTED" verification-scan.csv
   ```

### For MATCH_PACKAGE Status

1. **Verify Version Safety**:
   - Check if your version is before or after the infected versions
   - Review the package's changelog and security advisories
   - Consider updating to a known-safe version

2. **Update if Necessary**:

   ```bash
   # Update to latest safe version
   npm update package-name

   # Or specify a safe version
   npm install package-name@safe-version
   ```

## Advanced Analysis

### Using JSON Output

```bash
# Generate JSON output
scanner --infected-list shai-hulud2.csv --dir ~ --format json --output scan.json

# Use jq to analyze (install jq if needed: brew install jq)
# Find all infected packages
jq '.[] | .dependencies[] | select(.security == "INFECTED") | {name, version: .classifications.has.version, path: .classifications.has.path}' scan.json

# Count by security status
jq '[.[] | .dependencies[] | .security] | group_by(.) | map({status: .[0], count: length})' scan.json
```

### Scan Multiple Directories

```bash
# Scan multiple project directories
for dir in ~/projects/*; do
  echo "Scanning $dir..."
  scanner --infected-list shai-hulud2.csv --dir "$dir" --output "$(basename $dir)-scan.csv"
done

# Combine results
cat *-scan.csv | grep "INFECTED" > all-infected.csv
```

### Monitor Specific Ecosystems

```bash
# Scan only Node.js packages
scanner --infected-list shai-hulud2.csv --dir ~ --ecosystem node --output node-scan.csv

# Scan only Python packages
scanner --infected-list shai-hulud2.csv --dir ~ --ecosystem python --output python-scan.csv
```

## Shai Hulud Infected Packages

The `shai-hulud2.csv` file contains over 400 infected packages including:

### High-Profile Packages

- `@posthog/*` - Multiple PostHog packages (analytics platform)
- `@voiceflow/*` - Voiceflow SDK packages
- `@zapier/*` - Zapier integration packages
- `@ensdomains/*` - ENS (Ethereum Name Service) packages
- `posthog-js`, `posthog-node` - Core PostHog libraries

### Common Patterns

- Scoped packages (`@org/package`)
- Multiple versions per package (e.g., `1.0.1 | 1.0.2 | 1.0.3`)
- Development tools and CLI packages
- React Native components
- Build tools and plugins

## Best Practices

### 1. Regular Scanning

Set up regular scans of your systems:

```bash
# Add to cron (daily scan)
0 2 * * * /usr/local/bin/scanner --infected-list /path/to/shai-hulud2.csv --dir ~ --output ~/scans/daily-$(date +\%Y\%m\%d).csv
```

### 2. CI/CD Integration

Add scanning to your CI/CD pipeline:

```yaml
# GitHub Actions example
- name: Scan for infected packages
  run: |
    scanner --infected-list shai-hulud2.csv --dir . --output scan-results.csv
    if grep -q "INFECTED" scan-results.csv; then
      echo "Infected packages found!"
      grep "INFECTED" scan-results.csv
      exit 1
    fi
```

### 3. Team Communication

When infected packages are found:

1. **Document**: Save scan results with timestamps
2. **Notify**: Alert team members immediately
3. **Coordinate**: Plan remediation across all affected systems
4. **Verify**: Confirm removal on all systems

### 4. Post-Remediation

After removing infected packages:

1. **Rotate Credentials**: Change any credentials that may have been exposed
2. **Review Logs**: Check for suspicious activity
3. **Update Dependencies**: Ensure all packages are up-to-date
4. **Scan Again**: Verify complete removal

## Incident Response Checklist

- [ ] Run initial scan: `scanner --infected-list shai-hulud2.csv --dir ~ --output initial-scan.csv`
- [ ] Identify infected packages: `grep "INFECTED" initial-scan.csv > infected.txt`
- [ ] Document affected applications and paths
- [ ] Stop affected applications
- [ ] Remove infected packages
- [ ] Update to safe versions
- [ ] Verify removal with second scan
- [ ] Rotate exposed credentials
- [ ] Review system logs for suspicious activity
- [ ] Update security policies and procedures
- [ ] Document lessons learned

## Additional Resources

### Understanding the Attack

- Review the infected package list: `shai-hulud2.csv`
- Check npm advisories: <https://www.npmjs.com/advisories>
- Monitor security bulletins from package maintainers

### Prevention

1. **Use Lock Files**: Always commit `package-lock.json`, `yarn.lock`, or `pnpm-lock.yaml`
2. **Audit Dependencies**: Regularly run `npm audit` or `yarn audit`
3. **Pin Versions**: Use exact versions for critical dependencies
4. **Review Changes**: Check package updates before installing
5. **Use Private Registry**: Consider using a private npm registry with scanning

### Tools and Commands

```bash
# Check npm audit
npm audit

# Check for outdated packages
npm outdated

# Update packages safely
npm update --save

# Clean install
rm -rf node_modules package-lock.json
npm install

# Verify package integrity
npm ls
```

## Support and Reporting

If you discover infected packages not in the list:

1. Document the package name and version
2. Save the scan results
3. Report to the package maintainer
4. Report to npm security: <security@npmjs.com>
5. Consider creating a GitHub issue with details

## Conclusion

Supply chain attacks like Shai Hulud demonstrate the importance of:

- Regular security scanning
- Dependency management
- Rapid incident response
- Team coordination

Use this scanner regularly to protect your systems and respond quickly to emerging threats.

---

**Last Updated**: November 2024
**Scanner Version**: 0.1.3
**Shai Hulud List Version**: 2.0
