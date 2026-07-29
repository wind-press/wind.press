//! TrueType quadratic-outline compatibility.
//!
//! Static releases often contain the same contour with a different number of
//! points. A gvar table cannot encode that directly, so we convert affected
//! simple glyphs to a shared sequence of quadratic segments before computing
//! deltas. The default glyph is rewritten with that sequence as well.

use kurbo::{BezPath, PathEl, Point as BezPoint};
use linesweeper::{BinaryOp, FillRule, binary_op};
use read_fonts::tables::glyf::CurvePoint;
use write_fonts::tables::glyf::{Contour, SimpleGlyph};

use super::Point;

// This mirrors the source-unit density used by static-to-variable's line
// reconstruction: dense enough to preserve a curved TrueType outline, without
// inflating every compatible glyph into a needlessly large variable font.
const RESAMPLE_STEP: f64 = 18.0;
const MAX_POINTS_PER_CONTOUR: usize = 4096;
// The source tool retries a deliberately uneven set of corner thresholds.
// Different releases can encode the same apparent corner with subtly
// different quadratic handles, so one threshold is needlessly brittle.
const CORNER_ANGLE_SWEEP_DEGREES: [f64; 14] = [
    28.0, 24.0, 32.0, 20.0, 36.0, 16.0, 40.0, 12.0, 44.0, 10.0, 48.0, 8.0, 14.0, 26.0,
];
const DEFAULT_CORNER_ANGLE_RADIANS: f64 = std::f64::consts::PI * 28.0 / 180.0;
const INTERPOLATION_SAMPLES: [f64; 3] = [0.25, 0.5, 0.75];
// Keep this in lockstep with mblode/static-to-variable. The quality gate is
// deliberately evaluated at the same 72px scale as the reference pipeline;
// small outlines are remeasured at twice this resolution below.
const INK_RESOLUTION: usize = 72;
const INK_BLUR: usize = 2;
const INK_FREEZE_TOLERANCE: f64 = 1.5;
const QUALITY_AREA_TOLERANCE: f64 = 0.10;
const NECK_MAX_FRACTION: f64 = 0.16;
const NECK_MIN_ARC_FRACTION: f64 = 0.15;
const MAX_NECK_SAMPLES: usize = 512;
const MAX_SPLIT_CANDIDATES_PER_CONTOUR: usize = 32;
const MAX_TOPOLOGY_VARIANTS: usize = 96;
const BRIDGE_VARIANTS: usize = 6;
// Keep these values aligned with static-to-variable's dedicated
// counter-closing reconstruction. A nonzero floor lets a hole or a detached
// piece close gracefully, rather than suddenly collapsing to a single point.
const COUNTER_TAPER: f64 = 0.45;
const MIN_COUNTER_FRACTION: f64 = 5e-4;
const SYNTHETIC_HOLE_SCALE: f64 = 0.02;
pub(super) const DEFAULT_NUB_OVERLAP: f64 = 30.0;
pub(super) const DEFAULT_MIN_PROTRUDE: f64 = 70.0;

#[derive(Clone)]
pub(super) struct NormalizedSimpleGlyphs {
    pub(super) master_contours: Vec<Vec<Vec<Point>>>,
    pub(super) default_glyph: SimpleGlyph,
}

#[derive(Clone, Copy)]
pub(super) struct OpenBarOptions {
    pub(super) nub_overlap: f64,
    pub(super) min_protrude: f64,
    pub(super) anchor: OpenBarAnchor,
}

#[derive(Clone, Copy)]
pub(super) enum OpenBarAnchor {
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Quad {
    start: BezPoint,
    control: BezPoint,
    end: BezPoint,
}

#[derive(Clone, Copy, Debug)]
struct FloatPoint {
    x: f64,
    y: f64,
}

#[derive(Clone, Debug)]
struct DenseContour {
    points: Vec<FloatPoint>,
    corners: Vec<bool>,
}

#[derive(Clone, Copy)]
struct ContourFeature {
    center_x: f64,
    center_y: f64,
    area_fraction: f64,
    winding: i8,
}

#[derive(Clone, Copy)]
enum ResamplingStrategy {
    CornerAnchored,
    Uniform,
}

struct ReconstructionCandidate {
    contours: Vec<Vec<Vec<FloatPoint>>>,
    ink_defect: f64,
}

/// A scanline raster with arbitrary horizontal resolution.
///
/// mblode stores each row in a Python integer, which naturally grows beyond
/// 64 bits. The Rust port keeps the exact bitwise operations but represents a
/// row as little-endian `u64` words, so the 72px and 144px quality passes do
/// not silently lose their rightmost pixels.
#[derive(Clone)]
struct InkRaster {
    resolution: usize,
    rows: Vec<Vec<u64>>,
}

impl InkRaster {
    fn empty(resolution: usize) -> Self {
        let words = resolution.div_ceil(64);
        Self {
            resolution,
            rows: vec![vec![0; words]; resolution],
        }
    }

    fn word_count(&self) -> usize {
        self.rows.first().map_or(0, Vec::len)
    }

    fn count(&self) -> u32 {
        self.rows
            .iter()
            .flatten()
            .map(|word| word.count_ones())
            .sum()
    }

    fn set_columns(&mut self, row: usize, first: i32, last: i32) {
        let first = first.max(0) as usize;
        let last = last.min(self.resolution as i32 - 1);
        if first >= self.resolution || last < first as i32 {
            return;
        }
        let last = last as usize;
        let first_word = first / 64;
        let last_word = last / 64;
        for word_index in first_word..=last_word {
            let first_bit = if word_index == first_word {
                first % 64
            } else {
                0
            };
            let last_bit = if word_index == last_word {
                last % 64
            } else {
                63
            };
            let upper = if last_bit == 63 {
                u64::MAX
            } else {
                (1_u64 << (last_bit + 1)) - 1
            };
            let lower = if first_bit == 0 {
                0
            } else {
                (1_u64 << first_bit) - 1
            };
            self.rows[row][word_index] |= upper & !lower;
        }
    }

    fn pairwise(&self, other: &Self, operation: impl Fn(u64, u64) -> u64) -> Self {
        debug_assert_eq!(self.resolution, other.resolution);
        let mut output = Self::empty(self.resolution);
        for ((output_row, left_row), right_row) in
            output.rows.iter_mut().zip(&self.rows).zip(&other.rows)
        {
            for ((output, left), right) in output_row.iter_mut().zip(left_row).zip(right_row) {
                *output = operation(*left, *right);
            }
        }
        output
    }

    fn shifted_left(words: &[u64], last_mask: u64) -> Vec<u64> {
        words
            .iter()
            .enumerate()
            .map(|(index, word)| {
                let carry = index
                    .checked_sub(1)
                    .map_or(0, |previous| words[previous] >> 63);
                let shifted = (*word << 1) | carry;
                if index + 1 == words.len() {
                    shifted & last_mask
                } else {
                    shifted
                }
            })
            .collect()
    }

    fn shifted_right(words: &[u64]) -> Vec<u64> {
        words
            .iter()
            .enumerate()
            .map(|(index, word)| {
                let carry = words.get(index + 1).map_or(0, |next| *next << 63);
                (*word >> 1) | carry
            })
            .collect()
    }

    fn last_word_mask(&self) -> u64 {
        let bits = self.resolution % 64;
        if bits == 0 {
            u64::MAX
        } else {
            (1_u64 << bits) - 1
        }
    }
}

pub(super) fn normalize_simple_glyphs(
    masters: &[Vec<Vec<Point>>],
    master_locations: &[Vec<f64>],
    default_master: usize,
) -> Result<NormalizedSimpleGlyphs, String> {
    if default_master >= masters.len() {
        return Err(format!(
            "default master index {default_master} is outside the outline set"
        ));
    }
    if masters[default_master].is_empty() {
        return Err("empty simple glyph cannot be normalized".into());
    }
    let span_pairs = master_span_pairs(master_locations, masters.len());

    let mut errors = Vec::new();
    let resampled = 'angle_sweep: {
        for angle_degrees in CORNER_ANGLE_SWEEP_DEGREES {
            let angle_radians = std::f64::consts::PI * angle_degrees / 180.0;
            let dense_masters: Vec<Vec<DenseContour>> = masters
                .iter()
                .map(|contours| {
                    contours
                        .iter()
                        .map(|contour| dense_contour(contour, angle_radians))
                        .collect()
                })
                .collect::<Result<_, _>>()?;
            let mut candidates = Vec::new();
            for topology in contour_topology_variants(&dense_masters, master_locations) {
                match reconstruct_topology(&topology, default_master, &span_pairs) {
                    Ok(candidate) => candidates.push(candidate),
                    Err(error) => errors.push(error),
                }
            }
            if let Some(candidate) = candidates
                .into_iter()
                .min_by(|left, right| left.ink_defect.total_cmp(&right.ink_defect))
            {
                break 'angle_sweep candidate.contours;
            }
        }
        let detail = errors
            .into_iter()
            .next()
            .unwrap_or_else(|| "no topology candidates were available".into());
        return Err(format!("no safe reconstruction candidate: {detail}"));
    };

    let master_contours: Vec<Vec<Vec<Point>>> = resampled
        .iter()
        .map(|contours| {
            contours
                .iter()
                .map(|contour| {
                    contour
                        .iter()
                        .copied()
                        .map(float_to_point)
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<_, _>>()?;
    let default_glyph = make_simple_glyph(&master_contours[default_master]);

    Ok(NormalizedSimpleGlyphs {
        master_contours,
        default_glyph,
    })
}

/// Native port of mblode's explicit `open_bar` strategy.
///
/// This intentionally builds a new design rather than forcing the donor `$` or
/// `¢` through a generic contour repair: a reconstructed bare-letter body is
/// joined to two short, overlapping bar nubs, leaving the middle open. The
/// caller opts into this per glyph, exactly as mblode requires in its config.
pub(super) fn reconstruct_open_bar(
    glyph_masters: &[Vec<Vec<Point>>],
    letter_masters: &[Vec<Vec<Point>>],
    master_locations: &[Vec<f64>],
    default_master: usize,
    options: OpenBarOptions,
) -> Result<NormalizedSimpleGlyphs, String> {
    if glyph_masters.len() != letter_masters.len() || glyph_masters.is_empty() {
        return Err("open_bar requires matching glyph and bare-letter masters".into());
    }
    let bar_geometry: Vec<_> = glyph_masters
        .iter()
        .map(|contours| measure_bar_geometry(contours))
        .collect::<Result<_, _>>()?;
    let mut body_masters = Vec::with_capacity(letter_masters.len());
    for contours in letter_masters {
        let body = largest_positive_dense(contours).ok_or_else(|| {
            "open_bar needs one positive bare-letter contour per master".to_string()
        })?;
        let points = body
            .points
            .iter()
            .copied()
            .map(float_to_point)
            .collect::<Result<Vec<_>, _>>()?;
        body_masters.push(vec![points]);
    }
    let body =
        normalize_simple_glyphs(&body_masters, master_locations, default_master).or_else(|_| {
            // The full body reconstruction is preferred. This is mblode's
            // configured-anchor fallback for unusually incompatible body donors.
            reconstruct_open_bar_body_with_anchor(&body_masters, default_master, options.anchor)
        })?;
    if body
        .master_contours
        .iter()
        .any(|contours| contours.len() != 1 || contours[0].len() < 3)
    {
        return Err("open_bar could not reconstruct a single bare-letter body".into());
    }

    let mut master_contours = Vec::with_capacity(body.master_contours.len());
    let mut float_masters = Vec::with_capacity(body.master_contours.len());
    for (master_index, body_contours) in body.master_contours.iter().enumerate() {
        let body_dense = DenseContour {
            points: body_contours[0]
                .iter()
                .map(|point| FloatPoint {
                    x: f64::from(point.x),
                    y: f64::from(point.y),
                })
                .collect(),
            corners: body_contours[0]
                .iter()
                .map(|point| point.on_curve)
                .collect(),
        };
        let nubs = bar_nubs(&body_dense, bar_geometry[master_index], options)?;
        let mut output = vec![body_contours[0].clone()];
        output.extend(
            nubs.iter()
                .map(|nub| {
                    nub.points
                        .iter()
                        .copied()
                        .map(float_to_point)
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        float_masters.push(
            output
                .iter()
                .map(|contour| {
                    contour
                        .iter()
                        .map(|point| FloatPoint {
                            x: f64::from(point.x),
                            y: f64::from(point.y),
                        })
                        .collect()
                })
                .collect(),
        );
        master_contours.push(output);
    }
    let spans = master_span_pairs(master_locations, master_contours.len());
    reject_interpolation_defects(&float_masters, &spans)?;
    reject_interpolation_area_and_perimeter_defects(&float_masters, &spans)?;
    let default_glyph = make_simple_glyph(&master_contours[default_master]);
    Ok(NormalizedSimpleGlyphs {
        master_contours,
        default_glyph,
    })
}

fn reconstruct_open_bar_body_with_anchor(
    body_masters: &[Vec<Vec<Point>>],
    default_master: usize,
    anchor: OpenBarAnchor,
) -> Result<NormalizedSimpleGlyphs, String> {
    let mut dense = Vec::with_capacity(body_masters.len());
    for contours in body_masters {
        let mut contour = dense_contour(&contours[0], DEFAULT_CORNER_ANGLE_RADIANS)?;
        let start = match anchor {
            OpenBarAnchor::Left => contour
                .points
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    left.x.total_cmp(&right.x).then(left.y.total_cmp(&right.y))
                })
                .map(|(index, _)| index),
            OpenBarAnchor::Right => contour
                .points
                .iter()
                .enumerate()
                .max_by(|(_, left), (_, right)| {
                    left.x.total_cmp(&right.x).then(left.y.total_cmp(&right.y))
                })
                .map(|(index, _)| index),
        }
        .ok_or_else(|| "open_bar fallback body has no points".to_string())?;
        rotate_contour(&mut contour, start);
        dense.push(contour);
    }
    let resampled = resample_contour_set(&dense, default_master, ResamplingStrategy::Uniform)?;
    let master_contours: Vec<Vec<Vec<Point>>> = resampled
        .into_iter()
        .map(|contour| {
            contour
                .into_iter()
                .map(float_to_point)
                .collect::<Result<Vec<_>, _>>()
                .map(|contour| vec![contour])
        })
        .collect::<Result<_, _>>()?;
    let default_glyph = make_simple_glyph(&master_contours[default_master]);
    Ok(NormalizedSimpleGlyphs {
        master_contours,
        default_glyph,
    })
}

#[derive(Clone, Copy)]
struct BarGeometry {
    top_x_min: f64,
    top_x_max: f64,
    donor_y_min: f64,
    donor_y_max: f64,
}

fn largest_positive_dense(contours: &[Vec<Point>]) -> Option<DenseContour> {
    let mut contour = contours
        .iter()
        .filter_map(|contour| dense_contour(contour, DEFAULT_CORNER_ANGLE_RADIANS).ok())
        .max_by(|left, right| {
            dense_signed_area(left)
                .abs()
                .total_cmp(&dense_signed_area(right).abs())
        })?;
    // TrueType donors may reverse every contour between releases. The largest
    // ring is the body in this strategy, so normalize it to positive winding
    // before measuring the bar or joining the nubs.
    if dense_signed_area(&contour) < 0.0 {
        reverse_contour(&mut contour);
    }
    Some(contour)
}

fn measure_bar_geometry(contours: &[Vec<Point>]) -> Result<BarGeometry, String> {
    let bar = largest_positive_dense(contours)
        .ok_or_else(|| "open_bar could not find the donor's positive bar contour".to_string())?;
    let (x_min, y_min, x_max, y_max) = bounds(&bar.points);
    let top_band: Vec<_> = bar
        .points
        .iter()
        .copied()
        .filter(|point| point.y > y_max - 40.0)
        .collect();
    if top_band.is_empty() {
        return Err("open_bar could not measure the donor bar width".into());
    }
    Ok(BarGeometry {
        top_x_min: top_band.iter().map(|point| point.x).fold(x_max, f64::min),
        top_x_max: top_band.iter().map(|point| point.x).fold(x_min, f64::max),
        donor_y_min: y_min,
        donor_y_max: y_max,
    })
}

fn bar_nubs(
    body: &DenseContour,
    geometry: BarGeometry,
    options: OpenBarOptions,
) -> Result<[DenseContour; 2], String> {
    let (x_min, y_min, x_max, y_max) = bounds(&body.points);
    let height = y_max - y_min;
    if height <= 0.0 {
        return Err("open_bar bare-letter body has no height".into());
    }
    let width = geometry.top_x_max - geometry.top_x_min;
    if width <= 0.0 {
        return Err("open_bar donor bar has no width".into());
    }
    let protrude = (geometry.donor_y_max - y_max)
        .max(y_min - geometry.donor_y_min)
        .max(options.min_protrude);
    let top_spine_y = y_max - height * 0.08;
    let bottom_spine_y = y_min + height * 0.08;
    let top_spine_x = ink_span_at_y(&body.points, top_spine_y)
        .map(|(left, right)| (left + right) / 2.0)
        .unwrap_or((x_min + x_max) / 2.0);
    let bottom_spine_x = ink_span_at_y(&body.points, bottom_spine_y)
        .map(|(left, right)| (left + right) / 2.0)
        .unwrap_or((x_min + x_max) / 2.0);
    let slant = (top_spine_x - bottom_spine_x) / (top_spine_y - bottom_spine_y);
    let bar_x = |y: f64| bottom_spine_x + slant * (y - bottom_spine_y);
    let top_span = ink_span_at_x(&body.points, top_spine_x);
    let bottom_span = ink_span_at_x(&body.points, bottom_spine_x);
    let top_join_y = top_span.map_or(y_max, |(_, high)| high) - options.nub_overlap;
    let bottom_join_y = bottom_span.map_or(y_min, |(low, _)| low) + options.nub_overlap;
    let top_y = y_max + protrude;
    let bottom_y = y_min - protrude;
    let make_nub = |points: Vec<FloatPoint>| DenseContour {
        corners: vec![true; points.len()],
        points,
    };
    let mut top = vec![
        FloatPoint {
            x: bar_x(top_join_y) - width / 2.0,
            y: top_join_y,
        },
        FloatPoint {
            x: bar_x(top_join_y) + width / 2.0,
            y: top_join_y,
        },
        FloatPoint {
            x: bar_x(top_y) + width / 2.0,
            y: top_y,
        },
        FloatPoint {
            x: bar_x(top_y) - width / 2.0,
            y: top_y,
        },
    ];
    let mut bottom = vec![
        FloatPoint {
            x: bar_x(bottom_y) - width / 2.0,
            y: bottom_y,
        },
        FloatPoint {
            x: bar_x(bottom_y) + width / 2.0,
            y: bottom_y,
        },
        FloatPoint {
            x: bar_x(bottom_join_y) + width / 2.0,
            y: bottom_join_y,
        },
        FloatPoint {
            x: bar_x(bottom_join_y) - width / 2.0,
            y: bottom_join_y,
        },
    ];
    if signed_ring_area(&top) < 0.0 {
        top.reverse();
    }
    if signed_ring_area(&bottom) < 0.0 {
        bottom.reverse();
    }
    Ok([make_nub(top), make_nub(bottom)])
}

fn ink_span_at_x(points: &[FloatPoint], x: f64) -> Option<(f64, f64)> {
    let mut crossings = Vec::new();
    for (index, start) in points.iter().enumerate() {
        let end = points[(index + 1) % points.len()];
        if (start.x <= x && x < end.x) || (end.x <= x && x < start.x) {
            crossings.push(start.y + (x - start.x) / (end.x - start.x) * (end.y - start.y));
        }
    }
    crossings
        .iter()
        .copied()
        .min_by(f64::total_cmp)
        .zip(crossings.iter().copied().max_by(f64::total_cmp))
}

fn ink_span_at_y(points: &[FloatPoint], y: f64) -> Option<(f64, f64)> {
    let mut crossings = Vec::new();
    for (index, start) in points.iter().enumerate() {
        let end = points[(index + 1) % points.len()];
        if (start.y <= y && y < end.y) || (end.y <= y && y < start.y) {
            crossings.push(start.x + (y - start.y) / (end.y - start.y) * (end.x - start.x));
        }
    }
    crossings
        .iter()
        .copied()
        .min_by(f64::total_cmp)
        .zip(crossings.iter().copied().max_by(f64::total_cmp))
}

fn reconstruct_topology(
    dense_masters: &[Vec<DenseContour>],
    default_master: usize,
    span_pairs: &[(usize, usize)],
) -> Result<ReconstructionCandidate, String> {
    let (reference_index, reference) = dense_masters
        .iter()
        .enumerate()
        .max_by_key(|(_, contours)| contours.len())
        .ok_or_else(|| "no master outlines were supplied".to_string())?;
    if reference.is_empty() {
        return Err("empty simple glyph cannot be normalized".into());
    }

    let mut ordered = Vec::with_capacity(dense_masters.len());
    for (master_index, contours) in dense_masters.iter().enumerate() {
        if master_index == reference_index {
            ordered.push(contours.clone());
            continue;
        }
        ordered.push(match_contours_to_reference(reference, contours)?);
    }

    // The reconstruction is allowed to rewrite topology, but it must still
    // render close to every source master. mblode measures that against the
    // donor outlines rather than trusting point correspondence alone.
    let donors: Vec<Vec<Vec<FloatPoint>>> = dense_masters
        .iter()
        .map(|contours| {
            contours
                .iter()
                .map(|contour| contour.points.clone())
                .collect()
        })
        .collect();

    // Preserve crisp corners when they render cleanly, but also evaluate a
    // rotation-aligned uniform map. A low-resolution nonzero-winding ink test
    // checks each in-between master span and chooses the one whose visible
    // shape loses or gains the least ink—not merely the one with valid point
    // counts. This mirrors the reference tool's quality tournament entirely
    // inside Rust/WASM.
    let corner = reconstruct_candidate(
        &ordered,
        reference.len(),
        default_master,
        ResamplingStrategy::CornerAnchored,
        span_pairs,
        &donors,
    );
    let uniform = reconstruct_candidate(
        &ordered,
        reference.len(),
        default_master,
        ResamplingStrategy::Uniform,
        span_pairs,
        &donors,
    );
    match (corner, uniform) {
        (Ok(corner), Ok(uniform)) if uniform.ink_defect < corner.ink_defect - 1e-9 => Ok(uniform),
        (Ok(corner), _) => Ok(corner),
        (Err(_), Ok(uniform)) => Ok(uniform),
        (Err(corner_error), Err(uniform_error)) => Err(format!(
            "corner-anchored reconstruction failed ({corner_error}); uniform arc-length fallback failed ({uniform_error})"
        )),
    }
}

fn dense_contour(contour: &[Point], corner_angle: f64) -> Result<DenseContour, String> {
    let curves = contour_to_quadratics(contour)?;
    if curves.is_empty() {
        return Err("a contour has no drawable segments".into());
    }
    if curves.len() >= MAX_POINTS_PER_CONTOUR {
        return Err(format!(
            "a contour has more than {MAX_POINTS_PER_CONTOUR} drawable segments"
        ));
    }
    let total_length: f64 = curves.iter().copied().map(curve_length_hint).sum();
    let step = RESAMPLE_STEP.max(total_length / (MAX_POINTS_PER_CONTOUR - curves.len()) as f64);

    let mut points = Vec::new();
    let mut corners = Vec::new();
    for (curve_index, curve) in curves.iter().copied().enumerate() {
        if points.is_empty() {
            points.push(from_bez(curve.start));
            corners.push(is_corner(&curves, curve_index, corner_angle));
        }
        // A tiny closed contour can consist of only one or two quadratic
        // segments. Sampling each segment once would retain only its start
        // point after the duplicate closing point is removed, destroying a
        // valid small shape. Preserve at least three ring samples instead.
        let minimum_steps = match curves.len() {
            1 => 3,
            2 => 2,
            _ => 1,
        };
        let steps = (curve_length_hint(curve) / step)
            .ceil()
            .max(minimum_steps as f64) as usize;
        if points.len().saturating_add(steps) > MAX_POINTS_PER_CONTOUR {
            return Err(format!(
                "a contour would require more than {MAX_POINTS_PER_CONTOUR} reconstruction points"
            ));
        }
        for sample_index in 1..=steps {
            // A TrueType outline is implicitly closed. Do not store the final
            // duplicate of the start point as an extra gvar point.
            if curve_index + 1 == curves.len() && sample_index == steps {
                continue;
            }
            points.push(evaluate_quad(curve, sample_index as f64 / steps as f64));
            corners.push(if sample_index == steps {
                is_corner(&curves, (curve_index + 1) % curves.len(), corner_angle)
            } else {
                false
            });
        }
    }
    if points.len() < 3 {
        return Err("a contour has fewer than three drawable points".into());
    }
    Ok(DenseContour { points, corners })
}

fn stable_contour_assignment(
    reference: &[DenseContour],
    target: &[DenseContour],
) -> Result<Vec<Option<usize>>, String> {
    if target.len() > reference.len() {
        return Err("contour count exceeds every available master".into());
    }
    let reference_features = contour_features(reference);
    let target_features = contour_features(target);
    let costs: Vec<Vec<_>> = reference_features
        .iter()
        .map(|reference| {
            target_features
                .iter()
                .map(|target| contour_match_cost(*reference, *target))
                .chain((target_features.len()..reference_features.len()).map(|_| {
                    // The smallest feature is the safest one to collapse when
                    // a static release genuinely omits a contour.
                    (reference.area_fraction * 1_000_000.0).round() as i64
                }))
                .collect()
        })
        .collect();
    Ok(super::hungarian_assignment(&costs)?
        .into_iter()
        .map(|index| (index < target.len()).then_some(index))
        .collect())
}

fn contour_topology_variants(
    masters: &[Vec<DenseContour>],
    master_locations: &[Vec<f64>],
) -> Vec<Vec<Vec<DenseContour>>> {
    if masters.is_empty()
        || masters
            .iter()
            .map(Vec::len)
            .all(|count| count == masters[0].len())
    {
        return vec![masters.to_vec()];
    }

    let mut variants = Vec::new();
    // A counter closing is a very different case from an arbitrary topology
    // mismatch. Build its tiny, winding-aware transition first; generic
    // split/merge/union repairs remain the fallbacks for every other glyph.
    if let Some(counter_closing) = counter_closing_topology(masters, master_locations) {
        variants.push(counter_closing);
    }
    variants.push(masters.to_vec());
    if let Some(split) = split_topology_to_max(masters) {
        variants.push(split);
    }
    for pick in 0..=BRIDGE_VARIANTS {
        if let Ok(Some(merged)) = merge_topology_to_min(masters, pick) {
            variants.push(merged);
        }
    }
    variants.sort_by_key(|variant| topology_fingerprint(variant));
    variants.dedup_by(|left, right| topology_fingerprint(left) == topology_fingerprint(right));
    variants
}

/// Port of static-to-variable's `_counter_closing` repair.
///
/// A number of otherwise compatible families close a counter or merge a small
/// detached piece in a heavy master. Treating that master as if the contour had
/// simply vanished creates a sharp collapse in the middle of the axis. Here the
/// most-open master supplies winding-aware contour slots; missing slots are
/// reconstructed as tiny, shape-preserving rings and then passed through the
/// normal quality gates.
fn counter_closing_topology(
    masters: &[Vec<DenseContour>],
    master_locations: &[Vec<f64>],
) -> Option<Vec<Vec<DenseContour>>> {
    if masters.is_empty()
        || masters.iter().any(Vec::is_empty)
        || masters
            .iter()
            .map(Vec::len)
            .all(|count| count == masters[0].len())
    {
        return None;
    }

    #[derive(Clone, Copy)]
    struct Entry<'a> {
        contour: &'a DenseContour,
        center: FloatPoint,
        sign: i8,
    }

    let mut entries = Vec::with_capacity(masters.len());
    for contours in masters {
        let dominant_sign = contours
            .iter()
            .max_by(|left, right| {
                dense_signed_area(left)
                    .abs()
                    .total_cmp(&dense_signed_area(right).abs())
            })
            .map(contour_winding)?;
        entries.push(
            contours
                .iter()
                .map(|contour| Entry {
                    contour,
                    center: contour_center(contour),
                    // Normalize each master to its dominant winding. Some
                    // releases reverse every contour between masters, but a
                    // counter must never be matched to a positive body slot.
                    sign: contour_winding(contour) * dominant_sign,
                })
                .collect::<Vec<_>>(),
        );
    }

    let slot_master = masters
        .iter()
        .enumerate()
        .max_by_key(|(_, contours)| contours.len())?
        .0;
    let slots = &entries[slot_master];
    let slot_count = slots.len();
    let mut families = vec![vec![None; masters.len()]; slot_count];

    for (master_index, master_entries) in entries.iter().enumerate() {
        let mut used = vec![false; slot_count];
        for entry in master_entries {
            let slot = (0..slot_count)
                .filter(|slot| !used[*slot] && slots[*slot].sign == entry.sign)
                .min_by(|left, right| {
                    distance_float(entry.center, slots[*left].center)
                        .total_cmp(&distance_float(entry.center, slots[*right].center))
                });
            if let Some(slot) = slot {
                used[slot] = true;
                families[slot][master_index] = Some(entry.contour.clone());
            }
        }
    }

    if families
        .iter()
        .any(|family| family.iter().all(Option::is_none))
    {
        return None;
    }

    let body_reference_area = families
        .iter()
        .flatten()
        .filter_map(Option::as_ref)
        .map(|contour| dense_signed_area(contour).abs())
        .max_by(f64::total_cmp)
        .unwrap_or(1.0);
    let body_slot = slots
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| {
            dense_signed_area(left.contour)
                .abs()
                .total_cmp(&dense_signed_area(right.contour).abs())
        })?
        .0;
    let body_sign = slots[body_slot].sign;
    let (master_order, positions) = counter_closing_master_order(master_locations, masters.len());

    for slot in 0..slot_count {
        let present: Vec<_> = master_order
            .iter()
            .copied()
            .filter(|master| families[slot][*master].is_some())
            .collect();
        let missing: Vec<_> = master_order
            .iter()
            .copied()
            .filter(|master| families[slot][*master].is_none())
            .collect();
        if missing.is_empty() {
            continue;
        }
        let near_master = present.iter().copied().min_by(|left, right| {
            let left_distance = missing
                .iter()
                .map(|missing| (positions[*left] - positions[*missing]).abs())
                .fold(f64::INFINITY, f64::min);
            let right_distance = missing
                .iter()
                .map(|missing| (positions[*right] - positions[*missing]).abs())
                .fold(f64::INFINITY, f64::min);
            left_distance.total_cmp(&right_distance)
        })?;
        let near = families[slot][near_master].as_ref()?.clone();
        let near_center = contour_center(&near);

        if slots[slot].sign != body_sign {
            // A counter that does not exist in a donor needs to be nearly
            // invisible at that endpoint. Map it through the body bounds so
            // it stays inside the local bowl as it starts to emerge.
            for master in missing {
                let center = match (
                    families[body_slot][near_master].as_ref(),
                    families[body_slot][master].as_ref(),
                ) {
                    (Some(from_body), Some(to_body)) => {
                        map_bbox_point(near_center, from_body, to_body)
                    }
                    _ => near_center,
                };
                families[slot][master] = Some(scaled_contour(&near, SYNTHETIC_HOLE_SCALE, center));
            }
            continue;
        }

        // A positive bar/stub that merges into the body closes more slowly:
        // extrapolate the last two real areas, retain half the last area at a
        // minimum, then taper any remaining missing masters geometrically.
        let heaviest = *present.last()?;
        let heaviest_contour = families[slot][heaviest].as_ref()?.clone();
        let heaviest_area = dense_signed_area(&heaviest_contour).abs().max(1.0);
        let pairs: Vec<_> = present
            .iter()
            .map(|master| {
                (
                    positions[*master],
                    dense_signed_area(families[slot][*master].as_ref().expect("present contour"))
                        .abs(),
                )
            })
            .collect();
        let target_area = counter_area_target(&pairs).unwrap_or(heaviest_area * 0.5);
        let first_scale = (target_area.max(1.0) / heaviest_area).sqrt().min(0.95);
        let floor_scale =
            (MIN_COUNTER_FRACTION * body_reference_area).max(1.0).sqrt() / heaviest_area.sqrt();
        let heaviest_center = contour_center(&heaviest_contour);
        for (missing_index, master) in missing.into_iter().enumerate() {
            let scale = (first_scale * COUNTER_TAPER.powi(missing_index as i32)).max(floor_scale);
            families[slot][master] =
                Some(scaled_contour(&heaviest_contour, scale, heaviest_center));
        }
    }

    Some(
        (0..masters.len())
            .map(|master| {
                families
                    .iter()
                    .map(|family| family[master].clone().expect("all slots synthesized"))
                    .collect()
            })
            .collect(),
    )
}

fn contour_winding(contour: &DenseContour) -> i8 {
    if dense_signed_area(contour) >= 0.0 {
        1
    } else {
        -1
    }
}

fn counter_closing_master_order(
    master_locations: &[Vec<f64>],
    master_count: usize,
) -> (Vec<usize>, Vec<f64>) {
    let usable_locations = master_locations.len() == master_count
        && master_locations
            .first()
            .is_some_and(|location| !location.is_empty())
        && master_locations.iter().all(|location| {
            location.len() == master_locations[0].len()
                && location.iter().all(|value| value.is_finite())
        });
    let positions: Vec<f64> = if usable_locations {
        let axis = (0..master_locations[0].len())
            .max_by(|left, right| {
                let left_range = master_locations
                    .iter()
                    .map(|location| location[*left])
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), value| {
                        (min.min(value), max.max(value))
                    });
                let right_range = master_locations
                    .iter()
                    .map(|location| location[*right])
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), value| {
                        (min.min(value), max.max(value))
                    });
                (left_range.1 - left_range.0).total_cmp(&(right_range.1 - right_range.0))
            })
            .unwrap_or(0);
        master_locations
            .iter()
            .map(|location| location[axis])
            .collect()
    } else {
        (0..master_count).map(|index| index as f64).collect()
    };
    let mut order: Vec<_> = (0..master_count).collect();
    order.sort_by(|left, right| {
        positions[*left]
            .total_cmp(&positions[*right])
            .then(left.cmp(right))
    });
    (order, positions)
}

fn counter_area_target(present_pairs: &[(f64, f64)]) -> Option<f64> {
    let (before_position, before_area) = *present_pairs.iter().rev().nth(1)?;
    let (last_position, last_area) = *present_pairs.last()?;
    let slope = if last_position == before_position {
        0.0
    } else {
        (last_area - before_area) / (last_position - before_position)
    };
    Some((last_area + slope * (last_position - before_position)).max(last_area * 0.5))
}

fn scaled_contour(contour: &DenseContour, scale: f64, center: FloatPoint) -> DenseContour {
    let source_center = contour_center(contour);
    DenseContour {
        points: contour
            .points
            .iter()
            .map(|point| FloatPoint {
                x: center.x + (point.x - source_center.x) * scale,
                y: center.y + (point.y - source_center.y) * scale,
            })
            .collect(),
        corners: contour.corners.clone(),
    }
}

fn map_bbox_point(point: FloatPoint, from: &DenseContour, to: &DenseContour) -> FloatPoint {
    let from_bounds = bounds(&from.points);
    let to_bounds = bounds(&to.points);
    FloatPoint {
        x: to_bounds.0
            + (point.x - from_bounds.0) / (from_bounds.2 - from_bounds.0).max(1.0)
                * (to_bounds.2 - to_bounds.0).max(1.0),
        y: to_bounds.1
            + (point.y - from_bounds.1) / (from_bounds.3 - from_bounds.1).max(1.0)
                * (to_bounds.3 - to_bounds.1).max(1.0),
    }
}

fn topology_fingerprint(masters: &[Vec<DenseContour>]) -> Vec<i64> {
    masters
        .iter()
        .flat_map(|contours| {
            std::iter::once(contours.len() as i64).chain(contours.iter().flat_map(|contour| {
                let sample = |fraction: usize| contour.points[contour.points.len() * fraction / 3];
                [
                    contour.points.len() as i64,
                    sample(0).x.round() as i64,
                    sample(0).y.round() as i64,
                    sample(1).x.round() as i64,
                    sample(1).y.round() as i64,
                    sample(2).x.round() as i64,
                    sample(2).y.round() as i64,
                ]
            }))
        })
        .collect()
}

fn merge_topology_to_min(
    masters: &[Vec<DenseContour>],
    bridge_pick: usize,
) -> Result<Option<Vec<Vec<DenseContour>>>, String> {
    let target_count = masters.iter().map(Vec::len).min().unwrap_or_default();
    let mut merged = masters.to_vec();
    for contours in &mut merged {
        while contours.len() > target_count {
            if let Some((first, second, union)) = first_unionable_pair(contours)? {
                contours.remove(second);
                contours.remove(first);
                contours.push(union);
            } else if let Some(bridged) = bridge_smallest_contour(
                contours,
                if contours.len() == target_count + 1 {
                    bridge_pick
                } else {
                    0
                },
            ) {
                *contours = bridged;
            } else {
                return Ok(None);
            }
        }
    }
    Ok(Some(merged))
}

fn split_topology_to_max(masters: &[Vec<DenseContour>]) -> Option<Vec<Vec<DenseContour>>> {
    let target_count = masters.iter().map(Vec::len).max()?;
    if masters
        .iter()
        .all(|contours| contours.len() == target_count)
    {
        return None;
    }
    let template = masters
        .iter()
        .find(|contours| contours.len() == target_count)?;
    let mut split = Vec::with_capacity(masters.len());
    for contours in masters {
        if contours.len() == target_count {
            split.push(contours.clone());
        } else {
            split.push(split_contours_to_match(contours, template)?);
        }
    }
    Some(split)
}

fn bridge_smallest_contour(
    contours: &[DenseContour],
    bridge_pick: usize,
) -> Option<Vec<DenseContour>> {
    if contours.len() < 2 {
        return None;
    }
    let smallest = contours
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            dense_signed_area(left)
                .abs()
                .partial_cmp(&dense_signed_area(right).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })?
        .0;
    let small_center = contour_center(&contours[smallest]);
    let target = contours
        .iter()
        .enumerate()
        .filter(|(index, contour)| {
            *index != smallest
                && !polygon_strictly_contains(&contour.points, contours[smallest].points[0])
                && !polygon_strictly_contains(&contours[smallest].points, contour.points[0])
        })
        .min_by(|(_, left), (_, right)| {
            distance_float(contour_center(left), small_center)
                .partial_cmp(&distance_float(contour_center(right), small_center))
                .unwrap_or(std::cmp::Ordering::Equal)
        })?
        .0;
    let bridged = bridge_contours(&contours[target], &contours[smallest], bridge_pick)?;
    let mut output: Vec<_> = contours
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != smallest && *index != target)
        .map(|(_, contour)| contour.clone())
        .collect();
    output.push(bridged);
    Some(output)
}

fn bridge_contours(
    first: &DenseContour,
    second: &DenseContour,
    bridge_pick: usize,
) -> Option<DenseContour> {
    let bucket = if bridge_pick == 0 {
        0..second.points.len()
    } else {
        if bridge_pick > BRIDGE_VARIANTS {
            return None;
        }
        let width = second.points.len().div_ceil(BRIDGE_VARIANTS);
        let start = (bridge_pick - 1) * width;
        start..(start + width).min(second.points.len())
    };
    if bucket.is_empty() {
        return None;
    }
    let (first_index, second_index, _) = first
        .points
        .iter()
        .enumerate()
        .flat_map(|(first_index, first_point)| {
            let bucket = bucket.clone();
            second
                .points
                .iter()
                .enumerate()
                .filter(move |(second_index, _)| bucket.contains(second_index))
                .map(move |(second_index, second_point)| {
                    (
                        first_index,
                        second_index,
                        distance_float(*first_point, *second_point),
                    )
                })
        })
        .min_by(|left, right| {
            left.2
                .partial_cmp(&right.2)
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
    let mut points = Vec::with_capacity(first.points.len() + second.points.len() + 2);
    points.extend_from_slice(&first.points[..=first_index]);
    points.extend_from_slice(&second.points[second_index..]);
    points.extend_from_slice(&second.points[..=second_index]);
    points.extend_from_slice(&first.points[first_index..]);
    dense_from_polyline(points)
}

fn split_contours_to_match(
    contours: &[DenseContour],
    target: &[DenseContour],
) -> Option<Vec<DenseContour>> {
    if contours.len() > target.len() {
        return None;
    }
    if contours.len() == target.len() {
        return Some(contours.to_vec());
    }

    let target_signature = area_signature(target);
    let mut frontier = vec![(contours.to_vec(), 0.0_f64)];
    for _ in contours.len()..target.len() {
        let mut next = Vec::new();
        for (candidate, cost) in frontier {
            for (split, split_cost) in split_once_candidates(&candidate) {
                next.push((split, cost + split_cost));
            }
        }
        next.sort_by(|left, right| left.1.total_cmp(&right.1));
        next.truncate(MAX_TOPOLOGY_VARIANTS);
        if next.is_empty() {
            return None;
        }
        frontier = next;
    }

    frontier
        .into_iter()
        .filter_map(|(candidate, cut_cost)| {
            signature_distance(&area_signature(&candidate), &target_signature)
                .map(|signature_cost| (signature_cost, cut_cost, candidate))
        })
        .min_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.total_cmp(&right.1))
        })
        .map(|(_, _, candidate)| candidate)
}

fn split_once_candidates(contours: &[DenseContour]) -> Vec<(Vec<DenseContour>, f64)> {
    let mut output = Vec::new();
    for (contour_index, contour) in contours.iter().enumerate() {
        let (x_min, y_min, x_max, y_max) = bounds(&contour.points);
        let diagonal = (x_max - x_min).hypot(y_max - y_min).max(1e-9);
        for (first, second) in neck_candidates(&contour.points)
            .into_iter()
            .take(MAX_SPLIT_CANDIDATES_PER_CONTOUR)
        {
            let (Some(first_piece), Some(second_piece)) = (
                dense_from_polyline(ring_slice(&contour.points, first, second)),
                dense_from_polyline(ring_slice(&contour.points, second, first)),
            ) else {
                continue;
            };
            let mut candidate = contours.to_vec();
            candidate.remove(contour_index);
            candidate.insert(contour_index, first_piece);
            candidate.insert(contour_index + 1, second_piece);
            output.push((
                candidate,
                distance_float(contour.points[first], contour.points[second]) / diagonal,
            ));
        }
    }
    output
}

fn neck_candidates(points: &[FloatPoint]) -> Vec<(usize, usize)> {
    if points.len() < 8 {
        return Vec::new();
    }
    let minimum_arc = (points.len() as f64 * NECK_MIN_ARC_FRACTION)
        .ceil()
        .max(3.0) as usize;
    let (x_min, y_min, x_max, y_max) = bounds(points);
    let limit = (x_max - x_min).hypot(y_max - y_min) * NECK_MAX_FRACTION;
    let stride = points.len().div_ceil(MAX_NECK_SAMPLES).max(1);
    let mut candidates = Vec::new();
    for first in (0..points.len()).step_by(stride) {
        for second in ((first + minimum_arc)..points.len()).step_by(stride) {
            if points.len() - second + first < minimum_arc {
                continue;
            }
            let distance = distance_float(points[first], points[second]);
            if distance <= limit {
                candidates.push((distance, first, second));
            }
        }
    }
    candidates.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut result = Vec::new();
    let nearby = (points.len() as f64 * 0.04).ceil() as usize + 1;
    for (_, first, second) in candidates {
        if result.iter().any(|(kept_first, kept_second)| {
            cyclic_distance(first, *kept_first, points.len()) <= nearby
                && cyclic_distance(second, *kept_second, points.len()) <= nearby
        }) {
            continue;
        }
        result.push((first, second));
    }
    result
}

fn area_signature(contours: &[DenseContour]) -> Vec<(i8, f64)> {
    let areas: Vec<_> = contours.iter().map(dense_signed_area).collect();
    let total = areas.iter().map(|area| area.abs()).sum::<f64>().max(1e-9);
    let mut signature: Vec<_> = areas
        .into_iter()
        .map(|area| ((if area >= 0.0 { 1 } else { -1 }), area.abs() / total))
        .collect();
    signature.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.total_cmp(&right.1))
    });
    signature
}

fn signature_distance(left: &[(i8, f64)], right: &[(i8, f64)]) -> Option<f64> {
    if left.len() != right.len() {
        return None;
    }
    [1_i8, -1_i8].into_iter().find_map(|flip| {
        let mut flipped: Vec<_> = left
            .iter()
            .map(|(sign, area)| (sign * flip, *area))
            .collect();
        flipped.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.total_cmp(&right.1))
        });
        (flipped
            .iter()
            .map(|(sign, _)| sign)
            .eq(right.iter().map(|(sign, _)| sign)))
        .then(|| {
            flipped
                .iter()
                .zip(right)
                .map(|((_, left), (_, right))| (left - right).abs())
                .sum()
        })
    })
}

fn dense_from_polyline(points: Vec<FloatPoint>) -> Option<DenseContour> {
    if points.len() < 3 {
        return None;
    }
    Some(DenseContour {
        corners: polyline_corners(&points),
        points,
    })
}

fn first_unionable_pair(
    contours: &[DenseContour],
) -> Result<Option<(usize, usize, DenseContour)>, String> {
    for first in 0..contours.len() {
        for second in first + 1..contours.len() {
            if !boxes_overlap(
                bounds(&contours[first].points),
                bounds(&contours[second].points),
            ) || polygon_strictly_contains(&contours[first].points, contours[second].points[0])
                || polygon_strictly_contains(&contours[second].points, contours[first].points[0])
            {
                continue;
            }
            let merged = union_contours(&contours[first], &contours[second])?;
            if merged.len() == 1 {
                return Ok(Some((first, second, merged.into_iter().next().unwrap())));
            }
        }
    }
    Ok(None)
}

fn union_contours(
    first: &DenseContour,
    second: &DenseContour,
) -> Result<Vec<DenseContour>, String> {
    let first = dense_to_bez_path(first);
    let second = dense_to_bez_path(second);
    let union = binary_op(&first, &second, FillRule::EvenOdd, BinaryOp::Union)
        .map_err(|error| format!("native contour union failed: {error}"))?;
    union
        .contours()
        .map(|contour| dense_from_bez_path(&contour.path))
        .collect()
}

fn dense_to_bez_path(contour: &DenseContour) -> BezPath {
    let mut path = BezPath::new();
    if let Some(first) = contour.points.first() {
        path.move_to((first.x, first.y));
        for point in contour.points.iter().skip(1) {
            path.line_to((point.x, point.y));
        }
        path.close_path();
    }
    path
}

fn dense_from_bez_path(path: &BezPath) -> Result<DenseContour, String> {
    let mut points = Vec::new();
    kurbo::flatten(
        path.elements().iter().copied(),
        0.25,
        |element| match element {
            PathEl::MoveTo(point) | PathEl::LineTo(point) => points.push(from_bez(point)),
            PathEl::ClosePath => {}
            PathEl::QuadTo(_, _) | PathEl::CurveTo(_, _, _) => {
                unreachable!("flatten returns lines")
            }
        },
    );
    if points.last().is_some_and(|last| {
        points
            .first()
            .is_some_and(|first| distance_float(*last, *first) <= 1e-9)
    }) {
        points.pop();
    }
    if points.len() < 3 {
        return Err("native contour union produced fewer than three points".into());
    }
    let corners = polyline_corners(&points);
    Ok(DenseContour { points, corners })
}

fn contour_match_cost(reference: ContourFeature, target: ContourFeature) -> i64 {
    let center_distance = (reference.center_x - target.center_x).powi(2)
        + (reference.center_y - target.center_y).powi(2);
    let area_distance = (reference.area_fraction - target.area_fraction).powi(2);
    let winding_penalty = i64::from((reference.winding != target.winding) as i32) * 1_000_000_000;
    (center_distance * 10_000_000.0 + area_distance * 1_000_000.0)
        .round()
        .clamp(0.0, i64::MAX as f64) as i64
        + winding_penalty
}

fn match_contours_to_reference(
    reference: &[DenseContour],
    target: &[DenseContour],
) -> Result<Vec<DenseContour>, String> {
    let assignment = stable_contour_assignment(reference, target)?;
    let reference_bounds = dense_bounds(reference);
    let target_bounds = if target.is_empty() {
        reference_bounds
    } else {
        dense_bounds(target)
    };
    Ok(assignment
        .into_iter()
        .enumerate()
        .map(|(reference_index, target_index)| {
            target_index.map_or_else(
                || missing_contour(&reference[reference_index], reference_bounds, target_bounds),
                |target_index| target[target_index].clone(),
            )
        })
        .collect())
}

fn missing_contour(
    reference: &DenseContour,
    reference_bounds: (f64, f64, f64, f64),
    target_bounds: (f64, f64, f64, f64),
) -> DenseContour {
    let contour_bounds = bounds(&reference.points);
    let point = FloatPoint {
        x: denormalize_coordinate(
            normalize_coordinate(
                (contour_bounds.0 + contour_bounds.2) / 2.0,
                reference_bounds.0,
                reference_bounds.2,
            ),
            target_bounds.0,
            target_bounds.2,
        ),
        y: denormalize_coordinate(
            normalize_coordinate(
                (contour_bounds.1 + contour_bounds.3) / 2.0,
                reference_bounds.1,
                reference_bounds.3,
            ),
            target_bounds.1,
            target_bounds.3,
        ),
    };
    DenseContour {
        points: vec![point, point, point],
        corners: vec![true, false, false],
    }
}

fn dense_bounds(contours: &[DenseContour]) -> (f64, f64, f64, f64) {
    let points: Vec<_> = contours
        .iter()
        .flat_map(|contour| contour.points.iter().copied())
        .collect();
    bounds(&points)
}

fn contour_center(contour: &DenseContour) -> FloatPoint {
    let count = contour.points.len().max(1) as f64;
    let (x, y) = contour
        .points
        .iter()
        .fold((0.0, 0.0), |(x, y), point| (x + point.x, y + point.y));
    FloatPoint {
        x: x / count,
        y: y / count,
    }
}

fn cyclic_distance(left: usize, right: usize, length: usize) -> usize {
    let direct = left.abs_diff(right);
    direct.min(length.saturating_sub(direct))
}

fn contour_features(contours: &[DenseContour]) -> Vec<ContourFeature> {
    let all_points: Vec<_> = contours
        .iter()
        .flat_map(|contour| contour.points.iter().copied())
        .collect();
    let glyph_bounds = bounds(&all_points);
    let areas: Vec<_> = contours.iter().map(dense_signed_area).collect();
    let total_area = areas.iter().map(|area| area.abs()).sum::<f64>().max(1e-9);
    let dominant = areas
        .iter()
        .max_by(|left, right| {
            left.abs()
                .partial_cmp(&right.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
        .unwrap_or(1.0)
        .signum();

    contours
        .iter()
        .zip(areas)
        .map(|(contour, area)| {
            let contour_bounds = bounds(&contour.points);
            ContourFeature {
                center_x: normalize_coordinate(
                    (contour_bounds.0 + contour_bounds.2) / 2.0,
                    glyph_bounds.0,
                    glyph_bounds.2,
                ),
                center_y: normalize_coordinate(
                    (contour_bounds.1 + contour_bounds.3) / 2.0,
                    glyph_bounds.1,
                    glyph_bounds.3,
                ),
                area_fraction: area.abs() / total_area,
                winding: (area * dominant).signum() as i8,
            }
        })
        .collect()
}

fn resample_contour_set(
    contours: &[DenseContour],
    default_master: usize,
    strategy: ResamplingStrategy,
) -> Result<Vec<Vec<FloatPoint>>, String> {
    let mut aligned = contours.to_vec();
    // A counter may be absent from the default master, represented by a
    // zero-area ring. In that case its winding cannot define the slot: use the
    // first real instance so a negative counter remains opposite the body when
    // it appears along the axis.
    let winding_master = aligned
        .iter()
        .position(|contour| dense_signed_area(contour).abs() > 1e-9)
        .unwrap_or(default_master);
    let reference_area = dense_signed_area(&aligned[winding_master]);
    for contour in &mut aligned {
        if dense_signed_area(contour).signum() != reference_area.signum() {
            reverse_contour(contour);
        }
    }

    if matches!(strategy, ResamplingStrategy::Uniform) {
        // The uniform fallback intentionally ignores corner structure. Start at
        // each ring's topmost point, sample a shared number of arc-length
        // positions, then choose the least-squares cyclic rotation against the
        // default. This avoids a drifting corner anchor pairing unrelated
        // segments on asymmetric glyphs such as AE.
        for contour in &mut aligned {
            rotate_contour(contour, topmost_index(contour));
        }
        let mut resampled = resample_uniformly(&aligned)?;
        let reference = resampled[default_master].clone();
        for (master_index, contour) in resampled.iter_mut().enumerate() {
            if master_index != default_master {
                let rotation = best_point_rotation(&reference, contour);
                contour.rotate_left(rotation);
            }
        }
        return Ok(resampled);
    }

    let reference_anchor = anchor_index(&aligned[default_master]);
    rotate_contour(&mut aligned[default_master], reference_anchor);
    let reference = aligned[default_master].clone();
    let reference_corners = corner_indices(&reference);
    for (master_index, contour) in aligned.iter_mut().enumerate() {
        if master_index == default_master {
            continue;
        }
        let corners = corner_indices(contour);
        let start = if !reference_corners.is_empty() && corners.len() == reference_corners.len() {
            best_corner_rotation(&reference, contour, &reference_corners, &corners)
        } else {
            anchor_index(contour)
        };
        rotate_contour(contour, start);
    }

    let corner_sets: Vec<_> = aligned.iter().map(corner_indices).collect();
    let shared_corner_count = corner_sets[default_master].len();
    if shared_corner_count > 0
        && corner_sets
            .iter()
            .all(|corners| corners.len() == shared_corner_count)
    {
        resample_corner_runs(&aligned, &corner_sets, shared_corner_count)
    } else {
        // Do not discard the reference master's corner structure merely
        // because another release exposes a different number of corners.
        // Project its arc-length anchors onto the target rings, then sample
        // the matching runs. This is the deterministic reconstruction path
        // used by static-to-variable before it considers a uniform fallback.
        resample_projected_corner_runs(&aligned, default_master, &reference_corners)
    }
}

fn resample_all_contours(
    ordered: &[Vec<DenseContour>],
    contour_count: usize,
    default_master: usize,
    strategy: ResamplingStrategy,
) -> Result<Vec<Vec<Vec<FloatPoint>>>, String> {
    let mut resampled = vec![Vec::with_capacity(contour_count); ordered.len()];
    for contour_index in 0..contour_count {
        let contour_set: Vec<_> = ordered
            .iter()
            .map(|contours| contours[contour_index].clone())
            .collect();
        let contour_set = resample_contour_set(&contour_set, default_master, strategy)?;
        for (master, contour) in resampled.iter_mut().zip(contour_set) {
            master.push(contour);
        }
    }
    Ok(resampled)
}

fn reconstruct_candidate(
    ordered: &[Vec<DenseContour>],
    contour_count: usize,
    default_master: usize,
    strategy: ResamplingStrategy,
    span_pairs: &[(usize, usize)],
    donors: &[Vec<Vec<FloatPoint>>],
) -> Result<ReconstructionCandidate, String> {
    let contours = resample_all_contours(ordered, contour_count, default_master, strategy)?;
    reject_master_ink_deviation(&contours, donors)?;
    reject_interpolation_defects(&contours, span_pairs)?;
    reject_interpolation_area_and_perimeter_defects(&contours, span_pairs)?;
    let ink_defect = interpolation_ink_defect(&contours, span_pairs);
    if ink_defect > INK_FREEZE_TOLERANCE {
        return Err(format!(
            "ink quality gate measured a {ink_defect:.3} mid-axis defect"
        ));
    }
    Ok(ReconstructionCandidate {
        contours,
        ink_defect,
    })
}

fn interpolation_ink_defect(
    masters: &[Vec<Vec<FloatPoint>>],
    span_pairs: &[(usize, usize)],
) -> f64 {
    let mut worst = 0.0_f64;
    for &(left, right) in span_pairs {
        let (mut defect, union_count) = span_ink_defect_at_resolution(
            &masters[left],
            &masters[right],
            INK_RESOLUTION,
            INK_BLUR,
        );
        // Match mblode's small-shape safeguard. At 72px a quote, dot, or
        // accent can cover too few cells for the ratio to be meaningful, so
        // evaluate it again at the equivalent physical tolerance.
        if union_count < 500 {
            (defect, _) = span_ink_defect_at_resolution(
                &masters[left],
                &masters[right],
                INK_RESOLUTION * 2,
                INK_BLUR * 2,
            );
        }
        worst = worst.max(defect);
    }
    worst
}

fn span_ink_defect_at_resolution(
    left: &[Vec<FloatPoint>],
    right: &[Vec<FloatPoint>],
    resolution: usize,
    blur: usize,
) -> (f64, u32) {
    let all_points: Vec<_> = left
        .iter()
        .chain(right)
        .flat_map(|contour| contour.iter().copied())
        .collect();
    if all_points.is_empty() {
        return (0.0, 0);
    }
    let span_bounds = bounds(&all_points);
    let start = rasterize_ink_at_resolution(left, span_bounds, resolution);
    let end = rasterize_ink_at_resolution(right, span_bounds, resolution);
    let pre_blur_union_count = start.pairwise(&end, |left, right| left | right).count();
    if pre_blur_union_count == 0 {
        return (0.0, 0);
    }

    let mut shared = start.pairwise(&end, |left, right| left & right);
    let mut combined = start.pairwise(&end, |left, right| left | right);
    for _ in 0..blur {
        shared = erode_ink(&shared);
        combined = dilate_ink(&combined);
    }
    let denominator = shared.count().max(pre_blur_union_count / 10).max(1);

    let defect = INTERPOLATION_SAMPLES
        .iter()
        .copied()
        .map(|sample| {
            let midpoint: Vec<_> = left
                .iter()
                .zip(right)
                .map(|(left, right)| {
                    left.iter()
                        .zip(right)
                        .map(|(left, right)| lerp_float(*left, *right, sample))
                        .collect()
                })
                .collect();
            let midpoint = rasterize_ink_at_resolution(&midpoint, span_bounds, resolution);
            let mut midpoint_dilated = midpoint.clone();
            for _ in 0..blur {
                midpoint_dilated = dilate_ink(&midpoint_dilated);
            }
            let lost = count_and_not(&shared, &midpoint_dilated);
            let gained = count_and_not(&midpoint, &combined);
            f64::from(lost + gained) / f64::from(denominator)
        })
        .fold(0.0, f64::max);
    (defect, pre_blur_union_count)
}

fn rasterize_ink_at_resolution(
    contours: &[Vec<FloatPoint>],
    (x_min, y_min, x_max, y_max): (f64, f64, f64, f64),
    resolution: usize,
) -> InkRaster {
    let scale = (resolution.saturating_sub(2) as f64) / (x_max - x_min).max(y_max - y_min).max(1.0);
    let mut raster = InkRaster::empty(resolution);
    for row_index in 0..resolution {
        let y = y_min + (row_index as f64 + 0.5) / scale;
        let mut crossings = Vec::new();
        for contour in contours {
            if contour.is_empty() {
                continue;
            }
            for (index, start) in contour.iter().enumerate() {
                let end = contour[(index + 1) % contour.len()];
                if (start.y <= y && y < end.y) || (end.y <= y && y < start.y) {
                    let t = (y - start.y) / (end.y - start.y);
                    crossings.push((
                        start.x + (end.x - start.x) * t,
                        if end.y > start.y { 1 } else { -1 },
                    ));
                }
            }
        }
        crossings.sort_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut winding = 0_i32;
        let mut previous_x = x_min;
        for (x, delta) in crossings {
            if winding != 0 {
                // Python's int() truncates toward zero. This is intentionally
                // not floor(): it keeps the Rust scan conversion identical to
                // mblode around the left edge of the shared bbox.
                let first = ((previous_x - x_min) * scale) as i32;
                let last = ((x - x_min) * scale) as i32;
                raster.set_columns(row_index, first, last);
            }
            winding += delta;
            previous_x = x;
        }
    }
    raster
}

fn count_and_not(left: &InkRaster, right: &InkRaster) -> u32 {
    debug_assert_eq!(left.resolution, right.resolution);
    left.rows
        .iter()
        .zip(&right.rows)
        .flat_map(|(left, right)| left.iter().zip(right))
        .map(|(left, right)| (left & !right).count_ones())
        .sum()
}

fn erode_ink(input: &InkRaster) -> InkRaster {
    let mut result = InkRaster::empty(input.resolution);
    let last_mask = input.last_word_mask();
    for row_index in 1..input.rows.len().saturating_sub(1) {
        let right = InkRaster::shifted_right(&input.rows[row_index]);
        let left = InkRaster::shifted_left(&input.rows[row_index], last_mask);
        for word_index in 0..input.word_count() {
            result.rows[row_index][word_index] = input.rows[row_index][word_index]
                & right[word_index]
                & left[word_index]
                & input.rows[row_index - 1][word_index]
                & input.rows[row_index + 1][word_index];
        }
    }
    result
}

fn dilate_ink(input: &InkRaster) -> InkRaster {
    let mut result = InkRaster::empty(input.resolution);
    let last_mask = input.last_word_mask();
    for row_index in 0..input.rows.len() {
        let vertical: Vec<_> = (0..input.word_count())
            .map(|word_index| {
                input.rows[row_index][word_index]
                    | row_index
                        .checked_sub(1)
                        .map_or(0, |previous| input.rows[previous][word_index])
                    | input
                        .rows
                        .get(row_index + 1)
                        .map_or(0, |next| next[word_index])
            })
            .collect();
        let right = InkRaster::shifted_right(&vertical);
        let left = InkRaster::shifted_left(&vertical, last_mask);
        for word_index in 0..input.word_count() {
            result.rows[row_index][word_index] =
                vertical[word_index] | right[word_index] | left[word_index];
        }
    }
    result
}

/// Reject a topology reconstruction whose rendered master differs materially
/// from its donor. This is mblode's `_quality_offenders` gate: point counts
/// can match while a repair has accidentally filled a counter or discarded an
/// attached piece.
fn reject_master_ink_deviation(
    reconstructed: &[Vec<Vec<FloatPoint>>],
    donors: &[Vec<Vec<FloatPoint>>],
) -> Result<(), String> {
    if reconstructed.len() != donors.len() {
        return Err("reconstruction and donor master counts differ".into());
    }
    for (master_index, (output, donor)) in reconstructed.iter().zip(donors).enumerate() {
        let points: Vec<_> = output
            .iter()
            .chain(donor)
            .flat_map(|contour| contour.iter().copied())
            .collect();
        if points.is_empty() {
            continue;
        }
        let shared_bounds = bounds(&points);
        let donor_ink = rasterize_ink_at_resolution(donor, shared_bounds, INK_RESOLUTION).count();
        if donor_ink == 0 {
            continue;
        }
        let output_ink = rasterize_ink_at_resolution(output, shared_bounds, INK_RESOLUTION).count();
        let deviation = (f64::from(output_ink) / f64::from(donor_ink) - 1.0).abs();
        if deviation > QUALITY_AREA_TOLERANCE {
            return Err(format!(
                "master {master_index} would change rendered ink by {deviation:.2}"
            ));
        }
    }
    Ok(())
}

fn resample_corner_runs(
    contours: &[DenseContour],
    corner_sets: &[Vec<usize>],
    corner_count: usize,
) -> Result<Vec<Vec<FloatPoint>>, String> {
    let mut interior_counts = Vec::with_capacity(corner_count);
    for run_index in 0..corner_count {
        let mut target_count = 1;
        for (contour, corners) in contours.iter().zip(corner_sets) {
            let run = ring_slice(
                &contour.points,
                corners[run_index],
                corners[(run_index + 1) % corner_count],
            );
            let length = polyline_length(&run);
            let segments = (length / reconstruction_step(length)).ceil() as usize;
            target_count = target_count.max(segments.saturating_sub(1).max(1));
        }
        interior_counts.push(target_count);
    }

    contours
        .iter()
        .zip(corner_sets)
        .map(|(contour, corners)| {
            let mut output = Vec::new();
            for run_index in 0..corner_count {
                let run = ring_slice(
                    &contour.points,
                    corners[run_index],
                    corners[(run_index + 1) % corner_count],
                );
                output.push(run[0]);
                output.extend(resample_polyline(&run, interior_counts[run_index]));
            }
            if output.len() > MAX_POINTS_PER_CONTOUR {
                return Err(format!(
                    "a contour would require more than {MAX_POINTS_PER_CONTOUR} reconstruction points"
                ));
            }
            Ok(output)
        })
        .collect()
}

fn resample_projected_corner_runs(
    contours: &[DenseContour],
    default_master: usize,
    reference_corners: &[usize],
) -> Result<Vec<Vec<FloatPoint>>, String> {
    if reference_corners.is_empty() {
        return resample_uniformly(contours);
    }

    let reference = &contours[default_master];
    let reference_fractions = closed_fractions(&reference.points);
    let anchors: Vec<_> = reference_corners
        .iter()
        .map(|corner| reference_fractions[*corner])
        .collect();
    let run_count = anchors.len();
    let mut interior_counts = Vec::with_capacity(run_count);
    for run_index in 0..run_count {
        let start = anchors[run_index];
        let end = anchors[(run_index + 1) % run_count];
        let fraction = if end > start {
            end - start
        } else {
            1.0 - start + end
        };
        let target_count = contours
            .iter()
            .map(|contour| {
                let length = polyline_length(&contour.points) * fraction;
                (length / reconstruction_step(length)).ceil() as usize
            })
            .max()
            .unwrap_or(1)
            .saturating_sub(1)
            .max(1);
        interior_counts.push(target_count);
    }

    contours
        .iter()
        .map(|contour| {
            let total_length = polyline_length(&contour.points);
            if total_length <= 1e-9 {
                return Err("a contour has no measurable perimeter".into());
            }
            let cumulative = cumulative_lengths(&contour.points, true);
            let mut output = Vec::new();
            for run_index in 0..run_count {
                let start = anchors[run_index];
                let end = anchors[(run_index + 1) % run_count];
                let fraction = if end > start { end - start } else { 1.0 - start + end };
                output.push(point_at_distance(
                    &contour.points,
                    &cumulative,
                    start * total_length,
                ));
                for interior in 1..=interior_counts[run_index] {
                    let progress = interior as f64 / (interior_counts[run_index] + 1) as f64;
                    let position = (start + fraction * progress).rem_euclid(1.0);
                    output.push(point_at_distance(
                        &contour.points,
                        &cumulative,
                        position * total_length,
                    ));
                }
            }
            if output.len() > MAX_POINTS_PER_CONTOUR {
                return Err(format!(
                    "a contour would require more than {MAX_POINTS_PER_CONTOUR} reconstruction points"
                ));
            }
            Ok(output)
        })
        .collect()
}

fn resample_uniformly(contours: &[DenseContour]) -> Result<Vec<Vec<FloatPoint>>, String> {
    let count = contours
        .iter()
        .map(|contour| {
            let length = polyline_length(&contour.points);
            (length / reconstruction_step(length)).ceil() as usize
        })
        .max()
        .unwrap_or(0)
        .clamp(8, MAX_POINTS_PER_CONTOUR);
    contours
        .iter()
        .map(|contour| sample_closed_ring(&contour.points, count))
        .collect()
}

fn best_corner_rotation(
    reference: &DenseContour,
    target: &DenseContour,
    reference_corners: &[usize],
    target_corners: &[usize],
) -> usize {
    let reference_bounds = bounds(&reference.points);
    let target_bounds = bounds(&target.points);
    (0..target_corners.len())
        .map(|offset| {
            let cost = reference_corners
                .iter()
                .enumerate()
                .map(|(index, reference_index)| {
                    let target_index = target_corners[(index + offset) % target_corners.len()];
                    let reference_point = reference.points[*reference_index];
                    let target_point = target.points[target_index];
                    let x =
                        normalize_coordinate(
                            reference_point.x,
                            reference_bounds.0,
                            reference_bounds.2,
                        ) - normalize_coordinate(target_point.x, target_bounds.0, target_bounds.2);
                    let y =
                        normalize_coordinate(
                            reference_point.y,
                            reference_bounds.1,
                            reference_bounds.3,
                        ) - normalize_coordinate(target_point.y, target_bounds.1, target_bounds.3);
                    x * x + y * y
                })
                .sum::<f64>();
            (target_corners[offset], cost)
        })
        .min_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
        .unwrap_or_else(|| anchor_index(target))
}

fn anchor_index(contour: &DenseContour) -> usize {
    let choices: Vec<_> = corner_indices(contour);
    let choices: Vec<_> = if choices.is_empty() {
        (0..contour.points.len()).collect()
    } else {
        choices
    };
    choices
        .into_iter()
        .max_by(|left, right| {
            let left = contour.points[*left];
            let right = contour.points[*right];
            left.y
                .partial_cmp(&right.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    left.x
                        .partial_cmp(&right.x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
        .unwrap_or(0)
}

fn topmost_index(contour: &DenseContour) -> usize {
    contour
        .points
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| {
            left.y
                .partial_cmp(&right.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    left.x
                        .partial_cmp(&right.x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn best_point_rotation(reference: &[FloatPoint], target: &[FloatPoint]) -> usize {
    if reference.len() != target.len() || target.is_empty() {
        return 0;
    }
    (0..target.len())
        .map(|offset| {
            let cost = reference
                .iter()
                .enumerate()
                .map(|(index, point)| {
                    let target = target[(index + offset) % target.len()];
                    (point.x - target.x).powi(2) + (point.y - target.y).powi(2)
                })
                .sum::<f64>();
            (offset, cost)
        })
        .min_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(offset, _)| offset)
        .unwrap_or(0)
}

fn corner_indices(contour: &DenseContour) -> Vec<usize> {
    contour
        .corners
        .iter()
        .enumerate()
        .filter_map(|(index, corner)| corner.then_some(index))
        .collect()
}

fn polyline_corners(points: &[FloatPoint]) -> Vec<bool> {
    points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let previous = points[(index + points.len() - 1) % points.len()];
            let next = points[(index + 1) % points.len()];
            let incoming = FloatPoint {
                x: point.x - previous.x,
                y: point.y - previous.y,
            };
            let outgoing = FloatPoint {
                x: next.x - point.x,
                y: next.y - point.y,
            };
            let incoming_length = incoming.x.hypot(incoming.y);
            let outgoing_length = outgoing.x.hypot(outgoing.y);
            if incoming_length <= 1e-9 || outgoing_length <= 1e-9 {
                return true;
            }
            let cosine = ((incoming.x * outgoing.x + incoming.y * outgoing.y)
                / (incoming_length * outgoing_length))
                .clamp(-1.0, 1.0);
            cosine.acos() > DEFAULT_CORNER_ANGLE_RADIANS
        })
        .collect()
}

fn reverse_contour(contour: &mut DenseContour) {
    contour.points.reverse();
    contour.corners.reverse();
}

fn rotate_contour(contour: &mut DenseContour, start: usize) {
    contour.points.rotate_left(start);
    contour.corners.rotate_left(start);
}

fn ring_slice(points: &[FloatPoint], start: usize, end: usize) -> Vec<FloatPoint> {
    if start == end {
        let mut result = points[start..].to_vec();
        result.extend_from_slice(&points[..start]);
        result.push(points[start]);
        return result;
    }
    if start < end {
        return points[start..=end].to_vec();
    }
    let mut result = points[start..].to_vec();
    result.extend_from_slice(&points[..=end]);
    result
}

fn resample_polyline(points: &[FloatPoint], count: usize) -> Vec<FloatPoint> {
    if count == 0 || points.len() < 2 {
        return Vec::new();
    }
    let cumulative = cumulative_lengths(points, false);
    let total = *cumulative.last().unwrap_or(&0.0);
    if total <= 1e-9 {
        return vec![points[0]; count];
    }
    (1..=count)
        .map(|index| {
            point_at_distance(
                points,
                &cumulative,
                total * index as f64 / (count + 1) as f64,
            )
        })
        .collect()
}

fn sample_closed_ring(points: &[FloatPoint], count: usize) -> Result<Vec<FloatPoint>, String> {
    if points.len() < 3 {
        return Err("a contour has fewer than three drawable points".into());
    }
    let cumulative = cumulative_lengths(points, true);
    let total = *cumulative.last().unwrap_or(&0.0);
    if total <= 1e-9 {
        return Ok(vec![points[0]; count]);
    }
    Ok((0..count)
        .map(|index| point_at_distance(points, &cumulative, total * index as f64 / count as f64))
        .collect())
}

fn cumulative_lengths(points: &[FloatPoint], closed: bool) -> Vec<f64> {
    let segment_count = points.len().saturating_sub(1) + usize::from(closed);
    let mut cumulative = Vec::with_capacity(segment_count + 1);
    cumulative.push(0.0);
    for index in 0..segment_count {
        let start = points[index % points.len()];
        let end = points[(index + 1) % points.len()];
        cumulative.push(cumulative.last().copied().unwrap_or(0.0) + distance_float(start, end));
    }
    cumulative
}

fn closed_fractions(points: &[FloatPoint]) -> Vec<f64> {
    let cumulative = cumulative_lengths(points, true);
    let total = cumulative.last().copied().unwrap_or(0.0);
    if total <= 1e-9 {
        return vec![0.0; points.len()];
    }
    cumulative[..points.len()]
        .iter()
        .map(|length| length / total)
        .collect()
}

fn point_at_distance(points: &[FloatPoint], cumulative: &[f64], target: f64) -> FloatPoint {
    let segment = cumulative
        .windows(2)
        .position(|window| target <= window[1])
        .unwrap_or(cumulative.len().saturating_sub(2));
    let start = points[segment % points.len()];
    let end = points[(segment + 1) % points.len()];
    let length = cumulative[segment + 1] - cumulative[segment];
    let ratio = if length <= 1e-9 {
        0.0
    } else {
        (target - cumulative[segment]) / length
    };
    lerp_float(start, end, ratio)
}

fn polyline_length(points: &[FloatPoint]) -> f64 {
    cumulative_lengths(points, true)
        .last()
        .copied()
        .unwrap_or(0.0)
}

fn reconstruction_step(length: f64) -> f64 {
    RESAMPLE_STEP.max(length / MAX_POINTS_PER_CONTOUR as f64)
}

fn reject_interpolation_defects(
    masters: &[Vec<Vec<FloatPoint>>],
    span_pairs: &[(usize, usize)],
) -> Result<(), String> {
    for &(left_index, right_index) in span_pairs {
        let left = &masters[left_index];
        let right = &masters[right_index];
        for contour_index in 0..left.len() {
            if polygon_self_intersects(&left[contour_index])
                || polygon_self_intersects(&right[contour_index])
            {
                continue;
            }
            for sample in INTERPOLATION_SAMPLES {
                let midpoint: Vec<_> = left[contour_index]
                    .iter()
                    .zip(&right[contour_index])
                    .map(|(left, right)| lerp_float(*left, *right, sample))
                    .collect();
                if polygon_self_intersects(&midpoint) {
                    return Err(format!(
                        "contour {contour_index} would fold over itself between masters {left_index} and {right_index}"
                    ));
                }
            }
        }
        for first in 0..left.len() {
            for second in first + 1..left.len() {
                let left_a = bounds(&left[first]);
                let left_b = bounds(&left[second]);
                let right_a = bounds(&right[first]);
                let right_b = bounds(&right[second]);
                if boxes_overlap(left_a, left_b) || boxes_overlap(right_a, right_b) {
                    continue;
                }
                for sample in INTERPOLATION_SAMPLES {
                    let midpoint_a: Vec<_> = left[first]
                        .iter()
                        .zip(&right[first])
                        .map(|(left, right)| lerp_float(*left, *right, sample))
                        .collect();
                    let midpoint_b: Vec<_> = left[second]
                        .iter()
                        .zip(&right[second])
                        .map(|(left, right)| lerp_float(*left, *right, sample))
                        .collect();
                    if boxes_overlap(bounds(&midpoint_a), bounds(&midpoint_b))
                        && polygons_intersect(&midpoint_a, &midpoint_b)
                    {
                        return Err(format!(
                            "contours {first} and {second} would collide between masters {left_index} and {right_index}"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Port of mblode's analytic `_interp_ok` guard.
///
/// A point-compatible pair can avoid literal segment intersections yet still
/// lose a counter or twist a diagonal in the middle of an axis. Area catches
/// most collapses; perimeter catches the folds that preserve area. We retain
/// the existing intersection tests as a complementary, stricter structural
/// check.
fn reject_interpolation_area_and_perimeter_defects(
    masters: &[Vec<Vec<FloatPoint>>],
    span_pairs: &[(usize, usize)],
) -> Result<(), String> {
    const MIDPOINT_AREA_TOLERANCE: f64 = 0.18;
    const MIDPOINT_PERIMETER_TOLERANCE: f64 = 0.83;
    const PERIMETER_MINIMUM: f64 = 500.0;
    const CONTOUR_AREA_MINIMUM: f64 = 1_500.0;
    const CONTOUR_AREA_RATIO: f64 = 0.45;

    for &(left_index, right_index) in span_pairs {
        let left = &masters[left_index];
        let right = &masters[right_index];
        let left_area = containment_aware_area(left);
        let right_area = containment_aware_area(right);
        let mean_area = (left_area + right_area) / 2.0;
        if mean_area <= 0.0 {
            continue;
        }

        let mut midpoint = Vec::with_capacity(left.len());
        for (contour_index, (left_contour, right_contour)) in left.iter().zip(right).enumerate() {
            if left_contour.len() != right_contour.len() {
                return Err(format!(
                    "contour {contour_index} changes point count between masters {left_index} and {right_index}"
                ));
            }
            let midpoint_contour: Vec<_> = left_contour
                .iter()
                .zip(right_contour)
                .map(|(left, right)| lerp_float(*left, *right, 0.5))
                .collect();
            if midpoint_contour.len() >= 3 {
                let mean_perimeter =
                    (ring_perimeter(left_contour) + ring_perimeter(right_contour)) / 2.0;
                if mean_perimeter > PERIMETER_MINIMUM
                    && ring_perimeter(&midpoint_contour) / mean_perimeter
                        < MIDPOINT_PERIMETER_TOLERANCE
                {
                    return Err(format!(
                        "contour {contour_index} would lose too much perimeter between masters {left_index} and {right_index}"
                    ));
                }
            }
            midpoint.push(midpoint_contour);
        }
        if (containment_aware_area(&midpoint) / mean_area - 1.0).abs() > MIDPOINT_AREA_TOLERANCE {
            return Err(format!(
                "glyph ink area would collapse between masters {left_index} and {right_index}"
            ));
        }
        for (contour_index, ((left_contour, right_contour), midpoint_contour)) in
            left.iter().zip(right).zip(&midpoint).enumerate()
        {
            let endpoint_area = (signed_ring_area(left_contour).abs()
                + signed_ring_area(right_contour).abs())
                / 2.0;
            if endpoint_area > CONTOUR_AREA_MINIMUM
                && signed_ring_area(midpoint_contour).abs() / endpoint_area < CONTOUR_AREA_RATIO
            {
                return Err(format!(
                    "contour {contour_index} would collapse between masters {left_index} and {right_index}"
                ));
            }
        }
    }
    Ok(())
}

fn containment_aware_area(contours: &[Vec<FloatPoint>]) -> f64 {
    let contour_bounds: Vec<_> = contours
        .iter()
        .map(|contour| (!contour.is_empty()).then(|| bounds(contour)))
        .collect();
    let mut total = 0.0;
    for (index, contour) in contours.iter().enumerate() {
        if contour.len() < 3 {
            continue;
        }
        let center = FloatPoint {
            x: contour.iter().map(|point| point.x).sum::<f64>() / contour.len() as f64,
            y: contour.iter().map(|point| point.y).sum::<f64>() / contour.len() as f64,
        };
        let current_bounds = contour_bounds[index].expect("non-empty contour has bounds");
        let mut depth = 0;
        for (other_index, other) in contours.iter().enumerate() {
            if index == other_index || other.len() < 3 {
                continue;
            }
            let other_bounds = contour_bounds[other_index].expect("non-empty contour has bounds");
            if current_bounds.0 >= other_bounds.0 - 1.0
                && current_bounds.1 >= other_bounds.1 - 1.0
                && current_bounds.2 <= other_bounds.2 + 1.0
                && current_bounds.3 <= other_bounds.3 + 1.0
                && point_in_ring(center, other)
            {
                depth += 1;
            }
        }
        let area = signed_ring_area(contour).abs();
        total += if depth % 2 == 0 { area } else { -area };
    }
    total.abs()
}

fn point_in_ring(point: FloatPoint, ring: &[FloatPoint]) -> bool {
    let mut inside = false;
    for (index, start) in ring.iter().enumerate() {
        let end = ring[(index + 1) % ring.len()];
        if ((start.y <= point.y && point.y < end.y) || (end.y <= point.y && point.y < start.y))
            && point.x < start.x + (point.y - start.y) / (end.y - start.y) * (end.x - start.x)
        {
            inside = !inside;
        }
    }
    inside
}

fn signed_ring_area(points: &[FloatPoint]) -> f64 {
    points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let next = points[(index + 1) % points.len()];
            point.x * next.y - next.x * point.y
        })
        .sum::<f64>()
        / 2.0
}

fn ring_perimeter(points: &[FloatPoint]) -> f64 {
    points
        .iter()
        .enumerate()
        .map(|(index, point)| distance_float(*point, points[(index + 1) % points.len()]))
        .sum()
}

fn master_span_pairs(locations: &[Vec<f64>], master_count: usize) -> Vec<(usize, usize)> {
    if locations.len() != master_count || locations.iter().any(|location| location.is_empty()) {
        return all_master_pairs(master_count);
    }
    let axis_count = locations[0].len();
    if axis_count == 0
        || locations
            .iter()
            .any(|location| location.len() != axis_count)
    {
        return all_master_pairs(master_count);
    }

    let mut pairs = Vec::new();
    for axis in 0..axis_count {
        for seed in 0..master_count {
            let mut group: Vec<_> = (0..master_count)
                .filter(|candidate| {
                    (0..axis_count).all(|other_axis| {
                        other_axis == axis
                            || (locations[*candidate][other_axis] - locations[seed][other_axis])
                                .abs()
                                <= 1e-9
                    })
                })
                .collect();
            group.sort_by(|left, right| locations[*left][axis].total_cmp(&locations[*right][axis]));
            for window in group.windows(2) {
                let pair = (window[0].min(window[1]), window[0].max(window[1]));
                if !pairs.contains(&pair) {
                    pairs.push(pair);
                }
            }
        }
    }
    if pairs.is_empty() {
        all_master_pairs(master_count)
    } else {
        pairs
    }
}

fn all_master_pairs(master_count: usize) -> Vec<(usize, usize)> {
    (0..master_count)
        .flat_map(|left| ((left + 1)..master_count).map(move |right| (left, right)))
        .collect()
}

fn polygons_intersect(left: &[FloatPoint], right: &[FloatPoint]) -> bool {
    left.iter().enumerate().any(|(index, start)| {
        let end = left[(index + 1) % left.len()];
        right.iter().enumerate().any(|(other_index, other_start)| {
            let other_end = right[(other_index + 1) % right.len()];
            segments_intersect(*start, end, *other_start, other_end)
        })
    })
}

fn polygon_strictly_contains(polygon: &[FloatPoint], point: FloatPoint) -> bool {
    let mut inside = false;
    for (index, start) in polygon.iter().enumerate() {
        let end = polygon[(index + 1) % polygon.len()];
        if point_on_segment(point, *start, end) {
            return false;
        }
        let crosses_ray = (start.y > point.y) != (end.y > point.y)
            && point.x < (end.x - start.x) * (point.y - start.y) / (end.y - start.y) + start.x;
        if crosses_ray {
            inside = !inside;
        }
    }
    inside
}

fn point_on_segment(point: FloatPoint, start: FloatPoint, end: FloatPoint) -> bool {
    let cross = (point.x - start.x) * (end.y - start.y) - (point.y - start.y) * (end.x - start.x);
    cross.abs() <= 1e-6
        && point.x >= start.x.min(end.x) - 1e-6
        && point.x <= start.x.max(end.x) + 1e-6
        && point.y >= start.y.min(end.y) - 1e-6
        && point.y <= start.y.max(end.y) + 1e-6
}

fn polygon_self_intersects(points: &[FloatPoint]) -> bool {
    points.iter().enumerate().any(|(index, start)| {
        let end = points[(index + 1) % points.len()];
        points.iter().enumerate().any(|(other_index, other_start)| {
            let shares_endpoint = index == other_index
                || (index + 1) % points.len() == other_index
                || (other_index + 1) % points.len() == index;
            !shares_endpoint
                && segments_intersect(
                    *start,
                    end,
                    *other_start,
                    points[(other_index + 1) % points.len()],
                )
        })
    })
}

fn segments_intersect(a: FloatPoint, b: FloatPoint, c: FloatPoint, d: FloatPoint) -> bool {
    fn orientation(a: FloatPoint, b: FloatPoint, c: FloatPoint) -> f64 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }
    let first = orientation(a, b, c);
    let second = orientation(a, b, d);
    let third = orientation(c, d, a);
    let fourth = orientation(c, d, b);
    (first > 1e-6 && second < -1e-6 || first < -1e-6 && second > 1e-6)
        && (third > 1e-6 && fourth < -1e-6 || third < -1e-6 && fourth > 1e-6)
}

fn make_simple_glyph(contours: &[Vec<Point>]) -> SimpleGlyph {
    let mut glyph = SimpleGlyph {
        contours: contours
            .iter()
            .map(|contour| {
                Contour::from(
                    contour
                        .iter()
                        .map(|point| CurvePoint::on_curve(point.x, point.y))
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
        ..Default::default()
    };
    glyph.recompute_bounding_box();
    glyph
}

fn float_to_point(point: FloatPoint) -> Result<Point, String> {
    let x = point.x.round_ties_even() as i64;
    let y = point.y.round_ties_even() as i64;
    if !(-32768..=32767).contains(&x) || !(-32768..=32767).contains(&y) {
        return Err("a reconstructed point is outside TrueType coordinates".into());
    }
    Ok(Point {
        x: x as i16,
        y: y as i16,
        on_curve: true,
    })
}

fn is_corner(curves: &[Quad], index: usize, corner_angle: f64) -> bool {
    let incoming = tangent_into(curves[(index + curves.len() - 1) % curves.len()]);
    let outgoing = tangent_out_of(curves[index]);
    let incoming_length = (incoming.x * incoming.x + incoming.y * incoming.y).sqrt();
    let outgoing_length = (outgoing.x * outgoing.x + outgoing.y * outgoing.y).sqrt();
    if incoming_length <= 1e-9 || outgoing_length <= 1e-9 {
        return true;
    }
    let cosine = ((incoming.x * outgoing.x + incoming.y * outgoing.y)
        / (incoming_length * outgoing_length))
        .clamp(-1.0, 1.0);
    cosine.acos() > corner_angle
}

fn tangent_into(curve: Quad) -> FloatPoint {
    let control = from_bez(curve.control);
    let end = from_bez(curve.end);
    let start = from_bez(curve.start);
    if distance_float(control, end) > 1e-9 {
        FloatPoint {
            x: end.x - control.x,
            y: end.y - control.y,
        }
    } else {
        FloatPoint {
            x: end.x - start.x,
            y: end.y - start.y,
        }
    }
}

fn tangent_out_of(curve: Quad) -> FloatPoint {
    let start = from_bez(curve.start);
    let control = from_bez(curve.control);
    let end = from_bez(curve.end);
    if distance_float(start, control) > 1e-9 {
        FloatPoint {
            x: control.x - start.x,
            y: control.y - start.y,
        }
    } else {
        FloatPoint {
            x: end.x - start.x,
            y: end.y - start.y,
        }
    }
}

fn evaluate_quad(curve: Quad, t: f64) -> FloatPoint {
    let start = from_bez(curve.start);
    let control = from_bez(curve.control);
    let end = from_bez(curve.end);
    let inverse = 1.0 - t;
    FloatPoint {
        x: inverse * inverse * start.x + 2.0 * inverse * t * control.x + t * t * end.x,
        y: inverse * inverse * start.y + 2.0 * inverse * t * control.y + t * t * end.y,
    }
}

fn from_bez(point: BezPoint) -> FloatPoint {
    FloatPoint {
        x: point.x,
        y: point.y,
    }
}

fn dense_signed_area(contour: &DenseContour) -> f64 {
    contour
        .points
        .iter()
        .zip(contour.points.iter().cycle().skip(1))
        .take(contour.points.len())
        .map(|(left, right)| left.x * right.y - right.x * left.y)
        .sum::<f64>()
        / 2.0
}

fn bounds(points: &[FloatPoint]) -> (f64, f64, f64, f64) {
    points.iter().fold(
        (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ),
        |(x_min, y_min, x_max, y_max), point| {
            (
                x_min.min(point.x),
                y_min.min(point.y),
                x_max.max(point.x),
                y_max.max(point.y),
            )
        },
    )
}

fn boxes_overlap(left: (f64, f64, f64, f64), right: (f64, f64, f64, f64)) -> bool {
    left.0 < right.2 && right.0 < left.2 && left.1 < right.3 && right.1 < left.3
}

fn normalize_coordinate(value: f64, minimum: f64, maximum: f64) -> f64 {
    let span = maximum - minimum;
    if span.abs() <= 1e-9 {
        0.5
    } else {
        (value - minimum) / span
    }
}

fn denormalize_coordinate(value: f64, minimum: f64, maximum: f64) -> f64 {
    minimum + (maximum - minimum) * value
}

fn distance_float(left: FloatPoint, right: FloatPoint) -> f64 {
    (right.x - left.x).hypot(right.y - left.y)
}

fn lerp_float(left: FloatPoint, right: FloatPoint, t: f64) -> FloatPoint {
    FloatPoint {
        x: left.x + (right.x - left.x) * t,
        y: left.y + (right.y - left.y) * t,
    }
}

fn contour_to_quadratics(contour: &[Point]) -> Result<Vec<Quad>, String> {
    if contour.len() < 2 {
        return Err("a contour has fewer than two points".into());
    }
    let first_on_curve = contour.iter().position(|point| point.on_curve);
    let (start, mut index) = match first_on_curve {
        Some(index) => (as_bez(contour[index]), (index + 1) % contour.len()),
        None => (
            midpoint(as_bez(*contour.last().unwrap()), as_bez(contour[0])),
            0,
        ),
    };
    let mut current = start;
    let mut curves = Vec::new();
    loop {
        let point = contour[index];
        if point.on_curve {
            let end = as_bez(point);
            curves.push(line_as_quad(current, end));
            current = end;
            index = (index + 1) % contour.len();
        } else {
            let next_index = (index + 1) % contour.len();
            let next = contour[next_index];
            let end = if next.on_curve {
                as_bez(next)
            } else {
                midpoint(as_bez(point), as_bez(next))
            };
            curves.push(Quad {
                start: current,
                control: as_bez(point),
                end,
            });
            current = end;
            index = if next.on_curve {
                if Some(next_index) == first_on_curve {
                    next_index
                } else {
                    (next_index + 1) % contour.len()
                }
            } else {
                next_index
            };
        }
        if index == first_on_curve.unwrap_or(0) {
            break;
        }
    }
    if !same_point(current, start) {
        curves.push(line_as_quad(current, start));
    }
    Ok(curves)
}

fn curve_length_hint(curve: Quad) -> f64 {
    distance(curve.start, curve.control) + distance(curve.control, curve.end)
}

fn line_as_quad(start: BezPoint, end: BezPoint) -> Quad {
    Quad {
        start,
        control: midpoint(start, end),
        end,
    }
}

fn as_bez(point: Point) -> BezPoint {
    BezPoint::new(f64::from(point.x), f64::from(point.y))
}

fn midpoint(left: BezPoint, right: BezPoint) -> BezPoint {
    BezPoint::new((left.x + right.x) / 2.0, (left.y + right.y) / 2.0)
}

fn distance(left: BezPoint, right: BezPoint) -> f64 {
    (right.x - left.x).hypot(right.y - left.y)
}

fn same_point(left: BezPoint, right: BezPoint) -> bool {
    (left.x - right.x).abs() < 1e-9 && (left.y - right.y).abs() < 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: i16, y: i16, on_curve: bool) -> Point {
        Point { x, y, on_curve }
    }

    fn dense_polygon(points: &[(f64, f64)]) -> DenseContour {
        let points: Vec<_> = points
            .iter()
            .map(|(x, y)| FloatPoint { x: *x, y: *y })
            .collect();
        DenseContour {
            corners: polyline_corners(&points),
            points,
        }
    }

    #[test]
    fn reconciles_contours_with_different_point_counts() {
        let masters = vec![
            vec![vec![
                point(0, 0, true),
                point(100, 0, true),
                point(100, 100, true),
                point(0, 100, true),
            ]],
            vec![vec![
                point(0, 0, true),
                point(50, 0, true),
                point(100, 0, true),
                point(100, 100, true),
                point(0, 100, true),
            ]],
        ];
        let normalized = normalize_simple_glyphs(&masters, &[vec![0.0], vec![1.0]], 1).unwrap();
        assert_eq!(
            normalized.master_contours[0][0].len(),
            normalized.master_contours[1][0].len()
        );
        assert_eq!(
            normalized.default_glyph.contours[0].len(),
            normalized.master_contours[1][0].len()
        );
        assert_eq!(
            normalized.default_glyph.contours[0]
                .iter()
                .map(|point| Point {
                    x: point.x,
                    y: point.y,
                    on_curve: point.on_curve,
                })
                .collect::<Vec<_>>(),
            normalized.master_contours[1][0]
        );
    }

    #[test]
    fn collapses_a_missing_contour_at_its_normalized_position() {
        let square = vec![
            point(0, 0, true),
            point(100, 0, true),
            point(100, 100, true),
            point(0, 100, true),
        ];
        let dot = vec![
            point(20, 20, true),
            point(40, 20, true),
            point(40, 40, true),
            point(20, 40, true),
        ];
        let normalized = normalize_simple_glyphs(
            &[vec![square.clone()], vec![square, dot]],
            &[vec![0.0], vec![1.0]],
            0,
        )
        .unwrap();
        assert_eq!(normalized.master_contours[0].len(), 2);
        assert_eq!(normalized.master_contours[1].len(), 2);
        assert!(
            normalized.master_contours[0][1]
                .windows(2)
                .all(|points| points[0] == points[1])
        );
    }

    #[test]
    fn preserves_counter_winding_when_a_hole_closes_in_one_master() {
        let outer = vec![
            point(0, 0, true),
            point(100, 0, true),
            point(100, 100, true),
            point(0, 100, true),
        ];
        // Reverse direction from the outer ring so this is a nonzero-winding
        // counter rather than a second filled piece.
        let counter = vec![
            point(30, 30, true),
            point(30, 70, true),
            point(70, 70, true),
            point(70, 30, true),
        ];
        let normalized = normalize_simple_glyphs(
            &[vec![outer.clone()], vec![outer, counter]],
            &[vec![0.0], vec![1.0]],
            0,
        )
        .unwrap();
        let signed_area = |contour: &[Point]| {
            contour
                .iter()
                .zip(contour.iter().cycle().skip(1))
                .take(contour.len())
                .map(|(left, right)| {
                    f64::from(left.x) * f64::from(right.y) - f64::from(right.x) * f64::from(left.y)
                })
                .sum::<f64>()
                / 2.0
        };

        assert_eq!(normalized.master_contours[0].len(), 2);
        assert!(
            normalized.master_contours[0][1]
                .windows(2)
                .all(|points| points[0] == points[1])
        );
        assert!(
            signed_area(&normalized.master_contours[1][0])
                * signed_area(&normalized.master_contours[1][1])
                < 0.0
        );
    }

    #[test]
    fn counter_closing_synthesizes_a_tiny_winding_aware_hole() {
        let body = dense_polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]);
        let counter = dense_polygon(&[(30.0, 30.0), (30.0, 70.0), (70.0, 70.0), (70.0, 30.0)]);
        let reconstructed = counter_closing_topology(
            &[vec![body.clone(), counter.clone()], vec![body]],
            &[vec![400.0], vec![900.0]],
        )
        .expect("a closing counter should use the dedicated repair");

        assert_eq!(reconstructed[0].len(), 2);
        assert_eq!(reconstructed[1].len(), 2);
        let original_area = dense_signed_area(&reconstructed[0][1]).abs();
        let synthetic_area = dense_signed_area(&reconstructed[1][1]).abs();
        assert!(synthetic_area > 0.0);
        assert!(synthetic_area < original_area * 0.001);
        assert!(
            dense_signed_area(&reconstructed[1][0]) * dense_signed_area(&reconstructed[1][1]) < 0.0
        );
    }

    #[test]
    fn open_bar_builds_a_bare_body_with_two_safe_nubs() {
        let donor_bar = vec![
            point(45, -30, true),
            point(55, -30, true),
            point(55, 130, true),
            point(45, 130, true),
        ];
        let body = vec![
            point(0, 0, true),
            point(100, 0, true),
            point(100, 100, true),
            point(0, 100, true),
        ];
        let output = reconstruct_open_bar(
            &[vec![donor_bar.clone()], vec![donor_bar]],
            &[vec![body.clone()], vec![body]],
            &[vec![0.0], vec![1.0]],
            0,
            OpenBarOptions {
                nub_overlap: DEFAULT_NUB_OVERLAP,
                min_protrude: DEFAULT_MIN_PROTRUDE,
                anchor: OpenBarAnchor::Left,
            },
        )
        .expect("explicit open-bar repair should build");

        assert!(
            output
                .master_contours
                .iter()
                .all(|contours| contours.len() == 3)
        );
        for contours in output.master_contours {
            let top = &contours[1];
            let bottom = &contours[2];
            assert!(top.iter().any(|point| point.y > 100));
            assert!(bottom.iter().any(|point| point.y < 0));
            assert!(top.iter().all(|point| point.y >= 70));
            assert!(bottom.iter().all(|point| point.y <= 30));
        }
    }

    #[test]
    fn unions_overlapping_pieces_to_restore_shared_topology() {
        let left = dense_polygon(&[(0.0, 0.0), (80.0, 0.0), (80.0, 100.0), (0.0, 100.0)]);
        let right = dense_polygon(&[(40.0, 0.0), (120.0, 0.0), (120.0, 100.0), (40.0, 100.0)]);
        let merged = dense_polygon(&[(0.0, 0.0), (120.0, 0.0), (120.0, 100.0), (0.0, 100.0)]);
        let masters = vec![vec![left, right], vec![merged]];
        let reconciled = contour_topology_variants(&masters, &[vec![0.0], vec![1.0]])
            .into_iter()
            .find(|candidate| candidate.iter().all(|contours| contours.len() == 1))
            .expect("the overlapping pieces should produce a merged topology");

        assert_eq!(reconciled[0].len(), 1);
        assert_eq!(reconciled[1].len(), 1);
        assert!((dense_signed_area(&reconciled[0][0]).abs() - 12_000.0).abs() < 1.0);
    }

    #[test]
    fn splits_a_narrow_join_to_match_separate_master_pieces() {
        let joined = dense_polygon(&[
            (0.0, 0.0),
            (40.0, 0.0),
            (40.0, 18.0),
            (60.0, 18.0),
            (60.0, 0.0),
            (100.0, 0.0),
            (100.0, 40.0),
            (60.0, 40.0),
            (60.0, 22.0),
            (40.0, 22.0),
            (40.0, 40.0),
            (0.0, 40.0),
        ]);
        let left = dense_polygon(&[(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)]);
        let right = dense_polygon(&[(60.0, 0.0), (100.0, 0.0), (100.0, 40.0), (60.0, 40.0)]);

        let split = split_contours_to_match(&[joined], &[left, right]).unwrap();

        assert_eq!(split.len(), 2);
        let areas: Vec<_> = split
            .iter()
            .map(|contour| dense_signed_area(contour).abs())
            .collect();
        assert!(areas.iter().all(|area| *area > 1_500.0));
    }

    #[test]
    fn repeatedly_splits_two_necks_to_match_three_separate_pieces() {
        let joined = dense_polygon(&[
            (0.0, 0.0),
            (30.0, 0.0),
            (30.0, 13.0),
            (50.0, 13.0),
            (50.0, 0.0),
            (80.0, 0.0),
            (80.0, 13.0),
            (100.0, 13.0),
            (100.0, 0.0),
            (130.0, 0.0),
            (130.0, 40.0),
            (100.0, 40.0),
            (100.0, 27.0),
            (80.0, 27.0),
            (80.0, 40.0),
            (50.0, 40.0),
            (50.0, 27.0),
            (30.0, 27.0),
            (30.0, 40.0),
            (0.0, 40.0),
        ]);
        let target = [
            dense_polygon(&[(0.0, 0.0), (30.0, 0.0), (30.0, 40.0), (0.0, 40.0)]),
            dense_polygon(&[(50.0, 0.0), (80.0, 0.0), (80.0, 40.0), (50.0, 40.0)]),
            dense_polygon(&[(100.0, 0.0), (130.0, 0.0), (130.0, 40.0), (100.0, 40.0)]),
        ];

        let split = split_contours_to_match(&[joined], &target).unwrap();

        assert_eq!(split.len(), 3);
        assert!(
            split
                .iter()
                .all(|contour| dense_signed_area(contour).abs() > 1_000.0)
        );
    }

    #[test]
    fn anchors_corners_before_resampling_their_runs() {
        let masters = vec![
            vec![vec![
                point(0, 0, true),
                point(200, 0, true),
                point(200, 200, true),
                point(0, 200, true),
            ]],
            vec![vec![
                point(0, 0, true),
                point(180, 0, true),
                point(240, 200, true),
                point(0, 200, true),
            ]],
        ];
        let normalized = normalize_simple_glyphs(&masters, &[vec![0.0], vec![1.0]], 0).unwrap();
        let first = &normalized.master_contours[0][0];
        let second = &normalized.master_contours[1][0];
        assert_eq!(first.len(), second.len());
        assert!(first.iter().all(|point| point.on_curve));
        assert!(second.iter().all(|point| point.on_curve));
        assert!(first.len() > 4);
    }

    #[test]
    fn detects_a_contour_that_folds_over_itself() {
        let square = [
            FloatPoint { x: 0.0, y: 0.0 },
            FloatPoint { x: 10.0, y: 0.0 },
            FloatPoint { x: 10.0, y: 10.0 },
            FloatPoint { x: 0.0, y: 10.0 },
        ];
        let bow_tie = [
            FloatPoint { x: 0.0, y: 0.0 },
            FloatPoint { x: 10.0, y: 10.0 },
            FloatPoint { x: 0.0, y: 10.0 },
            FloatPoint { x: 10.0, y: 0.0 },
        ];
        assert!(!polygon_self_intersects(&square));
        assert!(polygon_self_intersects(&bow_tie));
    }

    #[test]
    fn ink_gate_detects_two_pieces_collapsing_into_one_mid_axis() {
        let left_piece = dense_polygon(&[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)]);
        let right_piece = dense_polygon(&[(80.0, 0.0), (100.0, 0.0), (100.0, 20.0), (80.0, 20.0)]);
        let left = vec![left_piece.points.clone(), right_piece.points.clone()];
        let swapped = vec![right_piece.points, left_piece.points];

        assert_eq!(
            span_ink_defect_at_resolution(&left, &left, INK_RESOLUTION, INK_BLUR).0,
            0.0
        );
        assert!(span_ink_defect_at_resolution(&left, &swapped, INK_RESOLUTION, INK_BLUR).0 > 0.5);
    }

    #[test]
    fn quality_gate_preserves_all_pixels_past_a_single_machine_word() {
        let donor = vec![vec![
            FloatPoint { x: 0.0, y: 0.0 },
            FloatPoint { x: 100.0, y: 0.0 },
            FloatPoint { x: 100.0, y: 100.0 },
            FloatPoint { x: 0.0, y: 100.0 },
        ]];
        let altered = vec![vec![
            FloatPoint { x: 0.0, y: 0.0 },
            FloatPoint { x: 50.0, y: 0.0 },
            FloatPoint { x: 50.0, y: 100.0 },
            FloatPoint { x: 0.0, y: 100.0 },
        ]];

        assert!(reject_master_ink_deviation(&[donor.clone()], &[donor]).is_ok());
        assert!(
            reject_master_ink_deviation(
                &[altered],
                &[vec![vec![
                    FloatPoint { x: 0.0, y: 0.0 },
                    FloatPoint { x: 100.0, y: 0.0 },
                    FloatPoint { x: 100.0, y: 100.0 },
                    FloatPoint { x: 0.0, y: 100.0 },
                ],]]
            )
            .is_err()
        );
    }
}
