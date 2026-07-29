use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRequest {
    pub axes: Vec<Axis>,
    pub masters: Vec<Master>,
    #[serde(default)]
    pub repairs: Vec<GlyphRepair>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Axis {
    pub tag: String,
    pub name: String,
    pub minimum: f32,
    pub default: f32,
    pub maximum: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Master {
    pub name: String,
    pub location: Vec<f32>,
    pub bytes: Vec<u8>,
}

/// An explicit, per-glyph equivalent of mblode's `glyphs.strategies` config.
/// It is intentionally opt-in: an open bar changes a glyph's design and must
/// never be inferred merely from its contour count.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlyphRepair {
    /// Production glyph name from the donor's `post` table, for example
    /// `dollar` or `cent`.
    pub glyph: String,
    pub strategy: GlyphRepairStrategy,
    /// The bare body donor used by `open_bar`, such as `S` or `c`.
    #[serde(default)]
    pub letter: Option<String>,
    /// `left` or `right`; used as the deterministic fallback anchor.
    #[serde(default)]
    pub anchor: Option<String>,
    #[serde(default)]
    pub nub_overlap: Option<f32>,
    #[serde(default)]
    pub min_protrude: Option<f32>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlyphRepairStrategy {
    Freeze,
    OpenBar,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    #[serde(skip)]
    pub font: Vec<u8>,
    pub default_master: usize,
    pub glyph_count: u16,
    pub axis_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub default_master: usize,
    pub glyph_count: u16,
    pub units_per_em: u16,
    pub axis_count: usize,
    pub master_count: usize,
    pub normalized_glyph_count: usize,
    pub frozen_glyph_count: usize,
}

/// Metadata read directly from a static font's OpenType tables. It is used to
/// suggest a design space in the browser; users can always edit the result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticFontAnalysis {
    /// Typographic family name (name ID 16), falling back to the legacy
    /// family name (name ID 1). Used to suggest a useful download filename.
    pub family: Option<String>,
    /// OS/2.usWeightClass when it is within the OpenType-defined 1..=1000
    /// range. `None` means the font does not provide a usable value.
    pub weight: Option<u16>,
    /// Derived from OS/2.fsSelection and head.macStyle, rather than from a
    /// filename or an outline heuristic.
    pub italic: bool,
}
