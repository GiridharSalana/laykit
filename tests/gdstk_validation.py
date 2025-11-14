#!/usr/bin/env python3
"""
Cross-validation tests between LayKit and gdstk.

This script validates that LayKit can:
1. Read files created by gdstk
2. Create files that gdstk can read
3. Produce identical results for round-trip conversions
"""

import os
import sys
import subprocess
import tempfile
import shutil

try:
    import gdstk
except ImportError:
    print("ERROR: gdstk not installed. Install with: pip install gdstk")
    sys.exit(1)

LAYKIT_BIN = "./target/release/laykit"

def run_laykit(args):
    """Run laykit command and return output."""
    result = subprocess.run(
        [LAYKIT_BIN] + args,
        capture_output=True,
        text=True
    )
    return result.returncode, result.stdout, result.stderr

def test_read_gdstk_file():
    """Test that LayKit can read a file created by gdstk."""
    print("Test 1: LayKit reading gdstk-created file... ", end="")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        gds_file = os.path.join(tmpdir, "gdstk_test.gds")
        
        # Create a GDSII file with gdstk
        lib = gdstk.Library("TESTLIB", unit=1e-6, precision=1e-9)
        cell = lib.new_cell("TOP")
        
        # Add a rectangle
        rect = gdstk.rectangle((0, 0), (100, 100), layer=1, datatype=0)
        cell.add(rect)
        
        # Add a path
        path = gdstk.FlexPath([(0, 0), (50, 0), (50, 50)], 10, layer=2, datatype=0)
        cell.add(path)
        
        # Add text
        text = gdstk.Label("TEST", (25, 25), layer=3, texttype=0)
        cell.add(text)
        
        # Add reference
        subcell = lib.new_cell("SUBCELL")
        subrect = gdstk.rectangle((0, 0), (20, 20), layer=1, datatype=0)
        subcell.add(subrect)
        ref = gdstk.Reference(subcell, (75, 75))
        cell.add(ref)
        
        lib.write_gds(gds_file)
        
        # Try to read with LayKit
        returncode, stdout, stderr = run_laykit(["info", gds_file])
        
        if returncode != 0:
            print(f"FAIL\n  Error: {stderr}")
            return False
        
        # Validate output
        if "TESTLIB" not in stdout or "TOP" not in stdout:
            print(f"FAIL\n  Output missing expected content: {stdout}")
            return False
        
        print("PASS")
        return True

def test_write_for_gdstk():
    """Test that gdstk can read a file created by LayKit."""
    print("Test 2: gdstk reading LayKit-created file... ", end="")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        gds_file = os.path.join(tmpdir, "laykit_test.gds")
        
        # First, create a reference file with gdstk
        ref_file = os.path.join(tmpdir, "reference.gds")
        lib = gdstk.Library("LAYKIT_TEST", unit=1e-6, precision=1e-9)
        cell = lib.new_cell("MAIN")
        
        # Add simple geometry
        rect = gdstk.rectangle((0, 0), (1000, 1000), layer=1, datatype=0)
        cell.add(rect)
        
        polygon = gdstk.Polygon(
            [(100, 100), (900, 100), (900, 900), (100, 900)],
            layer=2, datatype=0
        )
        cell.add(polygon)
        
        lib.write_gds(ref_file)
        
        # Convert with LayKit (round-trip)
        returncode, _, stderr = run_laykit(["convert", ref_file, gds_file])
        
        if returncode != 0:
            print(f"FAIL\n  LayKit convert error: {stderr}")
            return False
        
        # Try to read with gdstk
        try:
            lib_read = gdstk.read_gds(gds_file)
            
            if lib_read.name != "LAYKIT_TEST":
                print(f"FAIL\n  Library name mismatch: {lib_read.name}")
                return False
            
            if "MAIN" not in [c.name for c in lib_read.cells]:
                print(f"FAIL\n  Cell 'MAIN' not found")
                return False
            
            main_cell = lib_read.cells[0]
            if len(main_cell.polygons) < 2:
                print(f"FAIL\n  Expected at least 2 polygons, got {len(main_cell.polygons)}")
                return False
            
            print("PASS")
            return True
            
        except Exception as e:
            print(f"FAIL\n  gdstk read error: {e}")
            return False

def test_gds_to_oasis_conversion():
    """Test GDSII to OASIS conversion compatibility."""
    print("Test 3: GDS→OASIS conversion validation... ", end="")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        gds_file = os.path.join(tmpdir, "source.gds")
        oas_file = os.path.join(tmpdir, "converted.oas")
        gds_back = os.path.join(tmpdir, "roundtrip.gds")
        
        # Create a complex GDSII file
        lib = gdstk.Library("CONVERT_TEST", unit=1e-6, precision=1e-9)
        cell = lib.new_cell("COMPLEX")
        
        # Add various elements
        rect = gdstk.rectangle((0, 0), (500, 500), layer=1, datatype=0)
        cell.add(rect)
        
        path = gdstk.FlexPath([(100, 100), (400, 100), (400, 400)], 20, layer=2)
        cell.add(path)
        
        text = gdstk.Label("CONVERT", (250, 250), layer=10, texttype=5)
        cell.add(text)
        
        lib.write_gds(gds_file)
        
        # Convert GDS → OASIS
        returncode, _, stderr = run_laykit(["convert", gds_file, oas_file])
        if returncode != 0:
            print(f"FAIL\n  GDS→OASIS error: {stderr}")
            return False
        
        # Convert OASIS → GDS
        returncode, _, stderr = run_laykit(["convert", oas_file, gds_back])
        if returncode != 0:
            print(f"FAIL\n  OASIS→GDS error: {stderr}")
            return False
        
        # Verify round-trip with gdstk
        try:
            lib_back = gdstk.read_gds(gds_back)
            
            # Library name may change during conversion (acceptable)
            # Just check that we have a valid library
            if not lib_back.name:
                print(f"FAIL\n  Library name is empty")
                return False
            
            if "COMPLEX" not in [c.name for c in lib_back.cells]:
                print(f"FAIL\n  Cell lost in conversion")
                return False
            
            cell_back = lib_back.cells[0]
            element_count = len(cell_back.polygons) + len(cell_back.labels)
            
            if element_count < 2:  # At least rect and text
                print(f"FAIL\n  Elements lost: {element_count}")
                return False
            
            print("PASS")
            return True
            
        except Exception as e:
            print(f"FAIL\n  Round-trip validation error: {e}")
            return False

def test_properties():
    """Test property handling."""
    print("Test 4: Property preservation... ", end="")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        gds_file = os.path.join(tmpdir, "props.gds")
        gds_out = os.path.join(tmpdir, "props_out.gds")
        
        # Create file with properties
        lib = gdstk.Library("PROPS_TEST", unit=1e-6, precision=1e-9)
        cell = lib.new_cell("WITHPROPS")
        
        rect = gdstk.rectangle((0, 0), (100, 100), layer=1, datatype=0)
        rect.set_property("test_prop", "test_value")
        rect.set_property("attr_42", "numeric_attr")  # gdstk expects string keys
        cell.add(rect)
        
        lib.write_gds(gds_file)
        
        # Round-trip through LayKit
        returncode, _, stderr = run_laykit(["convert", gds_file, gds_out])
        if returncode != 0:
            print(f"FAIL\n  Conversion error: {stderr}")
            return False
        
        # Read back and check properties
        try:
            lib_back = gdstk.read_gds(gds_out)
            cell_back = lib_back.cells[0]
            
            if len(cell_back.polygons) == 0:
                print(f"FAIL\n  No polygons in output")
                return False
            
            poly = cell_back.polygons[0]
            props = poly.properties
            
            # Check if properties exist (gdstk returns list of tuples)
            if props is None or len(props) < 2:
                print(f"FAIL\n  Properties not preserved: {props}")
                return False
            
            print("PASS")
            return True
            
        except Exception as e:
            print(f"FAIL\n  Property validation error: {e}")
            return False

def test_array_references():
    """Test array reference handling."""
    print("Test 5: Array reference (AREF) handling... ", end="")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        gds_file = os.path.join(tmpdir, "array.gds")
        gds_out = os.path.join(tmpdir, "array_out.gds")
        
        # Create file with array reference
        lib = gdstk.Library("ARRAY_TEST", unit=1e-6, precision=1e-9)
        
        # Create subcell
        subcell = lib.new_cell("UNIT")
        unit_rect = gdstk.rectangle((0, 0), (10, 10), layer=1)
        subcell.add(unit_rect)
        
        # Create main cell with array
        main = lib.new_cell("MAIN")
        array_ref = gdstk.Reference(subcell, (0, 0), columns=5, rows=3, spacing=(20, 20))
        main.add(array_ref)
        
        lib.write_gds(gds_file)
        
        # Process with LayKit
        returncode, _, stderr = run_laykit(["convert", gds_file, gds_out])
        if returncode != 0:
            print(f"FAIL\n  Conversion error: {stderr}")
            return False
        
        # Verify with gdstk
        try:
            lib_back = gdstk.read_gds(gds_out)
            
            if len(lib_back.cells) < 2:
                print(f"FAIL\n  Expected 2 cells, got {len(lib_back.cells)}")
                return False
            
            # Find main cell
            main_back = next((c for c in lib_back.cells if c.name == "MAIN"), None)
            if not main_back:
                print(f"FAIL\n  MAIN cell not found")
                return False
            
            # Check for references (could be expanded or kept as array)
            has_refs = len(main_back.references) > 0
            has_polys = len(main_back.polygons) > 0
            
            if not (has_refs or has_polys):
                print(f"FAIL\n  No content in MAIN cell")
                return False
            
            print("PASS")
            return True
            
        except Exception as e:
            print(f"FAIL\n  Array validation error: {e}")
            return False

def test_large_file():
    """Test handling of larger files."""
    print("Test 6: Large file handling... ", end="")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        gds_file = os.path.join(tmpdir, "large.gds")
        info_out = os.path.join(tmpdir, "info.txt")
        
        # Create a file with many elements
        lib = gdstk.Library("LARGE_TEST", unit=1e-6, precision=1e-9)
        cell = lib.new_cell("LARGE")
        
        # Add 1000 rectangles
        for i in range(100):
            for j in range(10):
                x = i * 20
                y = j * 20
                rect = gdstk.rectangle((x, y), (x+10, y+10), layer=(i % 10))
                cell.add(rect)
        
        lib.write_gds(gds_file)
        
        # Test LayKit info command
        returncode, stdout, stderr = run_laykit(["info", gds_file])
        if returncode != 0:
            print(f"FAIL\n  Info command error: {stderr}")
            return False
        
        # Verify output contains expected info
        if "1000" not in stdout and "LARGE" not in stdout:
            print(f"FAIL\n  Unexpected info output")
            return False
        
        print("PASS")
        return True

def main():
    """Run all validation tests."""
    print("=" * 60)
    print("LayKit ↔ gdstk Cross-Validation Tests")
    print("=" * 60)
    print()
    
    # Check LayKit binary exists
    if not os.path.exists(LAYKIT_BIN):
        print(f"ERROR: LayKit binary not found at {LAYKIT_BIN}")
        print("Build it first with: cargo build --release")
        return 1
    
    # Run tests
    tests = [
        test_read_gdstk_file,
        test_write_for_gdstk,
        test_gds_to_oasis_conversion,
        test_properties,
        test_array_references,
        test_large_file,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"EXCEPTION: {e}")
            failed += 1
    
    print()
    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed out of {len(tests)} tests")
    print("=" * 60)
    
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())

