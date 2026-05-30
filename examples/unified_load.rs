// Load GDSII or OASIS with a single API (format auto-detected).

use laykit::load_library;

fn main() -> Result<(), laykit::LaykitError> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "example.gds".to_string());

    let lib = load_library(&path)?;
    println!("Loaded: {}", path);
    println!("  Original format: {:?}", lib.original_format());
    println!("  Library: {}", lib.name());
    println!("  Cells: {}", lib.cell_count());
    println!("  Units: {:.3e} / {:.3e}", lib.units().0, lib.units().1);

    Ok(())
}
