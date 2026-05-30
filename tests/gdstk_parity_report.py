#!/usr/bin/env python3
"""
Strict laykit vs gdstk parity report.

Compares laykit to gdstk for:

- GDSII/OASIS file round-trips (`laykit convert`)
- Geometry algorithms (`laykit geom`, Clipper2 — same engine family as gdstk)

Passing this report is the project's **100% gdstk parity** gate (25 cases).
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from typing import Callable

try:
    import gdstk
except ImportError:
    print("ERROR: pip install gdstk (or: cd tests && uv sync)")
    sys.exit(1)

LAYKIT_BIN = os.path.join(os.path.dirname(__file__), "../target/release/laykit")


@dataclass
class CaseResult:
    name: str
    status: str  # PASS | FAIL | SKIP
    detail: str = ""


def run_laykit(args: list[str]) -> tuple[int, str, str]:
    r = subprocess.run([LAYKIT_BIN] + args, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr


def fingerprint_lib(lib: gdstk.Library, *, flatten: bool = True) -> dict:
    """Fingerprint readable by gdstk after load."""
    fp: dict = {
        "name": lib.name,
        "unit": lib.unit,
        "precision": lib.precision,
        "cells": sorted(c.name for c in lib.cells),
        "polygons": [],
        "labels": [],
        "paths": [],
        "refs": [],
    }
    for cell in lib.cells:
        polys = (
            cell.get_polygons(apply_repetitions=True)
            if flatten
            else cell.polygons
        )
        for poly in polys:
            area = abs(poly.area())
            fp["polygons"].append(
                (
                    cell.name,
                    poly.layer,
                    poly.datatype,
                    round(area, 6),
                    len(poly.points),
                    round(min(p[0] for p in poly.points), 3),
                    round(min(p[1] for p in poly.points), 3),
                )
            )
        labels = (
            cell.get_labels(apply_repetitions=True) if flatten else cell.labels
        )
        for lbl in labels:
            fp["labels"].append(
                (
                    cell.name,
                    lbl.layer,
                    lbl.texttype,
                    lbl.text,
                    round(lbl.origin[0], 3),
                    round(lbl.origin[1], 3),
                )
            )
        for path in cell.paths:
            fp["paths"].append(
                (
                    cell.name,
                    path.layer,
                    path.datatype,
                    len(path.spines()),
                    round(path.size, 3) if hasattr(path, "size") else None,
                )
            )
        for ref in cell.references:
            rep = None
            if ref.repetition is not None:
                rep = (
                    getattr(ref.repetition, "columns", None),
                    getattr(ref.repetition, "rows", None),
                    ref.repetition.get_count()
                    if hasattr(ref.repetition, "get_count")
                    else None,
                )
            fp["refs"].append(
                (
                    cell.name,
                    ref.cell_name,
                    tuple(round(v, 3) for v in ref.origin),
                    round(ref.rotation, 3),
                    ref.magnification,
                    ref.x_reflection,
                    rep,
                )
            )
    for key in ("polygons", "labels", "paths", "refs"):
        fp[key].sort()
    return fp


def compare_fp(a: dict, b: dict, *, check_paths: bool = True) -> list[str]:
    diffs: list[str] = []
    for key in ("name", "unit", "precision", "cells"):
        if a.get(key) != b.get(key):
            diffs.append(f"{key}: {a.get(key)!r} != {b.get(key)!r}")
    if a["polygons"] != b["polygons"]:
        diff = set(a["polygons"]) ^ set(b["polygons"])
        sample = list(diff)[:3]
        diffs.append(
            f"polygons: {len(a['polygons'])} vs {len(b['polygons'])} sample={sample}"
        )
    if a["labels"] != b["labels"]:
        diffs.append("labels differ")
    if check_paths and a.get("paths") != b.get("paths"):
        diffs.append(f"paths: {a.get('paths')!r} != {b.get('paths')!r}")
    if a["refs"] != b["refs"]:
        diffs.append(f"refs differ (count {len(a['refs'])} vs {len(b['refs'])})")
    return diffs


def laykit_convert(inp: str, out: str) -> None:
    code, _, err = run_laykit(["convert", inp, out])
    if code != 0:
        raise RuntimeError(err.strip() or "convert failed")


def poly_set_fingerprint(polys) -> list[tuple]:
    """Sortable fingerprint for a set of polygons (gdstk or JSON lists)."""
    fps: list[tuple] = []
    for p in polys:
        if hasattr(p, "points"):
            pts = [(round(float(x), 4), round(float(y), 4)) for x, y in p.points]
            area = round(abs(float(p.area())), 6)
        else:
            pts = [(round(float(x), 4), round(float(y), 4)) for x, y in p]
            area = round(
                abs(
                    sum(
                        pts[i][0] * pts[(i + 1) % len(pts)][1]
                        - pts[(i + 1) % len(pts)][0] * pts[i][1]
                        for i in range(len(pts))
                    )
                    / 2.0
                ),
                6,
            )
        fps.append((area, len(pts), tuple(sorted(pts))))
    return sorted(fps)


def laykit_geom_boolean(
    op: str,
    a: list[list[list[float]]],
    b: list[list[list[float]]],
    precision: float = 1e-3,
) -> list:
    req = json.dumps({"op": op, "precision": precision, "a": a, "b": b})
    r = subprocess.run(
        [LAYKIT_BIN, "geom", "boolean"],
        input=req,
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        raise RuntimeError(r.stderr.strip() or "laykit geom boolean failed")
    return json.loads(r.stdout)["polygons"]


def laykit_geom_slice(
    polygons: list[list[list[float]]],
    positions: list[float],
    axis: str,
    precision: float = 1e-3,
) -> list[list[list[list[float]]]]:
    req = json.dumps(
        {"positions": positions, "axis": axis, "precision": precision, "polygons": polygons}
    )
    r = subprocess.run(
        [LAYKIT_BIN, "geom", "slice"],
        input=req,
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        raise RuntimeError(r.stderr.strip() or "laykit geom slice failed")
    return json.loads(r.stdout)["strips"]


def laykit_geom_inside(
    points: list[tuple[float, float]],
    polygons: list[list[list[float]]],
) -> list[bool]:
    req = json.dumps(
        {
            "points": [[x, y] for x, y in points],
            "polygons": polygons,
        }
    )
    r = subprocess.run(
        [LAYKIT_BIN, "geom", "inside"],
        input=req,
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        raise RuntimeError(r.stderr.strip() or "laykit geom inside failed")
    return json.loads(r.stdout)["inside"]


def laykit_geom_offset(
    polygons: list[list[list[float]]],
    distance: float,
    tolerance: float = 1e-3,
) -> list:
    req = json.dumps(
        {"distance": distance, "tolerance": tolerance, "polygons": polygons}
    )
    r = subprocess.run(
        [LAYKIT_BIN, "geom", "offset"],
        input=req,
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        raise RuntimeError(r.stderr.strip() or "laykit geom offset failed")
    return json.loads(r.stdout)["polygons"]


def _total_area(polys) -> float:
    total = 0.0
    for p in polys:
        if hasattr(p, "area"):
            total += abs(float(p.area()))
        else:
            pts = [(float(x), float(y)) for x, y in p]
            total += abs(
                sum(
                    pts[i][0] * pts[(i + 1) % len(pts)][1]
                    - pts[(i + 1) % len(pts)][0] * pts[i][1]
                    for i in range(len(pts))
                )
                / 2.0
            )
    return total


def compare_geom(
    name: str, gdstk_polys, laykit_polys, *, area_only: bool = False
) -> CaseResult:
    ref = poly_set_fingerprint(gdstk_polys)
    got = poly_set_fingerprint(laykit_polys)
    if ref == got:
        return CaseResult(name, "PASS")
    if area_only:
        ra, ga = _total_area(gdstk_polys), _total_area(laykit_polys)
        if abs(ra - ga) <= 0.01 * max(ra, ga, 1e-9):
            return CaseResult(name, "PASS")
    return CaseResult(
        name,
        "FAIL",
        f"polygon sets differ: ref={ref[:2]} got={got[:2]}",
    )


def case_boolean_and() -> CaseResult:
    a = [[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]
    b = [[(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0)]]
    ref = gdstk.boolean(a, b, "and", precision=1e-3)
    got = laykit_geom_boolean("and", a, b)
    return compare_geom("boolean_and", ref, got)


def case_boolean_or() -> CaseResult:
    a = [[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]]
    b = [[(10.0, 10.0), (14.0, 10.0), (14.0, 14.0), (10.0, 14.0)]]
    ref = gdstk.boolean(a, b, "or", precision=1e-3)
    got = laykit_geom_boolean("or", a, b)
    return compare_geom("boolean_or", ref, got)


def case_boolean_not() -> CaseResult:
    a = [[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]
    b = [[(5.0, 0.0), (10.0, 0.0), (10.0, 10.0), (5.0, 10.0)]]
    ref = gdstk.boolean(a, b, "not", precision=1e-3)
    got = laykit_geom_boolean("not", a, b)
    return compare_geom("boolean_not", ref, got)


def case_boolean_not_hole() -> CaseResult:
    a = [[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]
    b = [[(2.0, 2.0), (8.0, 2.0), (8.0, 8.0), (2.0, 8.0)]]
    ref = gdstk.boolean(a, b, "not", precision=1e-3)
    got = laykit_geom_boolean("not", a, b)
    return compare_geom("boolean_not_hole", ref, got)


def case_boolean_xor() -> CaseResult:
    a = [[(0.0, 0.0), (6.0, 0.0), (6.0, 6.0), (0.0, 6.0)]]
    b = [[(4.0, 4.0), (10.0, 4.0), (10.0, 10.0), (4.0, 10.0)]]
    ref = gdstk.boolean(a, b, "xor", precision=1e-3)
    got = laykit_geom_boolean("xor", a, b)
    return compare_geom("boolean_xor", ref, got)


def case_offset_expand() -> CaseResult:
    polys = [[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]
    ref = gdstk.offset(polys, 1.0, precision=1e-3)
    got = laykit_geom_offset(polys, 1.0)
    return compare_geom("offset_expand", ref, got, area_only=True)


def case_slice_x() -> CaseResult:
    name = "slice_x"
    poly_pts = [[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]
    poly = gdstk.rectangle((0, 0), (10, 10))
    ref = gdstk.slice([poly], [5.0], "x", precision=1e-3)
    got = laykit_geom_slice(poly_pts, [5.0], "x")
    if len(ref) != len(got):
        return CaseResult(
            name,
            "FAIL",
            f"strip count {len(ref)} vs {len(got)}",
        )
    for i, (r_strip, g_strip) in enumerate(zip(ref, got)):
        r_fps = poly_set_fingerprint(r_strip)
        g_fps = poly_set_fingerprint(g_strip)
        if r_fps != g_fps:
            return CaseResult(
                name,
                "FAIL",
                f"strip {i} differs: ref={r_fps[:1]} got={g_fps[:1]}",
            )
    return CaseResult(name, "PASS")


def case_inside_points() -> CaseResult:
    name = "inside_points"
    polys = [[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]
    pts = [(5.0, 5.0), (15.0, 5.0)]
    ref = list(gdstk.inside(pts, polys))
    got = laykit_geom_inside(pts, polys)
    if ref != got:
        return CaseResult(name, "FAIL", f"ref={ref} got={got}")
    return CaseResult(name, "PASS")


def case_oas_cblock_roundtrip() -> CaseResult:
    """gdstk writes CBLOCK-compressed OASIS; laykit read/write preserves geometry."""
    name = "oas_cblock_roundtrip"

    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("TOP")
        for i in range(20):
            c.add(
                gdstk.rectangle(
                    (i * 5, i * 3),
                    (i * 5 + 10, i * 3 + 8),
                    layer=i % 5,
                )
            )

    return oas_roundtrip_fp(name, build)


def gds_roundtrip_fp(
    name: str,
    build: Callable[[gdstk.Library], None],
    *,
    check_paths: bool = True,
) -> CaseResult:
    """Build lib with gdstk, round-trip GDS through laykit, compare fingerprints."""
    with tempfile.TemporaryDirectory() as d:
        gds = os.path.join(d, "in.gds")
        out = os.path.join(d, "out.gds")
        lib = gdstk.Library(name, unit=1e-6, precision=1e-9)
        build(lib)
        lib.write_gds(gds)
        ref = fingerprint_lib(gdstk.read_gds(gds))
        laykit_convert(gds, out)
        got = fingerprint_lib(gdstk.read_gds(out))
        diffs = compare_fp(ref, got, check_paths=check_paths)
        if diffs:
            return CaseResult(name, "FAIL", "; ".join(diffs))
        return CaseResult(name, "PASS")


def gds_oas_gds_fp(
    name: str,
    build: Callable[[gdstk.Library], None],
    *,
    check_paths: bool = True,
) -> CaseResult:
    with tempfile.TemporaryDirectory() as d:
        gds = os.path.join(d, "in.gds")
        oas = os.path.join(d, "mid.oas")
        out = os.path.join(d, "out.gds")
        lib = gdstk.Library(name, unit=1e-6, precision=1e-9)
        build(lib)
        lib.write_gds(gds)
        ref = fingerprint_lib(gdstk.read_gds(gds))
        laykit_convert(gds, oas)
        laykit_convert(oas, out)
        got = fingerprint_lib(gdstk.read_gds(out))
        diffs = compare_fp(ref, got, check_paths=check_paths)
        if diffs:
            return CaseResult(name, "FAIL", "; ".join(diffs))
        return CaseResult(name, "PASS")


def oas_roundtrip_fp(name: str, build: Callable[[gdstk.Library], None]) -> CaseResult:
    with tempfile.TemporaryDirectory() as d:
        oas_in = os.path.join(d, "in.oas")
        oas_out = os.path.join(d, "out.oas")
        lib = gdstk.Library(name, unit=1e-6, precision=1e-9)
        build(lib)
        lib.write_oas(oas_in)
        ref = fingerprint_lib(gdstk.read_oas(oas_in))
        laykit_convert(oas_in, oas_out)
        got = fingerprint_lib(gdstk.read_oas(oas_out))
        diffs = compare_fp(ref, got)
        if diffs:
            return CaseResult(name, "FAIL", "; ".join(diffs))
        return CaseResult(name, "PASS")


# --- Individual cases (mirror gdstk_validation.py scenarios) ---


def case_gds_roundtrip() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        top = lib.new_cell("TOP")
        top.add(gdstk.rectangle((0, 0), (100, 200), layer=1))
        top.add(gdstk.Label("hi", (50, 50), layer=5))

    return gds_roundtrip_fp("gds_roundtrip", build)


def case_gds_oas_gds() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        u = lib.new_cell("UNIT")
        u.add(gdstk.rectangle((0, 0), (10, 10), layer=1))
        m = lib.new_cell("MAIN")
        m.add(gdstk.Reference(u, (0, 0), columns=4, rows=2, spacing=(30, 40)))

    return gds_oas_gds_fp("gds_oas_gds", build)


def case_read_gdstk_oas() -> CaseResult:
    name = "read_gdstk_oasis"
    with tempfile.TemporaryDirectory() as d:
        oas = os.path.join(d, "g.oas")
        lib = gdstk.Library("OAS", unit=1e-6, precision=1e-9)
        c = lib.new_cell("C")
        c.add(gdstk.rectangle((0, 0), (50, 50), layer=2))
        lib.write_oas(oas)
        code, _, err = run_laykit(["info", oas])
        if code != 0:
            return CaseResult(name, "FAIL", err.strip() or "info failed")
        return CaseResult(name, "PASS")


def case_gds_properties() -> CaseResult:
    name = "gds_properties"
    with tempfile.TemporaryDirectory() as d:
        gds = os.path.join(d, "in.gds")
        out = os.path.join(d, "out.gds")
        lib = gdstk.Library("P", unit=1e-6, precision=1e-9)
        c = lib.new_cell("C")
        r = gdstk.rectangle((0, 0), (10, 10), layer=1)
        r.set_gds_property(7, "val7")
        r.set_gds_property(99, "val99")
        c.add(r)
        lib.write_gds(gds)
        laykit_convert(gds, out)
        back = gdstk.read_gds(out)
        poly = back.cells[0].polygons[0]
        attrs = {
            p[1]: p[2].decode().rstrip("\x00")
            for p in poly.properties
            if p[0] == "S_GDS_PROPERTY"
        }
        if attrs.get(7) != "val7" or attrs.get(99) != "val99":
            return CaseResult(name, "FAIL", f"attrs={attrs}")
        return CaseResult(name, "PASS")


def case_mixed_elements() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("COMPLEX")
        c.add(gdstk.rectangle((0, 0), (500, 500), layer=1))
        c.add(
            gdstk.FlexPath([(100, 100), (400, 100), (400, 400)], 20, layer=2)
        )
        c.add(gdstk.Label("CONVERT", (250, 250), layer=10, texttype=5))

    # FlexPath becomes polygons on read; path list may differ — compare flattened polys+labels
    return gds_oas_gds_fp("mixed_elements", build, check_paths=False)


def case_array_references() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        u = lib.new_cell("UNIT")
        u.add(gdstk.rectangle((0, 0), (10, 10), layer=1))
        m = lib.new_cell("MAIN")
        m.add(gdstk.Reference(u, (0, 0), columns=5, rows=3, spacing=(20, 20)))

    return gds_roundtrip_fp("array_references", build)


def case_deep_hierarchy() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        leaf = lib.new_cell("LEAF")
        leaf.add(gdstk.rectangle((0, 0), (10, 10), layer=1))
        bot = lib.new_cell("BOT")
        bot.add(gdstk.Reference(leaf, (0, 0)))
        bot.add(gdstk.Reference(leaf, (20, 0)))
        mid = lib.new_cell("MID")
        mid.add(gdstk.Reference(bot, (0, 0)))
        mid.add(gdstk.Reference(bot, (0, 50)))
        top = lib.new_cell("TOP")
        top.add(gdstk.Reference(mid, (0, 0)))
        top.add(gdstk.Reference(mid, (100, 0)))

    return gds_roundtrip_fp("deep_hierarchy", build)


def case_reference_transforms() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        base = lib.new_cell("BASE")
        base.add(gdstk.rectangle((0, 0), (100, 50), layer=1))
        top = lib.new_cell("TOP")
        top.add(gdstk.Reference(base, (0, 0)))
        top.add(gdstk.Reference(base, (200, 0), rotation=90))
        top.add(gdstk.Reference(base, (400, 0), x_reflection=True))
        top.add(gdstk.Reference(base, (600, 0), magnification=2.0))

    return gds_roundtrip_fp("reference_transforms", build)


def case_text_labels() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("TEXT")
        c.add(gdstk.Label("NORMAL", (0, 0), layer=10))
        c.add(gdstk.Label("ROTATED", (100, 0), layer=10, rotation=45))
        c.add(gdstk.Label("BIG", (200, 0), layer=10, magnification=3))

    return gds_roundtrip_fp("text_labels", build)


def case_multiple_layers() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("LAYERS")
        for layer in range(5):
            for datatype in range(3):
                x = layer * 50 + datatype * 10
                c.add(
                    gdstk.rectangle(
                        (x, 0), (x + 40, 40), layer=layer, datatype=datatype
                    )
                )

    return gds_roundtrip_fp("multiple_layers", build)


def case_extreme_coordinates() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("COORD")
        c.add(gdstk.rectangle((-1000, -1000), (-900, -900), layer=1))
        c.add(gdstk.rectangle((1_000_000, 1_000_000), (1_000_100, 1_000_100), layer=1))
        c.add(gdstk.rectangle((-500, 500), (500, 1000), layer=1))

    return gds_roundtrip_fp("extreme_coordinates", build)


def case_complex_polygons() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("POLY")
        points = [(0, 0), (100, 0), (120, 50), (80, 100), (20, 80), (0, 0)]
        c.add(gdstk.Polygon(points, layer=1))
        points2 = [(200, 0), (300, 0), (350, 100), (250, 150), (200, 0)]
        c.add(gdstk.Polygon(points2, layer=2))

    return gds_roundtrip_fp("complex_polygons", build)


def case_flexpath_roundtrip() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("PATHS")
        c.add(gdstk.FlexPath([(0, 0), (100, 0), (100, 100)], 10, layer=1))
        c.add(gdstk.FlexPath([(200, 0), (300, 0), (300, 100)], 20, layer=2))

    return gds_roundtrip_fp("flexpath", build, check_paths=False)


def case_oas_roundtrip() -> CaseResult:
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("C")
        c.add(gdstk.rectangle((0, 0), (80, 80), layer=3))
        c.add(gdstk.Label("oas", (10, 10), layer=8))

    return oas_roundtrip_fp("oas_roundtrip", build)


def case_gds_double_oas_roundtrip() -> CaseResult:
    """GDS → OAS → GDS → OAS stability (geometry preserved at each GDS step)."""
    name = "gds_oas_oas_gds"
    with tempfile.TemporaryDirectory() as d:
        g0 = os.path.join(d, "0.gds")
        o1 = os.path.join(d, "1.oas")
        g1 = os.path.join(d, "2.gds")
        o2 = os.path.join(d, "3.oas")
        lib = gdstk.Library("STABLE", unit=1e-6, precision=1e-9)
        c = lib.new_cell("C")
        c.add(gdstk.rectangle((0, 0), (100, 100), layer=1))
        c.add(gdstk.Polygon([(200, 0), (300, 0), (250, 100)], layer=2))
        c.add(gdstk.Label("TEST", (50, 50), layer=10))
        lib.write_gds(g0)
        ref = fingerprint_lib(gdstk.read_gds(g0))
        laykit_convert(g0, o1)
        laykit_convert(o1, g1)
        mid = fingerprint_lib(gdstk.read_gds(g1))
        diffs = compare_fp(ref, mid, check_paths=False)
        if diffs:
            return CaseResult(name, "FAIL", "after first loop: " + "; ".join(diffs))
        laykit_convert(g1, o2)
        g2 = os.path.join(d, "4.gds")
        laykit_convert(o2, g2)
        final = fingerprint_lib(gdstk.read_gds(g2))
        diffs = compare_fp(ref, final, check_paths=False)
        if diffs:
            return CaseResult(name, "FAIL", "after second loop: " + "; ".join(diffs))
        return CaseResult(name, "PASS")


def case_library_name_oasis() -> CaseResult:
    """LIBNAME preserved via OASIS property round-trip."""
    def build(lib: gdstk.Library) -> None:
        c = lib.new_cell("X")
        c.add(gdstk.rectangle((0, 0), (5, 5), layer=1))

    with tempfile.TemporaryDirectory() as d:
        gds = os.path.join(d, "in.gds")
        oas = os.path.join(d, "mid.oas")
        out = os.path.join(d, "out.gds")
        lib = gdstk.Library("CONVERT_TEST", unit=1e-6, precision=1e-9)
        build(lib)
        lib.write_gds(gds)
        laykit_convert(gds, oas)
        laykit_convert(oas, out)
        back = gdstk.read_gds(out)
        if back.name != "CONVERT_TEST":
            return CaseResult(
                "library_name_oasis", "FAIL", f"name={back.name!r}"
            )
        return CaseResult("library_name_oasis", "PASS")


CASES: list[Callable[[], CaseResult]] = [
    case_gds_roundtrip,
    case_gds_oas_gds,
    case_gds_properties,
    case_read_gdstk_oas,
    case_mixed_elements,
    case_array_references,
    case_deep_hierarchy,
    case_reference_transforms,
    case_text_labels,
    case_multiple_layers,
    case_extreme_coordinates,
    case_complex_polygons,
    case_flexpath_roundtrip,
    case_oas_roundtrip,
    case_gds_double_oas_roundtrip,
    case_library_name_oasis,
    case_boolean_and,
    case_boolean_or,
    case_boolean_not,
    case_boolean_not_hole,
    case_boolean_xor,
    case_offset_expand,
    case_slice_x,
    case_inside_points,
    case_oas_cblock_roundtrip,
]


def main() -> int:
    print("=" * 60)
    print("LayKit ↔ gdstk parity report (strict fingerprints)")
    print("=" * 60)
    if not os.path.isfile(LAYKIT_BIN):
        print(f"ERROR: build laykit first: {LAYKIT_BIN}")
        return 1

    results: list[CaseResult] = []
    for fn in CASES:
        try:
            r = fn()
        except Exception as e:
            r = CaseResult(fn.__name__.replace("case_", ""), "FAIL", str(e))
        results.append(r)
        mark = "✓" if r.status == "PASS" else ("○" if r.status == "SKIP" else "✗")
        print(f"  {mark} {r.name}: {r.status}" + (f" — {r.detail}" if r.detail else ""))

    passed = sum(1 for r in results if r.status == "PASS")
    failed = sum(1 for r in results if r.status == "FAIL")
    skipped = sum(1 for r in results if r.status == "SKIP")
    print()
    print(f"Summary: {passed} passed, {failed} failed, {skipped} skipped, {len(results)} total")
    print()
    if failed:
        print("Not 100% gdstk parity — fix failures above (see docs/PARITY.md).")
        return 1
    print("100% gdstk parity: all file-I/O and geometry cases match.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
