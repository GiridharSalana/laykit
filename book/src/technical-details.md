# Technical Details

Deep dive into the technical implementation of LayKit.

## Binary Format Specifications

### GDSII Binary Format

The GDSII Stream Format uses a record-based binary structure:

#### Record Structure

```
+----------------+----------------+----------------+----------------+
|  Record Length (2 bytes)        |  Record Type   |   Data Type    |
+----------------+----------------+----------------+----------------+
|                        Record Data (variable)                     |
+------------------------------------------------------------------+
```

**Fields:**
- **Length** (2 bytes, big-endian): Total record size including header
- **Record Type** (1 byte): Identifies the record (e.g., BOUNDARY, PATH)
- **Data Type** (1 byte): Data format (integer, real, string, etc.)
- **Data** (variable): Actual data content

#### Important Record Types

| Record Type | Hex | Description |
|-------------|-----|-------------|
| HEADER | 0x00 | File version |
| BGNLIB | 0x01 | Begin library |
| LIBNAME | 0x02 | Library name |
| UNITS | 0x03 | User and database units |
| ENDLIB | 0x04 | End library |
| BGNSTR | 0x05 | Begin structure |
| STRNAME | 0x06 | Structure name |
| ENDSTR | 0x07 | End structure |
| BOUNDARY | 0x08 | Polygon |
| PATH | 0x09 | Wire/trace |
| SREF | 0x0A | Structure reference |
| AREF | 0x0B | Array reference |
| TEXT | 0x0C | Text label |
| LAYER | 0x0D | Layer number |
| DATATYPE | 0x0E | Datatype number |
| XY | 0x10 | Coordinates |

#### Data Types

| Data Type | Value | Description |
|-----------|-------|-------------|
| NO_DATA | 0x00 | No data |
| BIT_ARRAY | 0x01 | Bit array |
| INT2 | 0x02 | 2-byte signed integer |
| INT4 | 0x03 | 4-byte signed integer |
| REAL8 | 0x05 | 8-byte real (custom format) |
| ASCII | 0x06 | ASCII string |

#### GDSII Real8 Format

Custom 8-byte floating point format:

```
 Bit:  63    62-56         55-0
      +----+--------+------------------+
      | S  |  Exp   |     Mantissa     |
      +----+--------+------------------+
       Sign 7 bits      56 bits
```

**Formula:**
```
value = (-1)^S × mantissa × 16^(exponent - 64)
```

**Example Implementation:**
```rust
fn decode_real8(bytes: [u8; 8]) -> f64 {
    let sign = if bytes[0] & 0x80 != 0 { -1.0 } else { 1.0 };
    let exponent = (bytes[0] & 0x7F) as i32 - 64;
    
    let mut mantissa = 0u64;
    for i in 1..8 {
        mantissa = (mantissa << 8) | bytes[i] as u64;
    }
    
    let mantissa_f = mantissa as f64 / (1u64 << 56) as f64;
    sign * mantissa_f * 16.0_f64.powi(exponent)
}
```

### OASIS Binary Format

OASIS uses a more compact, modern binary format:

#### File Structure

```
Magic String: "%SEMI-OASIS\r\n" (13 bytes)
START Record
  - Version
  - Unit
  - Offset table flag
Name Tables
  - CELLNAME records
  - TEXTSTRING records
  - PROPNAME records
Cell Records
  - CELL record (begin)
  - Element records
  - CELL record (end)
END Record
  - Validation information
  - Padding table
```

#### Variable-Length Integer Encoding

Unsigned integers (0-127 in single byte):

```
Value < 128:     0xxxxxxx
Value >= 128:    1xxxxxxx 0yyyyyyy
Value >= 16384:  1xxxxxxx 1yyyyyyy 0zzzzzzz ...
```

**Decoding algorithm:**
```rust
fn read_unsigned_integer<R: Read>(reader: &mut R) -> Result<u64, Error> {
    let mut value = 0u64;
    let mut shift = 0;
    
    loop {
        let byte = read_byte(reader)?;
        value |= ((byte & 0x7F) as u64) << shift;
        
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    
    Ok(value)
}
```

#### Zigzag Encoding (Signed Integers)

Maps signed integers to unsigned:

```
 0 →  0
-1 →  1
 1 →  2
-2 →  3
 2 →  4
-3 →  5
```

**Formula:**
```rust
fn encode_signed(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn decode_signed(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}
```

#### Real Number Encoding

OASIS supports 8 types of real numbers (0-7):

| Type | Description | Example |
|------|-------------|---------|
| 0 | Positive integer | 5 |
| 1 | Negative integer | -5 |
| 2 | Positive reciprocal | 1/5 |
| 3 | Negative reciprocal | -1/5 |
| 4 | Positive ratio | 3/5 |
| 5 | Negative ratio | -3/5 |
| 6 | Float32 (IEEE 754) | - |
| 7 | Float64 (IEEE 754) | - |

LayKit primarily uses type 7 (Float64) for maximum precision.

## Memory Management

### GDSII File in Memory

Typical memory layout:

```rust
GDSIIFile (~100 bytes overhead)
├── version: i16                    (2 bytes)
├── library_name: String            (~50 bytes + string length)
├── units: (f64, f64)              (16 bytes)
└── structures: Vec<GDSStructure>  (24 bytes + contents)
    └── For each structure:
        ├── name: String           (~50 bytes + length)
        ├── times: 2 × GDSTime    (24 bytes)
        └── elements: Vec          (24 bytes + N × element_size)
```

**Element sizes:**
- Boundary: ~80 bytes + (N vertices × 8 bytes)
- Path: ~80 bytes + (N points × 8 bytes)
- Text: ~100 bytes + string length
- StructRef: ~100 bytes
- ArrayRef: ~120 bytes

**Example calculation:**
```
File with:
- 100 structures
- Avg 1000 elements per structure
- Avg 5 vertices per boundary

Memory ≈ 100 + (100 × 200) + (100,000 × 120) + (500,000 × 8)
       ≈ 16 MB
```

### OASIS File in Memory

Generally more compact:

```rust
OASISFile (~100 bytes overhead)
├── version: String                (~50 bytes)
├── unit: f64                      (8 bytes)
├── names: NameTable              (~200 bytes + strings)
│   ├── cell_names: HashMap        (varies)
│   ├── text_strings: HashMap      (varies)
│   └── prop_names: HashMap        (varies)
└── cells: Vec<OASISCell>         (24 bytes + contents)
```

## Performance Characteristics

### Read Performance

**GDSII:**
- O(n) where n = file size
- ~20-30 MB/s on modern hardware
- Bottleneck: System calls, big-endian conversion

**OASIS:**
- O(n) where n = file size  
- ~15-25 MB/s on modern hardware
- Bottleneck: Variable-length integer decoding

### Write Performance

**GDSII:**
- O(n) where n = total elements
- ~25-35 MB/s on modern hardware
- Buffered I/O helps significantly

**OASIS:**
- O(n) where n = total elements
- ~20-30 MB/s on modern hardware
- Variable-length encoding overhead

### Memory Usage

| File Size | Memory Usage | Notes |
|-----------|-------------|-------|
| 1 MB | ~50 MB | Includes data structures |
| 10 MB | ~300 MB | 30× expansion typical |
| 100 MB | ~2.5 GB | May need 64-bit system |
| 1 GB | ~20 GB | Consider streaming |

## Coordinate Systems

### GDSII Coordinates

- **Type:** 32-bit signed integer (`i32`)
- **Range:** -2,147,483,648 to 2,147,483,647
- **Units:** Defined by database unit in file
- **Typical:** 1nm per database unit

**Example:**
```rust
// Units: (1e-6, 1e-9) = 1µm user, 1nm database
// Coordinate 1000 = 1000 database units = 1µm = 0.001mm
```

### OASIS Coordinates

- **Type:** 64-bit signed integer (`i64`)
- **Range:** -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
- **Units:** Defined by unit field (in meters)
- **Typical:** 1nm (1e-9 meters)

**Delta Encoding:**
OASIS often uses relative coordinates for compactness:
```rust
// Absolute: (0,0), (1000,0), (1000,1000), (0,1000)
// Delta:    (0,0), (1000,0), (0,1000), (-1000,0)
// Saves space in compressed format
```

## Error Handling

LayKit uses Rust's `Result` type throughout:

```rust
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
```

**Common error scenarios:**
- File not found → `io::Error`
- Invalid format → `Custom error message`
- Corrupted data → `Parse error`
- Out of memory → `Allocation failure`

## Thread Safety

**Current implementation:**
- ✅ Read-only operations are thread-safe
- ✅ Multiple readers can work simultaneously
- ⚠️ Writes require exclusive access
- ⚠️ No built-in synchronization

**Usage pattern:**
```rust
use std::sync::Arc;
use std::thread;

let gds = Arc::new(GDSIIFile::read_from_file("design.gds")?);

let handles: Vec<_> = (0..4).map(|i| {
    let gds_clone = Arc::clone(&gds);
    thread::spawn(move || {
        // Read-only analysis in parallel
        analyze_structures(&gds_clone, i);
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

## Future Optimizations

Potential improvements:

1. **Memory-mapped I/O** - For very large files
2. **SIMD operations** - For coordinate transformations
3. **Parallel parsing** - Using rayon
4. **Streaming API** - Process files without loading entirely
5. **Zero-copy parsing** - Reduce allocations
6. **Custom allocator** - Arena allocation for elements
