# Unified I/O

LayKit can load and save GDSII and OASIS through a single API, similar to using `gdstk.read_gds` / `gdstk.read_oas` with automatic format detection.

## Canonical library (`load_library`)

[`load_library`](https://docs.rs/laykit/latest/laykit/fn.load_library.html) returns a [`Library`](https://docs.rs/laykit/latest/laykit/struct.Library.html) normalized to GDSII structures internally, so topology, geometry, and boolean helpers work on any input:

```rust
use laykit::load_library;

let lib = load_library("layout.gds")?; // or .oas
println!("{} cells ({:?})", lib.cell_count(), lib.original_format());
for cell in lib.cells() {
    println!("  {}", cell.name);
}
lib.save("copy.oas")?;
```

## Native representation (`load`)

Use [`load`](https://docs.rs/laykit/latest/laykit/fn.load.html) when you want the on-disk format without conversion:

```rust
use laykit::{load, LayoutFile};

let layout = load("chip.oas")?;
match layout {
    LayoutFile::Oasis(oas) => { /* ... */ }
    LayoutFile::Gdsii(gds) => { /* ... */ }
}
```

## Options

- [`LoadOptions::extension_fallback`](https://docs.rs/laykit/latest/laykit/struct.LoadOptions.html) — infer format from `.gds` / `.oas` when magic bytes are inconclusive (off by default).
- [`SaveOptions::format_hint`](https://docs.rs/laykit/latest/laykit/struct.SaveOptions.html) — force output format when the path has no extension.

## Errors

Operations return [`LaykitError`](https://docs.rs/laykit/latest/laykit/enum.LaykitError.html) (`UnknownFormat`, `Io`, `Parse`) instead of opaque boxed errors.
