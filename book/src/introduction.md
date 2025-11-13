# Introduction

**LayKit** is a production-ready Rust library for reading, writing, and converting between GDSII (`.gds`) and OASIS (`.oas`) file formats used in integrated circuit layout design and electronic design automation (EDA).

## Key Features

### 🔄 Full Format Support
- **Complete GDSII Support** - Read and write all GDSII element types
- **Complete OASIS Support** - Read and write all OASIS element types
- **Bidirectional Conversion** - Convert seamlessly between formats

### 🚀 Performance & Safety
- **Zero Dependencies** - Pure Rust implementation using only `std`
- **Memory Safe** - Leverages Rust's ownership system for safety
- **High Performance** - Efficient binary parsing and serialization
- **Streaming Parser** - Process large files without loading entire file into memory
- **Production Ready** - 53 comprehensive tests, 100% passing

### 📦 Easy to Use
- Simple, intuitive API
- CLI tool for quick operations
- Property utilities and builders
- AREF expansion tools
- Comprehensive documentation
- Multiple examples included
- Type-safe element handling

## What are GDSII and OASIS?

### GDSII Format

GDSII Stream Format is the industry-standard binary file format for describing IC layouts. It's been widely used since the 1980s and is supported by virtually all EDA tools.

**Key characteristics:**
- Binary format with big-endian encoding
- Record-based structure
- Hierarchical cell/structure organization
- Custom 8-byte floating point format (Real8)
- File extension: `.gds`

### OASIS Format

Open Artwork System Interchange Standard (OASIS) is a modern replacement for GDSII, designed to be more compact and efficient.

**Key characteristics:**
- Compact binary format
- Variable-length integer encoding
- More primitive shapes (rectangles, trapezoids, circles)
- Name tables for string compression
- IEEE 754 floating point
- File extension: `.oas`

## Why LayKit?

### For Rust Developers
- **Native Rust Implementation** - No FFI overhead or C dependencies
- **Type Safety** - Catch errors at compile time
- **Modern API** - Idiomatic Rust with Result types
- **Zero Unsafe Code** - (except for necessary bit operations)

### For EDA Applications
- **Complete Support** - All element types and features
- **Reliable Conversion** - Preserves geometry and hierarchy
- **Performance** - Fast enough for production use
- **Well Tested** - Comprehensive test suite

### For IC Layout Tools
- **Easy Integration** - Simple API, minimal dependencies
- **Flexible I/O** - Read/write from files or buffers
- **Memory Efficient** - Reasonable memory usage for typical designs
- **Extensible** - Clear structure for adding features

## Getting Started

Jump right in with the [Getting Started](./getting-started.md) guide, or check out the [Quick Start](./quick-start.md) for immediate code examples.

## Project Status

LayKit v0.1.1 is **production-ready** with:

- ✅ Full GDSII read/write implementation
- ✅ Full OASIS read/write implementation
- ✅ Bidirectional format conversion
- ✅ Streaming parser for large files
- ✅ Command-line tool (convert, info, validate)
- ✅ Property management utilities
- ✅ AREF expansion utilities
- ✅ 53 comprehensive tests (100% passing)
- ✅ Zero compiler warnings
- ✅ Complete documentation
- ✅ Multiple working examples

## License

LayKit is licensed under the MIT License. See the [LICENSE](https://github.com/giridharsalana/laykit/blob/main/LICENSE) file for details.

## Support

- **GitHub Repository**: [giridharsalana/laykit](https://github.com/giridharsalana/laykit)
- **Issue Tracker**: [GitHub Issues](https://github.com/giridharsalana/laykit/issues)
- **Discussions**: [GitHub Discussions](https://github.com/giridharsalana/laykit/discussions)
