# CLI Tool

LayKit includes a command-line tool for quick file operations without writing code.

## Installation

The CLI tool is automatically built when you build the project:

```bash
cargo build --release
```

The binary will be available at `target/release/laykit`.

## Commands

### Convert

Convert between GDSII and OASIS formats:

```bash
# GDSII to OASIS
laykit convert input.gds output.oas

# OASIS to GDSII
laykit convert input.oas output.gds
```

The format is automatically detected from file extensions.

### Info

Display detailed information about a layout file:

```bash
laykit info design.gds
```

Output includes:
- File size and format
- Library/cell names
- Structure count
- Element counts by type
- Creation timestamps

Example output:

```
═══════════════════════════════════════════════════════
  GDSII File Information
═══════════════════════════════════════════════════════

File: design.gds
Size: 15234 bytes (14.88 KB)

Library: MY_LIBRARY
Version: 600
Units: 1.000e-06 user, 1.000e-09 database (meters)

Structures: 5

  [1] TOP
      Created: 2025-01-15 14:30:00
      Elements: 145
  [2] SUBCELL_A
      Created: 2025-01-15 14:30:00
      Elements: 23
  ...

Total Elements: 312

Element Breakdown:
  Boundary     156
  Path          89
  Text          45
  StructRef     22
```

### Validate

Validate file structure and check for common issues:

```bash
laykit validate layout.gds
```

The validator checks for:
- Empty library/structure names
- Invalid unit values
- Unclosed boundaries
- Paths with insufficient points
- Undefined structure references
- Invalid array dimensions
- Duplicate structure names

Example output:

```
═══════════════════════════════════════════════════════
  Validation Results
═══════════════════════════════════════════════════════

File: layout.gds

⚠ Found 2 issue(s):

  [1] Structure 'TOP' element 5 (Boundary): not closed
  [2] Structure 'CELL2': references undefined structure 'MISSING'
```

### Help

Show usage information:

```bash
laykit help
# or
laykit --help
```

## Use Cases

### Quick Format Conversion

```bash
# Convert all GDS files to OASIS
for file in *.gds; do
    laykit convert "$file" "${file%.gds}.oas"
done
```

### Batch Validation

```bash
# Validate all layout files
for file in *.gds *.oas; do
    echo "Validating $file"
    laykit validate "$file"
done
```

### File Inspection

```bash
# Get quick info about multiple files
for file in designs/*.gds; do
    echo "=== $file ==="
    laykit info "$file" | grep -E "Structures:|Total Elements:"
done
```

## Error Handling

The CLI tool returns appropriate exit codes:
- `0` - Success
- `1` - Error (file not found, invalid format, etc.)

This makes it suitable for use in scripts and CI/CD pipelines:

```bash
if laykit validate input.gds; then
    echo "File is valid"
    laykit convert input.gds output.oas
else
    echo "Validation failed"
    exit 1
fi
```

## Performance

The CLI tool is optimized for:
- Fast startup (no heavy initialization)
- Efficient memory usage
- Streaming I/O where possible
- Minimal overhead for small files

For very large files (>1GB), consider using the streaming parser API directly in your Rust code.

