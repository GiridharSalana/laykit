# Installation

## From Crates.io

Add laykit to your project:

```bash
cargo add laykit
```

Or add it manually to your `Cargo.toml` (check [crates.io/crates/laykit](https://crates.io/crates/laykit) for the latest version):

```toml
[dependencies]
laykit = "0"
```

## From Source

Clone and reference locally:

```bash
git clone https://github.com/giridharsalana/laykit.git
```

```toml
[dependencies]
laykit = { path = "../laykit" }
```

Or pin to a specific release tag:

```toml
[dependencies]
laykit = { git = "https://github.com/giridharsalana/laykit", tag = "vX.Y.Z" }
```

## Verifying Installation

```rust
use laykit::GDSIIFile;

fn main() {
    let gds = GDSIIFile::new("TEST".to_string());
    println!("LayKit is working! Library: {}", gds.library_name);
}
```

```bash
cargo run
# LayKit is working! Library: TEST
```

## Building from Source

```bash
git clone https://github.com/giridharsalana/laykit.git
cd laykit

cargo build --release
cargo test
cargo doc --open
```

## Requirements

- **Rust** — any recent stable release
- **Zero runtime dependencies** — only `std`

## Next Steps

Continue to the [Quick Start](./quick-start.md) guide.
