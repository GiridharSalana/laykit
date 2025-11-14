// LayKit CLI Tool
// Command-line interface for GDSII and OASIS file operations

use laykit::format_detection::{detect_format_from_file, FileFormat};
use laykit::{converter, GDSIIFile, OASISFile};
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "convert" => handle_convert(&args[2..]),
        "info" => handle_info(&args[2..]),
        "validate" => handle_validate(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!(
        "LayKit v{} - IC Layout File Format Tool",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("    laykit <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    convert <INPUT> <OUTPUT>    Convert between GDSII and OASIS formats");
    println!("    info <FILE>                 Display file information");
    println!("    validate <FILE>             Validate file format and structure");
    println!("    help                        Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    laykit convert input.gds output.oas");
    println!("    laykit convert input.oas output.gds");
    println!("    laykit info design.gds");
    println!("    laykit validate layout.oas");
}

fn handle_convert(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Error: convert command requires input and output file paths");
        eprintln!("Usage: laykit convert <INPUT> <OUTPUT>");
        process::exit(1);
    }

    let input_path = &args[0];
    let output_path = &args[1];

    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        process::exit(1);
    }

    // Detect input format by reading magic bytes
    let input_format = match detect_format_from_file(input_path) {
        Ok(format) => format,
        Err(e) => {
            eprintln!("Error: Cannot detect input file format: {}", e);
            process::exit(1);
        }
    };

    // Detect output format by reading magic bytes (if file exists) or use extension as hint
    let output_format = if Path::new(output_path).exists() {
        match detect_format_from_file(output_path) {
            Ok(format) => format,
            Err(_) => {
                // If we can't read the file, infer from extension
                let ext = Path::new(output_path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                match ext.as_str() {
                    "gds" => FileFormat::GDSII,
                    "oas" => FileFormat::OASIS,
                    _ => FileFormat::Unknown,
                }
            }
        }
    } else {
        // Output file doesn't exist, infer from extension
        let ext = Path::new(output_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "gds" => FileFormat::GDSII,
            "oas" => FileFormat::OASIS,
            _ => FileFormat::Unknown,
        }
    };

    println!("Converting {} -> {}", input_path, output_path);
    println!("  Input format: {:?}", input_format);
    println!("  Output format: {:?}", output_format);

    let result = match (input_format, output_format) {
        (FileFormat::GDSII, FileFormat::OASIS) => convert_gds_to_oas(input_path, output_path),
        (FileFormat::OASIS, FileFormat::GDSII) => convert_oas_to_gds(input_path, output_path),
        (FileFormat::GDSII, FileFormat::GDSII) => {
            eprintln!("Warning: Both files are GDSII format. Copying...");
            copy_file(input_path, output_path)
        }
        (FileFormat::OASIS, FileFormat::OASIS) => {
            eprintln!("Warning: Both files are OASIS format. Copying...");
            copy_file(input_path, output_path)
        }
        (FileFormat::Unknown, _) => {
            eprintln!("Error: Cannot determine input file format");
            eprintln!("       File does not appear to be valid GDSII or OASIS");
            process::exit(1);
        }
        (_, FileFormat::Unknown) => {
            eprintln!("Error: Cannot determine output file format");
            eprintln!("       Please use .gds or .oas extension for output file");
            process::exit(1);
        }
    };

    match result {
        Ok(_) => {
            println!("✓ Conversion successful!");
            if let Ok(metadata) = fs::metadata(output_path) {
                println!("  Output size: {} bytes", metadata.len());
            }
        }
        Err(e) => {
            eprintln!("✗ Conversion failed: {}", e);
            process::exit(1);
        }
    }
}

fn convert_gds_to_oas(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file(input)?;
    let oasis = converter::gdsii_to_oasis(&gds)?;
    oasis.write_to_file(output)?;
    Ok(())
}

fn convert_oas_to_gds(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let oasis = OASISFile::read_from_file(input)?;
    let gds = converter::oasis_to_gdsii_with_name(&oasis, Some(output))?;
    gds.write_to_file(output)?;
    Ok(())
}

fn copy_file(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::copy(input, output)?;
    Ok(())
}

fn handle_info(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: info command requires a file path");
        eprintln!("Usage: laykit info <FILE>");
        process::exit(1);
    }

    let file_path = &args[0];

    if !Path::new(file_path).exists() {
        eprintln!("Error: File '{}' does not exist", file_path);
        process::exit(1);
    }

    // Detect file format by reading magic bytes
    let format = match detect_format_from_file(file_path) {
        Ok(format) => format,
        Err(e) => {
            eprintln!("Error: Cannot detect file format: {}", e);
            process::exit(1);
        }
    };

    let result = match format {
        FileFormat::GDSII => show_gds_info(file_path),
        FileFormat::OASIS => show_oas_info(file_path),
        FileFormat::Unknown => {
            eprintln!("Error: Unknown file format");
            eprintln!("       File does not appear to be valid GDSII or OASIS");
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error reading file: {}", e);
        process::exit(1);
    }
}

fn show_gds_info(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file(file_path)?;
    let metadata = fs::metadata(file_path)?;

    println!("═══════════════════════════════════════════════════════");
    println!("  GDSII File Information");
    println!("═══════════════════════════════════════════════════════");
    println!();
    println!("File: {}", file_path);
    println!(
        "Size: {} bytes ({:.2} KB)",
        metadata.len(),
        metadata.len() as f64 / 1024.0
    );
    println!();
    println!("Library: {}", gds.library_name);
    println!("Version: {}", gds.version);
    println!(
        "Units: {:.3e} user, {:.3e} database (meters)",
        gds.units.0, gds.units.1
    );
    println!();
    println!("Structures: {}", gds.structures.len());
    println!();

    let mut total_elements = 0;
    let mut element_counts = std::collections::HashMap::new();

    for (idx, structure) in gds.structures.iter().enumerate() {
        println!("  [{}] {}", idx + 1, structure.name);
        println!(
            "      Created: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            structure.creation_time.year,
            structure.creation_time.month,
            structure.creation_time.day,
            structure.creation_time.hour,
            structure.creation_time.minute,
            structure.creation_time.second
        );
        println!("      Elements: {}", structure.elements.len());
        total_elements += structure.elements.len();

        for element in &structure.elements {
            let elem_type = match element {
                laykit::GDSElement::Boundary(_) => "Boundary",
                laykit::GDSElement::Path(_) => "Path",
                laykit::GDSElement::StructRef(_) => "StructRef",
                laykit::GDSElement::ArrayRef(_) => "ArrayRef",
                laykit::GDSElement::Text(_) => "Text",
                laykit::GDSElement::Node(_) => "Node",
                laykit::GDSElement::Box(_) => "Box",
            };
            *element_counts.entry(elem_type).or_insert(0) += 1;
        }
    }

    println!();
    println!("Total Elements: {}", total_elements);
    if !element_counts.is_empty() {
        println!();
        println!("Element Breakdown:");
        let mut counts: Vec<_> = element_counts.iter().collect();
        counts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (elem_type, count) in counts {
            println!("  {:<12} {}", elem_type, count);
        }
    }
    println!();

    Ok(())
}

fn show_oas_info(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let oasis = OASISFile::read_from_file(file_path)?;
    let metadata = fs::metadata(file_path)?;

    println!("═══════════════════════════════════════════════════════");
    println!("  OASIS File Information");
    println!("═══════════════════════════════════════════════════════");
    println!();
    println!("File: {}", file_path);
    println!(
        "Size: {} bytes ({:.2} KB)",
        metadata.len(),
        metadata.len() as f64 / 1024.0
    );
    println!();
    println!("Version: {}", oasis.version);
    println!("Unit: {:.3e} meters", oasis.unit);
    println!();
    println!("Cells: {}", oasis.cells.len());
    println!();

    let mut total_elements = 0;
    let mut element_counts = std::collections::HashMap::new();

    for (idx, cell) in oasis.cells.iter().enumerate() {
        println!("  [{}] {}", idx + 1, cell.name);
        println!("      Elements: {}", cell.elements.len());
        total_elements += cell.elements.len();

        for element in &cell.elements {
            let elem_type = match element {
                laykit::OASISElement::Rectangle(_) => "Rectangle",
                laykit::OASISElement::Polygon(_) => "Polygon",
                laykit::OASISElement::Path(_) => "Path",
                laykit::OASISElement::Trapezoid(_) => "Trapezoid",
                laykit::OASISElement::CTrapezoid(_) => "CTrapezoid",
                laykit::OASISElement::Circle(_) => "Circle",
                laykit::OASISElement::Text(_) => "Text",
                laykit::OASISElement::Placement(_) => "Placement",
            };
            *element_counts.entry(elem_type).or_insert(0) += 1;
        }
    }

    println!();
    println!("Total Elements: {}", total_elements);
    if !element_counts.is_empty() {
        println!();
        println!("Element Breakdown:");
        let mut counts: Vec<_> = element_counts.iter().collect();
        counts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (elem_type, count) in counts {
            println!("  {:<12} {}", elem_type, count);
        }
    }

    if !oasis.names.cell_names.is_empty() {
        println!();
        println!("Name Table: {} cell names", oasis.names.cell_names.len());
    }

    println!();

    Ok(())
}

fn handle_validate(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: validate command requires a file path");
        eprintln!("Usage: laykit validate <FILE>");
        process::exit(1);
    }

    let file_path = &args[0];

    if !Path::new(file_path).exists() {
        eprintln!("Error: File '{}' does not exist", file_path);
        process::exit(1);
    }

    // Detect file format by reading magic bytes
    let format = match detect_format_from_file(file_path) {
        Ok(format) => format,
        Err(e) => {
            eprintln!("Error: Cannot detect file format: {}", e);
            process::exit(1);
        }
    };

    let result = match format {
        FileFormat::GDSII => validate_gds(file_path),
        FileFormat::OASIS => validate_oas(file_path),
        FileFormat::Unknown => {
            eprintln!("Error: Unknown file format");
            eprintln!("       File does not appear to be valid GDSII or OASIS");
            process::exit(1);
        }
    };

    match result {
        Ok(issues) => {
            println!("═══════════════════════════════════════════════════════");
            println!("  Validation Results");
            println!("═══════════════════════════════════════════════════════");
            println!();
            println!("File: {}", file_path);
            println!();

            if issues.is_empty() {
                println!("✓ File is valid - no issues found");
                println!();
            } else {
                println!("⚠ Found {} issue(s):", issues.len());
                println!();
                for (idx, issue) in issues.iter().enumerate() {
                    println!("  [{}] {}", idx + 1, issue);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("✗ Validation failed: {}", e);
            process::exit(1);
        }
    }
}

fn validate_gds(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file(file_path)?;
    let mut issues = Vec::new();

    // Check library name
    if gds.library_name.is_empty() {
        issues.push("Library name is empty".to_string());
    }

    // Check units
    if gds.units.0 <= 0.0 || gds.units.1 <= 0.0 {
        issues.push(format!("Invalid units: ({}, {})", gds.units.0, gds.units.1));
    }

    // Check structures
    if gds.structures.is_empty() {
        issues.push("No structures found in file".to_string());
    }

    for (idx, structure) in gds.structures.iter().enumerate() {
        if structure.name.is_empty() {
            issues.push(format!("Structure {} has empty name", idx));
        }

        // Check for duplicate structure names
        let duplicate_count = gds
            .structures
            .iter()
            .filter(|s| s.name == structure.name)
            .count();
        if duplicate_count > 1 {
            issues.push(format!("Duplicate structure name: '{}'", structure.name));
        }

        // Validate elements
        for (elem_idx, element) in structure.elements.iter().enumerate() {
            match element {
                laykit::GDSElement::Boundary(b) => {
                    if b.xy.len() < 4 {
                        issues.push(format!(
                            "Structure '{}' element {} (Boundary): insufficient points ({} < 4)",
                            structure.name,
                            elem_idx,
                            b.xy.len()
                        ));
                    }
                    if b.xy.first() != b.xy.last() {
                        issues.push(format!(
                            "Structure '{}' element {} (Boundary): not closed",
                            structure.name, elem_idx
                        ));
                    }
                }
                laykit::GDSElement::Path(p) => {
                    if p.xy.len() < 2 {
                        issues.push(format!(
                            "Structure '{}' element {} (Path): insufficient points ({} < 2)",
                            structure.name,
                            elem_idx,
                            p.xy.len()
                        ));
                    }
                }
                laykit::GDSElement::StructRef(sref) => {
                    if !gds.structures.iter().any(|s| s.name == sref.sname) {
                        issues.push(format!(
                            "Structure '{}' element {} (StructRef): references undefined structure '{}'",
                            structure.name, elem_idx, sref.sname
                        ));
                    }
                }
                laykit::GDSElement::ArrayRef(aref) => {
                    if !gds.structures.iter().any(|s| s.name == aref.sname) {
                        issues.push(format!(
                            "Structure '{}' element {} (ArrayRef): references undefined structure '{}'",
                            structure.name, elem_idx, aref.sname
                        ));
                    }
                    if aref.columns == 0 || aref.rows == 0 {
                        issues.push(format!(
                            "Structure '{}' element {} (ArrayRef): invalid dimensions ({}x{})",
                            structure.name, elem_idx, aref.columns, aref.rows
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    Ok(issues)
}

fn validate_oas(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let oasis = OASISFile::read_from_file(file_path)?;
    let mut issues = Vec::new();

    // Check version
    if oasis.version.is_empty() {
        issues.push("Version string is empty".to_string());
    }

    // Check unit
    if oasis.unit <= 0.0 {
        issues.push(format!("Invalid unit: {}", oasis.unit));
    }

    // Check cells
    if oasis.cells.is_empty() {
        issues.push("No cells found in file".to_string());
    }

    for (idx, cell) in oasis.cells.iter().enumerate() {
        if cell.name.is_empty() {
            issues.push(format!("Cell {} has empty name", idx));
        }

        // Check for duplicate cell names
        let duplicate_count = oasis.cells.iter().filter(|c| c.name == cell.name).count();
        if duplicate_count > 1 {
            issues.push(format!("Duplicate cell name: '{}'", cell.name));
        }

        // Validate elements
        for (elem_idx, element) in cell.elements.iter().enumerate() {
            match element {
                laykit::OASISElement::Rectangle(r) => {
                    if r.width == 0 || r.height == 0 {
                        issues.push(format!(
                            "Cell '{}' element {} (Rectangle): invalid dimensions ({}x{})",
                            cell.name, elem_idx, r.width, r.height
                        ));
                    }
                }
                laykit::OASISElement::Polygon(p) => {
                    if p.points.len() < 3 {
                        issues.push(format!(
                            "Cell '{}' element {} (Polygon): insufficient points ({} < 3)",
                            cell.name,
                            elem_idx,
                            p.points.len()
                        ));
                    }
                }
                laykit::OASISElement::Path(p) => {
                    if p.points.len() < 2 {
                        issues.push(format!(
                            "Cell '{}' element {} (Path): insufficient points ({} < 2)",
                            cell.name,
                            elem_idx,
                            p.points.len()
                        ));
                    }
                }
                laykit::OASISElement::Placement(pl) => {
                    if !oasis.cells.iter().any(|c| c.name == pl.cell_name) {
                        issues.push(format!(
                            "Cell '{}' element {} (Placement): references undefined cell '{}'",
                            cell.name, elem_idx, pl.cell_name
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    Ok(issues)
}
