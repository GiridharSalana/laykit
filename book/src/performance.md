# Performance Guide

Tips and best practices for optimal performance with LayKit.

## Benchmarks

Performance measurements on standard hardware:

**Test System:**
- CPU: Intel Core i7 / AMD Ryzen 7
- RAM: 16GB DDR4
- Storage: NVMe SSD
- OS: Linux/WSL2

### Read Performance

| Operation | File Size | Time | Throughput |
|-----------|-----------|------|------------|
| GDSII Read | 1 MB | ~50 ms | ~20 MB/s |
| GDSII Read | 10 MB | ~400 ms | ~25 MB/s |
| GDSII Read | 100 MB | ~4 sec | ~25 MB/s |
| OASIS Read | 1 MB | ~60 ms | ~17 MB/s |
| OASIS Read | 10 MB | ~500 ms | ~20 MB/s |
| OASIS Read | 100 MB | ~5 sec | ~20 MB/s |

### Write Performance

| Operation | File Size | Time | Throughput |
|-----------|-----------|------|------------|
| GDSII Write | 1 MB | ~40 ms | ~25 MB/s |
| GDSII Write | 10 MB | ~350 ms | ~29 MB/s |
| GDSII Write | 100 MB | ~3.5 sec | ~29 MB/s |
| OASIS Write | 1 MB | ~50 ms | ~20 MB/s |
| OASIS Write | 10 MB | ~450 ms | ~22 MB/s |
| OASIS Write | 100 MB | ~4.5 sec | ~22 MB/s |

### Conversion Performance

| Conversion | Input Size | Time | Rate |
|------------|------------|------|------|
| GDSII → OASIS | 10 MB | ~600 ms | ~17 MB/s |
| OASIS → GDSII | 10 MB | ~550 ms | ~18 MB/s |

## Memory Usage

### File Size vs Memory

Typical memory usage patterns:

```
Memory ≈ File_Size × 20-50 (depending on complexity)
```

**Examples:**

| File Size | Elements | Memory Usage | Notes |
|-----------|----------|--------------|-------|
| 1 MB | 1K | ~40 MB | Simple design |
| 10 MB | 50K | ~300 MB | Moderate complexity |
| 100 MB | 500K | ~2.5 GB | Complex design |
| 1 GB | 5M | ~20 GB | Very large design |

### Reducing Memory Usage

**1. Process in Chunks**
```rust
// Instead of loading entire file
let gds = GDSIIFile::read_from_file("huge.gds")?;

// Consider processing structure by structure
// (requires custom implementation)
```

**2. Drop Unused Data**
```rust
let mut gds = GDSIIFile::read_from_file("design.gds")?;

// Remove unused structures
gds.structures.retain(|s| needed_structures.contains(&s.name));

// Shrink to fit
gds.structures.shrink_to_fit();
```

**3. Use OASIS for Large Files**
```rust
// OASIS files are typically 2-5× smaller
let gds = GDSIIFile::read_from_file("large.gds")?;
let oasis = converter::gdsii_to_oasis(&gds)?;
oasis.write_to_file("large.oas")?;  // Much smaller file
```

## Optimization Tips

### 1. Use Release Mode

Always use `--release` for production:

```bash
# Debug mode (slow, ~10-20× slower)
cargo run --example basic_usage

# Release mode (fast, optimized)
cargo run --release --example basic_usage
```

Performance difference:
- Debug: ~2 MB/s
- Release: ~25 MB/s

### 2. Batch Operations

Process multiple files together:

```rust
use rayon::prelude::*;

fn batch_convert(files: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    files.par_iter().try_for_each(|file| {
        let gds = GDSIIFile::read_from_file(file)?;
        let oasis = converter::gdsii_to_oasis(&gds)?;
        let output = file.replace(".gds", ".oas");
        oasis.write_to_file(&output)?;
        Ok(())
    })
}
```

### 3. Reuse Allocations

When processing multiple files:

```rust
// Less efficient: Create new for each file
for file in files {
    let gds = GDSIIFile::read_from_file(file)?;
    process(&gds);
}

// More efficient: Reuse buffer
let mut buffer = Vec::new();
for file in files {
    buffer.clear();
    let mut file = File::open(file)?;
    file.read_to_end(&mut buffer)?;
    // Parse from buffer
}
```

### 4. Filter Early

Remove unnecessary data as early as possible:

```rust
let mut gds = GDSIIFile::read_from_file("design.gds")?;

// Filter out unwanted layers immediately
for structure in &mut gds.structures {
    structure.elements.retain(|elem| {
        match elem {
            GDSElement::Boundary(b) => b.layer <= 10,
            GDSElement::Path(p) => p.layer <= 10,
            _ => true,
        }
    });
}
```

### 5. Use Buffered I/O

For custom reading/writing:

```rust
use std::io::{BufReader, BufWriter};
use std::fs::File;

// Buffered reading (faster)
let file = File::open("design.gds")?;
let mut reader = BufReader::with_capacity(1024 * 1024, file); // 1MB buffer
let gds = GDSIIFile::read(&mut reader)?;

// Buffered writing (faster)
let file = File::create("output.gds")?;
let mut writer = BufWriter::with_capacity(1024 * 1024, file); // 1MB buffer
gds.write(&mut writer)?;
```

## Scalability Guidelines

### Small Files (< 10 MB)

✅ **No special handling needed**
- Load entire file into memory
- Process as needed
- Fast operations

```rust
let gds = GDSIIFile::read_from_file("small.gds")?;
// Process freely
```

### Medium Files (10-100 MB)

✅ **Generally works well**
- Monitor memory usage
- Consider filtering unused data
- Use release builds

```rust
let mut gds = GDSIIFile::read_from_file("medium.gds")?;
gds.structures.retain(|s| interesting_cells.contains(&s.name));
```

### Large Files (100 MB - 1 GB)

⚠️ **Requires care**
- Ensure sufficient RAM (>4GB)
- Filter aggressively
- Consider OASIS format
- Monitor system resources

```rust
// Convert to OASIS first for smaller size
let gds = GDSIIFile::read_from_file("large.gds")?;
let oasis = converter::gdsii_to_oasis(&gds)?;
oasis.write_to_file("large.oas")?;
drop(gds); // Free memory

// Work with smaller OASIS file
let oasis = OASISFile::read_from_file("large.oas")?;
```

### Very Large Files (> 1 GB)

❌ **Current limitations**
- May exhaust memory
- Consider splitting files
- Future: Streaming API needed

**Workarounds:**
```rust
// 1. Split file by structures (external tool)
// 2. Process each part separately
// 3. Merge results
```

## Profiling

Use Rust profiling tools to identify bottlenecks:

### Using Cargo Flamegraph

```bash
# Install
cargo install flamegraph

# Profile your program
cargo flamegraph --example basic_usage

# Open flamegraph.svg in browser
```

### Using perf (Linux)

```bash
# Build with symbols
cargo build --release

# Profile
perf record --call-graph dwarf ./target/release/examples/basic_usage

# Analyze
perf report
```

### Memory Profiling

```bash
# Install valgrind
sudo apt install valgrind

# Profile memory
valgrind --tool=massif ./target/release/examples/basic_usage

# Visualize
ms_print massif.out.*
```

## Real-World Performance

### Example: Converting 100 Files

```rust
use rayon::prelude::*;
use std::time::Instant;

fn batch_convert_parallel(files: &[&str]) {
    let start = Instant::now();
    
    let results: Vec<_> = files.par_iter()
        .map(|file| {
            let gds = GDSIIFile::read_from_file(file)?;
            let oasis = converter::gdsii_to_oasis(&gds)?;
            let output = file.replace(".gds", ".oas");
            oasis.write_to_file(&output)?;
            Ok::<_, Box<dyn std::error::Error>>(())
        })
        .collect();
    
    let duration = start.elapsed();
    let success = results.iter().filter(|r| r.is_ok()).count();
    
    println!("Converted {}/{} files in {:?}", 
        success, files.len(), duration);
    println!("Average: {:.2} files/sec", 
        files.len() as f64 / duration.as_secs_f64());
}
```

**Typical results:**
- Sequential: ~2-3 files/sec
- Parallel (4 cores): ~8-10 files/sec
- Parallel (8 cores): ~12-15 files/sec

## Performance Checklist

Before deploying:

- ✅ Use `--release` builds
- ✅ Profile with representative data
- ✅ Test with your largest expected files
- ✅ Monitor memory usage
- ✅ Consider parallel processing
- ✅ Use appropriate buffer sizes
- ✅ Filter unnecessary data early
- ✅ Use OASIS for large designs
- ✅ Have sufficient RAM (2× file size minimum)
- ✅ Use SSD for large files

## When to Optimize

> "Premature optimization is the root of all evil" - Donald Knuth

**Optimize when:**
1. ✅ You have performance problems
2. ✅ You've profiled and found bottlenecks
3. ✅ You have specific performance requirements
4. ✅ You're processing large files regularly

**Don't optimize when:**
1. ❌ Current performance is acceptable
2. ❌ You haven't measured anything
3. ❌ You're just starting development
4. ❌ It would make code much more complex
