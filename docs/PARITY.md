# LayKit ↔ gdstk — 100% parity gate

**100% gdstk parity** means: every automated case in `tests/gdstk_parity_report.py`
passes. That script is the single source of truth.

```bash
cargo build --release
cd tests && uv sync && uv run python3 gdstk_parity_report.py
```

Exit code **0** ⇒ 100% parity for the covered surface. Exit code **1** ⇒ not there yet.

## What the gate covers (25 cases)

### File I/O (17 cases)

GDS/OAS read, write, and convert with strict fingerprints after reload in gdstk:

- Units, library name, properties
- Hierarchy, transforms, AREF/repetition grids
- FlexPath (as flattened polygons), labels, complex polygons
- OASIS modal geometry + CBLOCK read/write (gdstk-style raw deflate)
- Double GDS↔OAS round-trip stability

Flattened mask geometry must match; on-disk encoding may differ (e.g. AREF as OASIS matrix placement).

### Geometry (8 cases)

Same **Clipper2 / PolyTree** family as gdstk (`clipper_tools.cpp`: `boolean`, `offset`, `slice`):

| Case | gdstk API | laykit API |
|------|-----------|------------|
| `boolean_and` | `gdstk.boolean(..., "and")` | `laykit geom boolean` |
| `boolean_or` | `"or"` | same |
| `boolean_not` | `"not"` (partial overlap) | same |
| `boolean_not_hole` | `"not"` (hole inside host) | PolyTree + `link_holes` |
| `boolean_xor` | `"xor"` | same |
| `offset_expand` | `gdstk.offset` | `laykit geom offset` |
| `slice_x` | `gdstk.slice(..., "x")` | `laykit geom slice` |
| `inside_points` | `gdstk.inside` | `laykit geom inside` |

Default precision: **1e-3** (gdstk default).

`offset_expand` matches on **total area** (rounded joins may use more vertices than gdstk).

## Also run (smoke / Rust)

```bash
cd tests && uv run python3 gdstk_validation.py   # 14 interoperability smokes
cargo test --test tests                           # Rust integration
cargo test --lib                                  # unit tests
```

## Outside the 100% gate (not claimed)

| Item | Notes |
|------|--------|
| Python `import gdstk` API | LayKit is Rust + CLI; no drop-in Python module |
| `FlexPath` / `RobustPath` construction | File parity uses flattened polygons only |
| Every gdstk helper (`fracture`, `chop`, …) | Add a `case_*` to extend the gate |
| Every PDK OASIS file on earth | Gate uses representative gdstk-generated fixtures |
| Byte-identical GDS/OAS files | Semantic match via gdstk reload, not hex equality |

To **extend** the gate: add a `case_*` function and list it in `CASES` inside `gdstk_parity_report.py`.
