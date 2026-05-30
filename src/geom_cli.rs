//! JSON geometry CLI for gdstk parity tests (`laykit geom ...`).

use crate::{
    Axis, BooleanOp, boolean_with_precision, inside, offset_with_precision,
    slice_at_positions_with_precision,
};
use std::io::{self, Read, Write};

#[derive(serde::Deserialize)]
struct BooleanRequest {
    op: String,
    precision: Option<f64>,
    a: Vec<Vec<[f64; 2]>>,
    b: Vec<Vec<[f64; 2]>>,
}

#[derive(serde::Deserialize)]
struct OffsetRequest {
    distance: f64,
    tolerance: Option<f64>,
    precision: Option<f64>,
    polygons: Vec<Vec<[f64; 2]>>,
}

#[derive(serde::Deserialize)]
struct SliceRequest {
    positions: Vec<f64>,
    axis: String,
    precision: Option<f64>,
    polygons: Vec<Vec<[f64; 2]>>,
}

#[derive(serde::Deserialize)]
struct InsideRequest {
    points: Vec<[f64; 2]>,
    polygons: Vec<Vec<[f64; 2]>>,
}

#[derive(serde::Serialize)]
struct BooleanResponse {
    polygons: Vec<Vec<[f64; 2]>>,
}

#[derive(serde::Serialize)]
struct SliceResponse {
    strips: Vec<Vec<Vec<[f64; 2]>>>,
}

#[derive(serde::Serialize)]
struct InsideResponse {
    inside: Vec<bool>,
}

fn parse_op(s: &str) -> Result<BooleanOp, String> {
    match s.to_lowercase().as_str() {
        "or" => Ok(BooleanOp::Or),
        "and" => Ok(BooleanOp::And),
        "not" => Ok(BooleanOp::Not),
        "xor" => Ok(BooleanOp::Xor),
        _ => Err(format!("unknown op {s:?}")),
    }
}

fn parse_axis(s: &str) -> Result<Axis, String> {
    match s.to_lowercase().as_str() {
        "x" => Ok(Axis::X),
        "y" => Ok(Axis::Y),
        _ => Err(format!("unknown axis {s:?}")),
    }
}

fn to_polys(raw: Vec<Vec<[f64; 2]>>) -> Vec<Vec<(f64, f64)>> {
    raw.into_iter()
        .map(|p| p.into_iter().map(|pt| (pt[0], pt[1])).collect())
        .collect()
}

fn from_polys(polys: Vec<Vec<(f64, f64)>>) -> Vec<Vec<[f64; 2]>> {
    polys
        .into_iter()
        .map(|p| p.into_iter().map(|(x, y)| [x, y]).collect())
        .collect()
}

/// Run `laykit geom <subcommand>` reading JSON from stdin.
pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: laykit geom <boolean|offset|slice|inside>");
        return 1;
    }

    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("Failed to read stdin");
        return 1;
    }

    match args[0].as_str() {
        "boolean" => match serde_json::from_str::<BooleanRequest>(&input) {
            Ok(req) => {
                let precision = req.precision.unwrap_or(1e-3);
                let op = match parse_op(&req.op) {
                    Ok(o) => o,
                    Err(e) => {
                        eprintln!("{e}");
                        return 1;
                    }
                };
                let result =
                    boolean_with_precision(&to_polys(req.a), &to_polys(req.b), op, precision);
                let resp = BooleanResponse {
                    polygons: from_polys(result),
                };
                if serde_json::to_writer(io::stdout().lock(), &resp).is_err() {
                    return 1;
                }
                let _ = io::stdout().lock().write_all(b"\n");
                0
            }
            Err(e) => {
                eprintln!("JSON parse error: {e}");
                1
            }
        },
        "offset" => match serde_json::from_str::<OffsetRequest>(&input) {
            Ok(req) => {
                let tol = req.tolerance.unwrap_or(1e-3);
                let precision = req.precision.unwrap_or(1e-3);
                let result =
                    offset_with_precision(&to_polys(req.polygons), req.distance, tol, precision);
                let resp = BooleanResponse {
                    polygons: from_polys(result),
                };
                if serde_json::to_writer(io::stdout().lock(), &resp).is_err() {
                    return 1;
                }
                let _ = io::stdout().lock().write_all(b"\n");
                0
            }
            Err(e) => {
                eprintln!("JSON parse error: {e}");
                1
            }
        },
        "slice" => match serde_json::from_str::<SliceRequest>(&input) {
            Ok(req) => {
                let precision = req.precision.unwrap_or(1e-3);
                let axis = match parse_axis(&req.axis) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("{e}");
                        return 1;
                    }
                };
                let strips = slice_at_positions_with_precision(
                    &to_polys(req.polygons),
                    &req.positions,
                    axis,
                    precision,
                );
                let resp = SliceResponse {
                    strips: strips.into_iter().map(from_polys).collect(),
                };
                if serde_json::to_writer(io::stdout().lock(), &resp).is_err() {
                    return 1;
                }
                let _ = io::stdout().lock().write_all(b"\n");
                0
            }
            Err(e) => {
                eprintln!("JSON parse error: {e}");
                1
            }
        },
        "inside" => match serde_json::from_str::<InsideRequest>(&input) {
            Ok(req) => {
                let points: Vec<(f64, f64)> =
                    req.points.into_iter().map(|p| (p[0], p[1])).collect();
                let flags = inside(&points, &to_polys(req.polygons));
                let resp = InsideResponse { inside: flags };
                if serde_json::to_writer(io::stdout().lock(), &resp).is_err() {
                    return 1;
                }
                let _ = io::stdout().lock().write_all(b"\n");
                0
            }
            Err(e) => {
                eprintln!("JSON parse error: {e}");
                1
            }
        },
        other => {
            eprintln!("Unknown geom subcommand: {other}");
            1
        }
    }
}
