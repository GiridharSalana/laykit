# Installation

There are several ways to add LayKit to your Rust project.

## From Source (Current)

Since LayKit is not yet published to crates.io, you can add it as a git dependency or local path dependency.

### Option 1: Git Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
laykit = { git = "https://github.com/giridharsalana/laykit", branch = "main" }
```

Or pin to a specific tag (see [GitHub releases](https://github.com/giridharsalana/laykit/tags) for available versions):

```toml
[dependencies]
laykit = { git = "https://github.com/giridharsalana/laykit", tag = "vX.Y.Z" }
```

### Option 2: Local Path

Clone the repository and reference it locally:

```bash
git clone https://github.com/giridharsalana/laykit.git
```

Then in your `Cargo.toml`:

```toml
[dependencies]
laykit = { path = "../laykit" }
```

## From Crates.io (Coming Soon)

Once published, you'll be able to add LayKit like any other crate:

```toml
[dependencies]
laykit = "0.2"
```

## Verifying Installation

Create a simple test program to verify the installation:

```rust
use laykit::GDSIIFile;

fn main() {
    let gds = GDSIIFile::new("TEST".to_string());
    println!("LayKit is working! Library: {}", gds.library_name);
}
```

Run it:

```bash
cargo run
```

You should see: `LayKit is working! Library: TEST`

## Building from Source

If you want to contribute or build the library yourself:

```bash
# Clone the repository
git clone https://github.com/giridharsalana/laykit.git
cd laykit

# Build the library
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Dependencies

LayKit has **zero external dependencies** - it only uses Rust's standard library (`std`). This means:

- Faster compilation times
- Smaller binary size
- No dependency conflicts
- Easier security auditing

## Next Steps

Now that LayKit is installed, continue to the [Quick Start](./quick-start.md) guide to write your first program!
