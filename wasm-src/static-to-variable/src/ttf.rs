use std::collections::{BTreeMap, BTreeSet};

use read_fonts::{
    FontRef, TableProvider,
    tables::{
        gdef::GlyphClassDef,
        glyf::{CompositeGlyphFlags, Glyph},
        head::MacStyle,
        os2::SelectionFlags,
    },
    types::GlyphId16,
};
use skrifa::{
    FontRef as SkrifaFontRef, GlyphId as SkrifaGlyphId, MetadataProvider, Tag as SkrifaTag,
    instance::Size,
    outline::{DrawSettings, OutlinePen},
    setting::VariationSetting,
};
use unicode_general_category::{GeneralCategory, get_general_category};
use write_fonts::{
    FontBuilder,
    from_obj::{FromTableRef, ToOwnedTable},
    tables::{
        fvar::{AxisInstanceArrays, Fvar, InstanceRecord, VariationAxisRecord},
        gdef::Gdef,
        glyf::{Glyf, GlyfLocaBuilder, Glyph as WriteGlyph},
        gvar::{GlyphDelta, GlyphDeltas, GlyphVariations, Gvar, Tent as GvarTent},
        head::Head,
        hvar::Hvar,
        layout::ClassDef,
        loca::Loca,
        maxp::Maxp,
        name::{Name, NameRecord},
        stat::{AxisRecord, AxisValue, AxisValueTableFlags, Stat},
        variations::{
            DeltaSetIndexMap, RegionAxisCoordinates, VariationRegion,
            ivs_builder::VariationStoreBuilder,
        },
    },
    types::{F2Dot14, Fixed, GlyphId, NameId, Tag},
};

#[path = "compat.rs"]
mod compat;

const MAX_BUILT_FREEZE_PASSES: usize = 40;
const BUILT_MIDPOINT_AREA_TOLERANCE: f64 = 0.22;
const BUILT_MIDPOINT_MIN_AREA: f64 = 800.0;

use crate::{
    error::BuildError,
    model::{
        Axis, BuildRequest, BuildResult, GlyphRepair, GlyphRepairStrategy, Master,
        StaticFontAnalysis, ValidationReport,
    },
    variation_model::{DeltaValue, Support, VariationModel},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    x: i16,
    y: i16,
    on_curve: bool,
}

pub fn analyze_static_font(bytes: &[u8]) -> Result<StaticFontAnalysis, String> {
    let font = FontRef::new(bytes).map_err(|error| error.to_string())?;
    let family = font.name().ok().and_then(|names| {
        let string_data = names.string_data();
        [NameId::TYPOGRAPHIC_FAMILY_NAME, NameId::FAMILY_NAME]
            .into_iter()
            .find_map(|name_id| {
                names
                    .name_record()
                    .iter()
                    .filter(|record| record.name_id() == name_id)
                    .filter_map(|record| {
                        record
                            .string(string_data)
                            .ok()
                            .map(|value| value.to_string())
                    })
                    .find(|value| !value.trim().is_empty())
            })
    });
    let os2 = font.os2().ok();
    let weight = os2
        .as_ref()
        .map(|table| table.us_weight_class())
        .filter(|weight| (1..=1000).contains(weight));
    let italic_from_os2 = os2.is_some_and(|table| {
        let selection = table.fs_selection();
        selection.contains(SelectionFlags::ITALIC) || selection.contains(SelectionFlags::OBLIQUE)
    });
    let italic_from_head = font
        .head()
        .ok()
        .is_some_and(|table| table.mac_style().contains(MacStyle::ITALIC));

    Ok(StaticFontAnalysis {
        family,
        weight,
        italic: italic_from_os2 || italic_from_head,
    })
}

impl Point {
    const ZERO: Self = Self {
        x: 0,
        y: 0,
        on_curve: true,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct VariationPoint {
    x: f64,
    y: f64,
}

impl VariationPoint {
    const ZERO: Self = Self { x: 0.0, y: 0.0 };

    fn is_zero(self) -> bool {
        self.x.abs() < 1e-9 && self.y.abs() < 1e-9
    }
}

impl DeltaValue for VariationPoint {
    fn subtract_scaled(&self, other: Self, scalar: f64) -> Self {
        Self {
            x: self.x - other.x * scalar,
            y: self.y - other.y * scalar,
        }
    }
}

#[derive(Clone, Copy)]
struct MetricValue(f64);

impl DeltaValue for MetricValue {
    fn subtract_scaled(&self, other: Self, scalar: f64) -> Self {
        Self(self.0 - other.0 * scalar)
    }
}

#[derive(Clone, Debug)]
enum ParsedGlyph {
    Empty,
    Simple {
        contours: Vec<Vec<Point>>,
    },
    Composite {
        components: Vec<read_fonts::tables::glyf::Component>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GlyphMetrics {
    x_min: i16,
    advance: u16,
    side_bearing: i16,
}

#[derive(Clone, Debug)]
struct ParsedMaster {
    glyph_count: u16,
    units_per_em: u16,
    /// Glyph identities from the donor's `post` table. mblode's bootstrap
    /// stage establishes the default donor's glyph order, then resolves every
    /// other donor by name rather than assuming matching glyph IDs.
    glyph_names: Vec<Option<String>>,
    /// Unicode selectors from the best cmap subtable. This is the fallback
    /// for post format 3 fonts, which omit production glyph names entirely.
    cmap: BTreeMap<u32, u16>,
    glyphs: Vec<ParsedGlyph>,
    metrics: Vec<GlyphMetrics>,
}

struct Prepared<'a> {
    axis_tags: Vec<Tag>,
    locations: Vec<Vec<f64>>,
    default_master: usize,
    masters: Vec<ParsedMaster>,
    normalized_simple_glyphs: BTreeMap<u16, compat::NormalizedSimpleGlyphs>,
    frozen_glyphs: BTreeSet<u16>,
    direct_metric_composites: BTreeSet<u16>,
    request: &'a BuildRequest,
}

struct RewrittenGlyfTables {
    glyf: Glyf,
    loca: Loca,
    head: Head,
    maxp: Maxp,
}

pub fn validate_request(request: &BuildRequest) -> Result<ValidationReport, BuildError> {
    let prepared = prepare(request)?;
    let default = &prepared.masters[prepared.default_master];
    validate_all_glyphs(&prepared)?;
    Ok(ValidationReport {
        default_master: prepared.default_master,
        glyph_count: default.glyph_count,
        units_per_em: default.units_per_em,
        axis_count: prepared.axis_tags.len(),
        master_count: prepared.masters.len(),
        normalized_glyph_count: prepared.normalized_simple_glyphs.len(),
        frozen_glyph_count: prepared.frozen_glyphs.len(),
    })
}

pub fn build_variable_font(request: &BuildRequest) -> Result<BuildResult, BuildError> {
    let mut prepared = prepare(request)?;
    for _ in 0..MAX_BUILT_FREEZE_PASSES {
        let font = build_font_binary(&prepared)?;
        let collapsed = built_font_collapsing_glyphs(&font, &prepared)?;
        let fresh: Vec<_> = collapsed
            .difference(&prepared.frozen_glyphs)
            .copied()
            .collect();
        if fresh.is_empty() {
            let default = &prepared.masters[prepared.default_master];
            return Ok(BuildResult {
                font,
                default_master: prepared.default_master,
                glyph_count: default.glyph_count,
                axis_count: prepared.axis_tags.len(),
            });
        }
        for glyph in fresh {
            // mblode's post-build freeze loop restores the default donor, not
            // the intermediate repaired outline. Removing the replacement
            // glyf record as well as its gvar data gives that same result.
            prepared.frozen_glyphs.insert(glyph);
            prepared.normalized_simple_glyphs.remove(&glyph);
            prepared.direct_metric_composites.remove(&glyph);
        }
    }
    Err(BuildError::FreezeLoopDidNotConverge)
}

fn build_font_binary(prepared: &Prepared<'_>) -> Result<Vec<u8>, BuildError> {
    let request = prepared.request;
    let model = VariationModel::new(prepared.locations.clone())?;
    let glyph_variations = build_gvar_variations(prepared, &model)?;

    let default_font =
        FontRef::new(&request.masters[prepared.default_master].bytes).map_err(|error| {
            BuildError::InvalidTrueType {
                master: request.masters[prepared.default_master].name.clone(),
                reason: error.to_string(),
            }
        })?;
    let mut names: Name = default_font
        .name()
        .map_err(|error| invalid_font(&request.masters[prepared.default_master], error))?
        .to_owned_table();
    let (axis_name_ids, instance_name_ids) =
        append_variation_names(&mut names, &request.axes, &request.masters)?;
    let fvar = build_fvar(
        &request.axes,
        &prepared.axis_tags,
        &axis_name_ids,
        &request.masters,
        &instance_name_ids,
    );
    let stat = build_stat(
        &prepared.axis_tags,
        &axis_name_ids,
        &request.axes,
        &request.masters,
        &instance_name_ids,
    );
    let hvar = build_hvar(prepared, &model)?;
    let generated_gdef = default_font
        .data_for_tag(Tag::new(b"GDEF"))
        .is_none()
        .then(|| build_gdef(&prepared.masters[prepared.default_master]));
    let gvar = Gvar::new(glyph_variations, prepared.axis_tags.len() as u16).map_err(|error| {
        BuildError::WriteTable {
            table: "gvar",
            reason: error.to_string(),
        }
    })?;
    let rewritten_glyf = (!prepared.normalized_simple_glyphs.is_empty()
        || !prepared.direct_metric_composites.is_empty())
    .then(|| rebuild_glyf_tables(&default_font, prepared))
    .transpose()?;

    let mut builder = FontBuilder::new();
    builder
        .add_table(&names)
        .map_err(|error| BuildError::WriteTable {
            table: "name",
            reason: error.to_string(),
        })?;
    builder
        .add_table(&fvar)
        .map_err(|error| BuildError::WriteTable {
            table: "fvar",
            reason: error.to_string(),
        })?;
    builder
        .add_table(&stat)
        .map_err(|error| BuildError::WriteTable {
            table: "STAT",
            reason: error.to_string(),
        })?;
    builder
        .add_table(&hvar)
        .map_err(|error| BuildError::WriteTable {
            table: "HVAR",
            reason: error.to_string(),
        })?;
    if let Some(gdef) = generated_gdef {
        builder
            .add_table(&gdef)
            .map_err(|error| BuildError::WriteTable {
                table: "GDEF",
                reason: error.to_string(),
            })?;
    }
    builder
        .add_table(&gvar)
        .map_err(|error| BuildError::WriteTable {
            table: "gvar",
            reason: error.to_string(),
        })?;
    if let Some(tables) = &rewritten_glyf {
        builder
            .add_table(&tables.glyf)
            .map_err(|error| BuildError::WriteTable {
                table: "glyf",
                reason: error.to_string(),
            })?;
        builder
            .add_table(&tables.loca)
            .map_err(|error| BuildError::WriteTable {
                table: "loca",
                reason: error.to_string(),
            })?;
        builder
            .add_table(&tables.head)
            .map_err(|error| BuildError::WriteTable {
                table: "head",
                reason: error.to_string(),
            })?;
        builder
            .add_table(&tables.maxp)
            .map_err(|error| BuildError::WriteTable {
                table: "maxp",
                reason: error.to_string(),
            })?;
    }
    for record in default_font.table_directory().table_records() {
        let tag = record.tag();
        // A DSIG authenticates the original binary. Classic TrueType hinting
        // programs are tied to the old static outlines; mblode's Glyphs →
        // Fontmake build drops them for the same reason. Retaining either
        // after replacing glyf/gvar would make an apparently valid VF render
        // unpredictably in hinting engines.
        if [b"DSIG", b"cvt ", b"fpgm", b"prep", b"gasp"]
            .into_iter()
            .map(Tag::new)
            .any(|excluded| tag == excluded)
            || builder.contains(tag)
        {
            continue;
        }
        if let Some(data) = default_font.data_for_tag(tag) {
            builder.add_raw(tag, data);
        }
    }
    let font = builder.build();
    validate_generated_gvar(&font, &request.masters[prepared.default_master])?;
    Ok(font)
}

/// Reject a serialized gvar record that a second OpenType implementation would
/// decode differently. Required deltas have no point-number list, so they mean
/// “every point”; composites therefore must include their component points and
/// all four phantom points. This caught the class of output that native
/// rendering accepted but FontTools correctly rejected as truncated.
fn validate_generated_gvar(bytes: &[u8], master: &Master) -> Result<(), BuildError> {
    let font = FontRef::new(bytes).map_err(|error| invalid_font(master, error))?;
    let gvar = font.gvar().map_err(|error| invalid_font(master, error))?;
    let glyf = font.glyf().map_err(|error| invalid_font(master, error))?;
    let loca = font
        .loca(None)
        .map_err(|error| invalid_font(master, error))?;
    for glyph_id in 0..font
        .maxp()
        .map_err(|error| invalid_font(master, error))?
        .num_glyphs()
    {
        let glyph = GlyphId::new(glyph_id as u32);
        let Some(variation_data) =
            gvar.glyph_variation_data(glyph)
                .map_err(|error| BuildError::WriteTable {
                    table: "gvar",
                    reason: format!("glyph {glyph_id} has malformed variation data: {error}"),
                })?
        else {
            continue;
        };
        let expected_points = match loca
            .get_glyf(glyph, &glyf)
            .map_err(|error| invalid_font(master, error))?
        {
            None => 4,
            Some(Glyph::Simple(simple)) => simple.num_points() + 4,
            Some(Glyph::Composite(composite)) => composite.components().count() + 4,
        };
        for tuple in variation_data.tuples() {
            if !tuple.has_deltas_for_all_points() {
                continue;
            }
            let actual_points = tuple.deltas().count();
            if actual_points != expected_points {
                return Err(BuildError::WriteTable {
                    table: "gvar",
                    reason: format!(
                        "glyph {glyph_id} encodes {actual_points} required deltas for {expected_points} points"
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Execute mblode's final safety step against the *serialized* variable font.
///
/// A structurally valid `gvar` can still collapse after tuple rounding or an
/// interaction between composite components. Rendering the finished font at
/// every adjacent master span and its midpoint catches that class of problem
/// before it reaches the preview or download.
fn built_font_collapsing_glyphs(
    bytes: &[u8],
    prepared: &Prepared<'_>,
) -> Result<BTreeSet<u16>, BuildError> {
    let font = SkrifaFontRef::new(bytes).map_err(|error| BuildError::WriteTable {
        table: "glyf",
        reason: format!("could not inspect the generated variable font: {error}"),
    })?;
    let spans = master_span_pairs(&prepared.locations);
    if spans.is_empty() {
        return Ok(BTreeSet::new());
    }

    let mut collapsed = BTreeSet::new();
    for glyph in 1..prepared.masters[prepared.default_master].glyph_count {
        if prepared.frozen_glyphs.contains(&glyph) {
            continue;
        }
        for (left, right) in &spans {
            let left_location = &prepared.request.masters[*left].location;
            let right_location = &prepared.request.masters[*right].location;
            let midpoint: Vec<_> = left_location
                .iter()
                .zip(right_location)
                .map(|(left, right)| (left + right) / 2.0)
                .collect();
            let Some(left_area) =
                rendered_glyph_area(&font, glyph, &prepared.request.axes, left_location)
            else {
                continue;
            };
            let Some(right_area) =
                rendered_glyph_area(&font, glyph, &prepared.request.axes, right_location)
            else {
                continue;
            };
            let Some(midpoint_area) =
                rendered_glyph_area(&font, glyph, &prepared.request.axes, &midpoint)
            else {
                continue;
            };
            if left_area <= 0.0 || right_area <= 0.0 {
                continue;
            }
            let endpoint_mean = (left_area + right_area) / 2.0;
            if endpoint_mean > BUILT_MIDPOINT_MIN_AREA
                && (midpoint_area / endpoint_mean - 1.0).abs() > BUILT_MIDPOINT_AREA_TOLERANCE
            {
                collapsed.insert(glyph);
                break;
            }
        }
    }
    Ok(collapsed)
}

fn master_span_pairs(locations: &[Vec<f64>]) -> Vec<(usize, usize)> {
    let Some(axis_count) = locations.first().map(Vec::len) else {
        return Vec::new();
    };
    if axis_count == 0
        || locations
            .iter()
            .any(|location| location.len() != axis_count)
    {
        return Vec::new();
    }
    let mut pairs = Vec::new();
    for axis in 0..axis_count {
        for seed in 0..locations.len() {
            let mut group: Vec<_> = (0..locations.len())
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
            for pair in group.windows(2) {
                let pair = (pair[0].min(pair[1]), pair[0].max(pair[1]));
                if !pairs.contains(&pair) {
                    pairs.push(pair);
                }
            }
        }
    }
    pairs
}

fn rendered_glyph_area(
    font: &SkrifaFontRef<'_>,
    glyph: u16,
    axes: &[Axis],
    location: &[f32],
) -> Option<f64> {
    if axes.len() != location.len() {
        return None;
    }
    let settings: Vec<_> = axes
        .iter()
        .zip(location)
        .map(|(axis, value)| {
            let tag: [u8; 4] = axis.tag.as_bytes().try_into().expect("validated axis tag");
            VariationSetting::new(SkrifaTag::new(&tag), *value)
        })
        .collect();
    let location = font.axes().location(settings);
    let outline = font
        .outline_glyphs()
        .get(SkrifaGlyphId::new(u32::from(glyph)))?;
    let mut pen = GlyphAreaPen::default();
    outline
        .draw(
            DrawSettings::unhinted(Size::unscaled(), &location),
            &mut pen,
        )
        .ok()?;
    // TrueType outlines normally close every contour, but retain a final
    // drawable segment if a malformed outline omits its close command. This
    // makes the native freeze check conservative instead of treating it as
    // zero-area.
    pen.finish_contour();
    pen.area()
}

#[derive(Clone, Copy)]
struct OutlinePoint {
    x: f64,
    y: f64,
}

#[derive(Default)]
struct GlyphAreaPen {
    contours: Vec<Vec<OutlinePoint>>,
    current: Vec<OutlinePoint>,
}

impl GlyphAreaPen {
    fn point(x: f32, y: f32) -> OutlinePoint {
        OutlinePoint {
            x: f64::from(x),
            y: f64::from(y),
        }
    }

    fn current_point(&self) -> Option<OutlinePoint> {
        self.current.last().copied()
    }

    fn finish_contour(&mut self) {
        if self.current.len() >= 3 {
            self.contours.push(std::mem::take(&mut self.current));
        } else {
            self.current.clear();
        }
    }

    fn area(&self) -> Option<f64> {
        let signed_area: f64 = self
            .contours
            .iter()
            .map(|contour| {
                contour
                    .iter()
                    .enumerate()
                    .map(|(index, point)| {
                        let next = contour[(index + 1) % contour.len()];
                        point.x * next.y - next.x * point.y
                    })
                    .sum::<f64>()
                    / 2.0
            })
            .sum();
        signed_area.is_finite().then_some(signed_area.abs())
    }
}

impl OutlinePen for GlyphAreaPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish_contour();
        self.current.push(Self::point(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current.push(Self::point(x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let Some(start) = self.current_point() else {
            self.move_to(x, y);
            return;
        };
        let control = Self::point(cx0, cy0);
        let end = Self::point(x, y);
        for step in 1..=16 {
            let t = step as f64 / 16.0;
            let inverse = 1.0 - t;
            self.current.push(OutlinePoint {
                x: inverse * inverse * start.x + 2.0 * inverse * t * control.x + t * t * end.x,
                y: inverse * inverse * start.y + 2.0 * inverse * t * control.y + t * t * end.y,
            });
        }
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let Some(start) = self.current_point() else {
            self.move_to(x, y);
            return;
        };
        let first_control = Self::point(cx0, cy0);
        let second_control = Self::point(cx1, cy1);
        let end = Self::point(x, y);
        for step in 1..=24 {
            let t = step as f64 / 24.0;
            let inverse = 1.0 - t;
            self.current.push(OutlinePoint {
                x: inverse.powi(3) * start.x
                    + 3.0 * inverse * inverse * t * first_control.x
                    + 3.0 * inverse * t * t * second_control.x
                    + t.powi(3) * end.x,
                y: inverse.powi(3) * start.y
                    + 3.0 * inverse * inverse * t * first_control.y
                    + 3.0 * inverse * t * t * second_control.y
                    + t.powi(3) * end.y,
            });
        }
    }

    fn close(&mut self) {
        self.finish_contour();
    }
}

fn prepare(request: &BuildRequest) -> Result<Prepared<'_>, BuildError> {
    if request.masters.len() < 2 {
        return Err(BuildError::TooFewMasters);
    }
    if request.axes.is_empty() {
        return Err(BuildError::NoAxes);
    }

    let axis_tags: Vec<Tag> = request
        .axes
        .iter()
        .map(validate_axis)
        .collect::<Result<_, _>>()?;
    for (index, tag) in axis_tags.iter().enumerate() {
        if axis_tags[..index].contains(tag) {
            return Err(BuildError::DuplicateAxisTag {
                axis: request.axes[index].tag.clone(),
            });
        }
    }
    let locations = normalized_locations(request)?;
    let default_indices: Vec<_> = locations
        .iter()
        .enumerate()
        .filter_map(|(index, location)| {
            location
                .iter()
                .all(|coordinate| coordinate.abs() < 1e-9)
                .then_some(index)
        })
        .collect();
    if default_indices.len() != 1 {
        return Err(BuildError::InvalidDefaultMasterCount {
            count: default_indices.len(),
        });
    }

    let mut masters: Vec<_> = request
        .masters
        .iter()
        .map(parse_master)
        .collect::<Result<_, _>>()?;
    let default_master = default_indices[0];
    let default = masters[default_master].clone();
    for (index, master) in masters.iter().enumerate() {
        if index == default_master {
            continue;
        }
        if master.units_per_em != default.units_per_em {
            return Err(BuildError::UnitsPerEmMismatch {
                master: request.masters[index].name.clone(),
                expected: default.units_per_em,
                actual: master.units_per_em,
            });
        }
    }
    for (index, master) in masters.iter_mut().enumerate() {
        if index == default_master {
            continue;
        }
        *master =
            align_to_default_glyph_order(&default, master.clone(), &request.masters[index].name)?;
    }
    let repair_index = glyph_repair_index(&default, &request.repairs)?;
    let normalized =
        normalize_simple_glyphs(&masters, request, &locations, default_master, &repair_index)?;
    let direct_metric_composites = direct_metric_composites(&masters, default_master)
        .difference(&normalized.frozen)
        .copied()
        .collect();

    Ok(Prepared {
        axis_tags,
        locations,
        default_master,
        masters,
        normalized_simple_glyphs: normalized.glyphs,
        frozen_glyphs: normalized.frozen,
        direct_metric_composites,
        request,
    })
}

fn glyph_repair_index(
    default: &ParsedMaster,
    repairs: &[GlyphRepair],
) -> Result<BTreeMap<u16, usize>, BuildError> {
    if repairs.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut result = BTreeMap::new();
    for (repair_index, repair) in repairs.iter().enumerate() {
        let glyph = glyph_id_for_selector(default, repair.glyph.trim()).ok_or_else(|| {
            BuildError::InvalidGlyphRepair {
                glyph: repair.glyph.clone(),
                reason: "that glyph name or Unicode selector is not present in the default donor"
                    .into(),
            }
        })?;
        if result.insert(glyph, repair_index).is_some() {
            return Err(BuildError::InvalidGlyphRepair {
                glyph: repair.glyph.clone(),
                reason: "a glyph can have only one repair rule".into(),
            });
        }
        if repair.strategy == GlyphRepairStrategy::OpenBar {
            let letter = repair.letter.as_deref().map(str::trim).unwrap_or_default();
            if letter.is_empty() || glyph_id_for_selector(default, letter).is_none() {
                return Err(BuildError::InvalidGlyphRepair {
                    glyph: repair.glyph.clone(),
                    reason: "open_bar needs a bare-letter glyph in the default donor".into(),
                });
            }
            if !matches!(repair.anchor.as_deref(), Some("left" | "right")) {
                return Err(BuildError::InvalidGlyphRepair {
                    glyph: repair.glyph.clone(),
                    reason: "open_bar anchor must be left or right".into(),
                });
            }
            if repair
                .nub_overlap
                .is_some_and(|value| !value.is_finite() || value < 0.0)
                || repair
                    .min_protrude
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
            {
                return Err(BuildError::InvalidGlyphRepair {
                    glyph: repair.glyph.clone(),
                    reason: "open_bar dimensions must be finite positive values".into(),
                });
            }
        }
    }
    Ok(result)
}

fn validate_axis(axis: &Axis) -> Result<Tag, BuildError> {
    if axis.tag.len() != 4 {
        return Err(BuildError::InvalidAxisTag {
            axis: axis.tag.clone(),
        });
    }
    Tag::new_checked(axis.tag.as_bytes()).map_err(|_| BuildError::InvalidAxisTag {
        axis: axis.tag.clone(),
    })?;
    if axis.minimum > axis.default || axis.default > axis.maximum || axis.minimum == axis.maximum {
        return Err(BuildError::InvalidAxisRange {
            axis: axis.tag.clone(),
        });
    }
    Ok(Tag::new_checked(axis.tag.as_bytes()).expect("validated tag"))
}

fn normalized_locations(request: &BuildRequest) -> Result<Vec<Vec<f64>>, BuildError> {
    request
        .masters
        .iter()
        .map(|master| {
            if master.location.len() != request.axes.len() {
                return Err(BuildError::WrongLocationLength {
                    master: master.name.clone(),
                    expected: request.axes.len(),
                    actual: master.location.len(),
                });
            }
            master
                .location
                .iter()
                .zip(&request.axes)
                .map(|(location, axis)| {
                    if *location < axis.minimum || *location > axis.maximum {
                        return Err(BuildError::LocationOutsideAxisRange {
                            master: master.name.clone(),
                            axis: axis.tag.clone(),
                        });
                    }
                    let normalized = if *location < axis.default {
                        (*location - axis.default) / (axis.default - axis.minimum)
                    } else if *location > axis.default {
                        (*location - axis.default) / (axis.maximum - axis.default)
                    } else {
                        0.0
                    };
                    Ok(normalized as f64)
                })
                .collect()
        })
        .collect()
}

fn parse_master(master: &Master) -> Result<ParsedMaster, BuildError> {
    let font = FontRef::new(&master.bytes).map_err(|error| invalid_font(master, error))?;
    for tag in [
        b"fvar", b"gvar", b"avar", b"HVAR", b"VVAR", b"MVAR", b"cvar", b"STAT", b"CFF2",
    ] {
        let tag = Tag::new(tag);
        if font.data_for_tag(tag).is_some() {
            return Err(BuildError::VariableInput {
                master: master.name.clone(),
                table: tag.to_string(),
            });
        }
    }

    let head = font.head().map_err(|error| invalid_font(master, error))?;
    let glyph_count = font
        .maxp()
        .map_err(|error| invalid_font(master, error))?
        .num_glyphs();
    let glyf = font.glyf().map_err(|error| invalid_font(master, error))?;
    let loca = font
        .loca(None)
        .map_err(|error| invalid_font(master, error))?;
    let hmtx = font.hmtx().map_err(|error| invalid_font(master, error))?;
    let post = font.post().ok();
    let cmap = font
        .cmap()
        .ok()
        .and_then(|table| table.best_subtable().map(|(_, _, subtable)| subtable))
        .map(|subtable| {
            subtable
                .iter()
                .filter_map(|(codepoint, glyph)| {
                    let glyph = glyph.to_u32();
                    (glyph < u32::from(glyph_count)).then_some((codepoint, glyph as u16))
                })
                .collect()
        })
        .unwrap_or_default();
    if loca.len() != glyph_count as usize {
        return Err(BuildError::InvalidTrueType {
            master: master.name.clone(),
            reason: format!(
                "loca has {} glyph locations but maxp declares {glyph_count}",
                loca.len()
            ),
        });
    }

    let mut glyphs = Vec::with_capacity(glyph_count as usize);
    let mut metrics = Vec::with_capacity(glyph_count as usize);
    let mut glyph_names = Vec::with_capacity(glyph_count as usize);
    for glyph_id in 0..glyph_count {
        let glyph = loca
            .get_glyf(GlyphId::new(glyph_id as u32), &glyf)
            .map_err(|error| invalid_font(master, error))?;
        let (parsed, x_min) = match glyph {
            None => (ParsedGlyph::Empty, 0),
            Some(Glyph::Simple(glyph)) => {
                let expected_points = glyph.num_points();
                let mut points = glyph.points();
                let mut start = 0;
                let mut contours = Vec::new();
                for end_point in glyph.end_pts_of_contours() {
                    let end = end_point.get() as usize + 1;
                    let contour = points
                        .by_ref()
                        .take(end.saturating_sub(start))
                        .map(|point| Point {
                            x: point.x,
                            y: point.y,
                            on_curve: point.on_curve,
                        })
                        .collect();
                    contours.push(contour);
                    start = end;
                }
                if contours.iter().map(Vec::len).sum::<usize>() != expected_points {
                    return Err(BuildError::InvalidTrueType {
                        master: master.name.clone(),
                        reason: format!("could not read every point in simple glyph {glyph_id}"),
                    });
                }
                (ParsedGlyph::Simple { contours }, glyph.x_min())
            }
            Some(Glyph::Composite(glyph)) => {
                let components: Vec<_> = glyph.components().collect();
                (ParsedGlyph::Composite { components }, glyph.x_min())
            }
        };
        let identifier = GlyphId::new(glyph_id as u32);
        glyph_names.push(
            post.as_ref()
                .and_then(|table| table.glyph_name(GlyphId16::new(glyph_id)))
                .map(str::to_owned),
        );
        let advance = hmtx
            .advance(identifier)
            .ok_or_else(|| BuildError::InvalidTrueType {
                master: master.name.clone(),
                reason: format!("hmtx has no advance width for glyph {glyph_id}"),
            })?;
        let side_bearing =
            hmtx.side_bearing(identifier)
                .ok_or_else(|| BuildError::InvalidTrueType {
                    master: master.name.clone(),
                    reason: format!("hmtx has no left side bearing for glyph {glyph_id}"),
                })?;
        glyphs.push(parsed);
        metrics.push(GlyphMetrics {
            x_min,
            advance,
            side_bearing,
        });
    }

    Ok(ParsedMaster {
        glyph_count,
        units_per_em: head.units_per_em(),
        glyph_names,
        cmap,
        glyphs,
        metrics,
    })
}

/// mblode bootstraps its source from the default donor, making that donor's
/// glyph names and order the authoritative set. Static releases often have a
/// different glyph order or carry extra private glyphs, so align named donors
/// before outline reconstruction rather than failing on an unrelated count.
fn align_to_default_glyph_order(
    default: &ParsedMaster,
    target: ParsedMaster,
    master_name: &str,
) -> Result<ParsedMaster, BuildError> {
    let named_default: Option<Vec<_>> = default.glyph_names.iter().cloned().collect();
    let named_target: Option<Vec<_>> = target.glyph_names.iter().cloned().collect();
    let Some(default_names) = named_default else {
        return same_order_or_count_error(default, target, master_name);
    };
    let Some(target_names) = named_target else {
        return same_order_or_count_error(default, target, master_name);
    };

    let mut target_by_name = BTreeMap::new();
    for (glyph_id, name) in target_names.iter().enumerate() {
        if target_by_name.insert(name.as_str(), glyph_id).is_some() {
            return same_order_or_count_error(default, target, master_name);
        }
    }

    let mut glyphs = Vec::with_capacity(default.glyphs.len());
    let mut metrics = Vec::with_capacity(default.metrics.len());
    for (glyph_id, name) in default_names.iter().enumerate() {
        if let Some(&target_id) = target_by_name.get(name.as_str()) {
            glyphs.push(target.glyphs[target_id].clone());
            metrics.push(target.metrics[target_id].clone());
        } else {
            // The default donor defines the source glyph set. A missing glyph
            // is deliberately represented as empty here; mblode's rebuild
            // path later freezes it to the default donor instead of inventing
            // a distorted outline.
            glyphs.push(ParsedGlyph::Empty);
            metrics.push(default.metrics[glyph_id].clone());
        }
    }

    Ok(ParsedMaster {
        glyph_count: default.glyph_count,
        units_per_em: target.units_per_em,
        glyph_names: default.glyph_names.clone(),
        cmap: default.cmap.clone(),
        glyphs,
        metrics,
    })
}

fn same_order_or_count_error(
    default: &ParsedMaster,
    target: ParsedMaster,
    master_name: &str,
) -> Result<ParsedMaster, BuildError> {
    if target.glyph_count == default.glyph_count {
        Ok(target)
    } else {
        Err(BuildError::GlyphCountMismatch {
            master: master_name.to_owned(),
            expected: default.glyph_count,
            actual: target.glyph_count,
        })
    }
}

fn invalid_font(master: &Master, error: impl std::fmt::Display) -> BuildError {
    BuildError::InvalidTrueType {
        master: master.name.clone(),
        reason: error.to_string(),
    }
}

struct GlyphNormalization {
    glyphs: BTreeMap<u16, compat::NormalizedSimpleGlyphs>,
    frozen: BTreeSet<u16>,
}

fn normalize_simple_glyphs(
    masters: &[ParsedMaster],
    request: &BuildRequest,
    locations: &[Vec<f64>],
    default_master: usize,
    repair_index: &BTreeMap<u16, usize>,
) -> Result<GlyphNormalization, BuildError> {
    let Some(default) = masters.get(default_master) else {
        return Ok(GlyphNormalization {
            glyphs: BTreeMap::new(),
            frozen: BTreeSet::new(),
        });
    };
    let mut normalized = BTreeMap::new();
    let mut frozen = BTreeSet::new();
    for glyph in 0..default.glyph_count {
        if repair_index
            .get(&glyph)
            .is_some_and(|index| request.repairs[*index].strategy == GlyphRepairStrategy::Freeze)
        {
            frozen.insert(glyph);
            continue;
        }
        let all_composites = masters.iter().all(|master| {
            matches!(
                master.glyphs.get(glyph as usize),
                Some(ParsedGlyph::Composite { .. })
            )
        });
        let direct_composite_variation = all_composites
            && masters.iter().enumerate().all(|(master_index, master)| {
                master_index == default_master
                    || matches!(
                        (
                            default.glyphs.get(glyph as usize),
                            master.glyphs.get(glyph as usize),
                        ),
                        (
                            Some(ParsedGlyph::Composite { components: base }),
                            Some(ParsedGlyph::Composite { components: target }),
                        ) if align_composite_components(base, target).is_ok()
                    )
            });
        if direct_composite_variation {
            // Retain matching composites so their component-offset gvar data
            // stays compact. Mixed simple/composite releases are flattened
            // below because one glyf record must have a single glyph kind.
            continue;
        }
        let mut outlines: Vec<Vec<Vec<Point>>> = Vec::with_capacity(masters.len());
        let mut reference_index: Option<usize> = None;
        let mut needs_normalization = false;
        for (master_index, master) in masters.iter().enumerate() {
            let (mut outline, was_simple) = match master.glyphs.get(glyph as usize) {
                Some(ParsedGlyph::Simple { contours }) => (contours.clone(), true),
                Some(ParsedGlyph::Empty) => {
                    needs_normalization = true;
                    (Vec::new(), false)
                }
                Some(ParsedGlyph::Composite { .. }) => {
                    needs_normalization = true;
                    let outline =
                        flatten_composite_glyph(&master.glyphs, glyph, Affine::IDENTITY, 0)
                            .map_err(|reason| BuildError::IncompatibleGlyph {
                                glyph,
                                master: request.masters[master_index].name.clone(),
                                reason,
                            })?;
                    (outline, false)
                }
                None => continue,
            };
            if outline.iter().any(|contour| !is_drawable_contour(contour)) {
                // Some valid production TTFs carry zero-area contour records
                // as no-ink markers. They cannot contribute to a TrueType
                // interpolation topology, so drop them before matching or
                // rebuilding the outline.
                outline.retain(|contour| is_drawable_contour(contour));
                needs_normalization = true;
            }
            if was_simple {
                if let Some(reference) = reference_index {
                    if align_simple_contours(outlines[reference].as_slice(), &outline).is_err() {
                        needs_normalization = true;
                    }
                } else {
                    reference_index = Some(outlines.len());
                }
            }
            if reference_index.is_none() && !outline.is_empty() {
                reference_index = Some(outlines.len());
            }
            outlines.push(outline);
        }
        if let Some(repair_index) = repair_index.get(&glyph)
            && request.repairs[*repair_index].strategy == GlyphRepairStrategy::OpenBar
            && let Some(letter_glyph) = glyph_id_named(
                default,
                request.repairs[*repair_index]
                    .letter
                    .as_deref()
                    .expect("validated open_bar letter"),
            )
            && let Some(letter_outlines) =
                drawable_outlines_for_glyph(masters, letter_glyph, request)?
        {
            let repair = &request.repairs[*repair_index];
            let options = compat::OpenBarOptions {
                nub_overlap: f64::from(
                    repair
                        .nub_overlap
                        .unwrap_or(compat::DEFAULT_NUB_OVERLAP as f32),
                ),
                min_protrude: f64::from(
                    repair
                        .min_protrude
                        .unwrap_or(compat::DEFAULT_MIN_PROTRUDE as f32),
                ),
                anchor: match repair.anchor.as_deref().expect("validated open_bar anchor") {
                    "left" => compat::OpenBarAnchor::Left,
                    "right" => compat::OpenBarAnchor::Right,
                    _ => unreachable!("validated open_bar anchor"),
                },
            };
            let open_bar = compat::reconstruct_open_bar(
                &outlines,
                &letter_outlines,
                locations,
                default_master,
                options,
            )
            .map_err(|reason| BuildError::InvalidGlyphRepair {
                glyph: repair.glyph.clone(),
                reason: format!("open_bar could not build a safe repair: {reason}"),
            })?;
            normalized.insert(glyph, open_bar);
            continue;
        }
        if outlines.len() != masters.len() || !needs_normalization {
            continue;
        }
        if reference_index.is_none() || outlines.iter().all(Vec::is_empty) {
            // The source glyphs contained only zero-length contours. Rebuild
            // them as a genuinely empty glyph so gvar carries only their
            // phantom-point metric deltas.
            normalized.insert(
                glyph,
                compat::NormalizedSimpleGlyphs {
                    master_contours: vec![Vec::new(); masters.len()],
                    default_glyph: write_fonts::tables::glyf::SimpleGlyph::default(),
                },
            );
            continue;
        }
        if outlines.iter().any(Vec::is_empty) {
            // Never invent a collapsed outline for a master that does not
            // contain this glyph. It can make an apparently valid gvar entry
            // that tears or explodes between weights. mblode's freeze loop
            // keeps the default donor instead, which is a clean and valid
            // variable-font fallback.
            frozen.insert(glyph);
            continue;
        }
        match compat::normalize_simple_glyphs(&outlines, locations, default_master) {
            Ok(glyphs) => {
                normalized.insert(
                    glyph,
                    compat::NormalizedSimpleGlyphs {
                        master_contours: glyphs.master_contours,
                        default_glyph: glyphs.default_glyph,
                    },
                );
            }
            Err(_) => {
                // This is the native equivalent of mblode's final freeze
                // fallback: every attempted topology repair and its ink gates
                // have failed, so preserve the default donor rather than ship
                // a distorted interpolation.
                frozen.insert(glyph);
            }
        }
    }
    Ok(GlyphNormalization {
        glyphs: normalized,
        frozen,
    })
}

fn glyph_id_named(default: &ParsedMaster, name: &str) -> Option<u16> {
    glyph_id_for_selector(default, name)
}

fn glyph_id_for_selector(default: &ParsedMaster, selector: &str) -> Option<u16> {
    default
        .glyph_names
        .iter()
        .position(|candidate| candidate.as_deref() == Some(selector))
        .and_then(|index| u16::try_from(index).ok())
        .or_else(|| {
            unicode_selector(selector).and_then(|codepoint| default.cmap.get(&codepoint).copied())
        })
}

fn unicode_selector(selector: &str) -> Option<u32> {
    let selector = selector.trim();
    if let Some(hex) = selector
        .strip_prefix("U+")
        .or_else(|| selector.strip_prefix("u+"))
    {
        return u32::from_str_radix(hex, 16)
            .ok()
            .filter(|codepoint| char::from_u32(*codepoint).is_some());
    }
    let mut characters = selector.chars();
    let character = characters.next()?;
    characters.next().is_none().then_some(u32::from(character))
}

fn drawable_outlines_for_glyph(
    masters: &[ParsedMaster],
    glyph: u16,
    request: &BuildRequest,
) -> Result<Option<Vec<Vec<Vec<Point>>>>, BuildError> {
    let mut outlines = Vec::with_capacity(masters.len());
    for (master_index, master) in masters.iter().enumerate() {
        let mut outline = match master.glyphs.get(glyph as usize) {
            Some(ParsedGlyph::Simple { contours }) => contours.clone(),
            Some(ParsedGlyph::Composite { .. }) => {
                flatten_composite_glyph(&master.glyphs, glyph, Affine::IDENTITY, 0).map_err(
                    |reason| BuildError::IncompatibleGlyph {
                        glyph,
                        master: request.masters[master_index].name.clone(),
                        reason,
                    },
                )?
            }
            Some(ParsedGlyph::Empty) | None => return Ok(None),
        };
        outline.retain(|contour| is_drawable_contour(contour));
        if outline.is_empty() {
            return Ok(None);
        }
        outlines.push(outline);
    }
    Ok(Some(outlines))
}

fn is_drawable_contour(contour: &[Point]) -> bool {
    if contour.len() < 3 {
        return false;
    }
    let (x_min, x_max, y_min, y_max) = contour.iter().fold(
        (i16::MAX, i16::MIN, i16::MAX, i16::MIN),
        |(x_min, x_max, y_min, y_max), point| {
            (
                x_min.min(point.x),
                x_max.max(point.x),
                y_min.min(point.y),
                y_max.max(point.y),
            )
        },
    );
    x_min != x_max && y_min != y_max
}

#[derive(Clone, Copy)]
struct Affine {
    xx: f64,
    yx: f64,
    xy: f64,
    yy: f64,
    dx: f64,
    dy: f64,
}

impl Affine {
    const IDENTITY: Self = Self {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: 1.0,
        dx: 0.0,
        dy: 0.0,
    };

    fn compose(self, other: Self) -> Self {
        Self {
            xx: self.xx * other.xx + self.xy * other.yx,
            yx: self.yx * other.xx + self.yy * other.yx,
            xy: self.xx * other.xy + self.xy * other.yy,
            yy: self.yx * other.xy + self.yy * other.yy,
            dx: self.xx * other.dx + self.xy * other.dy + self.dx,
            dy: self.yx * other.dx + self.yy * other.dy + self.dy,
        }
    }

    fn apply(self, point: Point) -> Result<Point, String> {
        let x = (self.xx * f64::from(point.x) + self.xy * f64::from(point.y) + self.dx)
            .round_ties_even() as i64;
        let y = (self.yx * f64::from(point.x) + self.yy * f64::from(point.y) + self.dy)
            .round_ties_even() as i64;
        if !(-32768..=32767).contains(&x) || !(-32768..=32767).contains(&y) {
            return Err("a decomposed composite point is outside TrueType coordinates".into());
        }
        Ok(Point {
            x: x as i16,
            y: y as i16,
            on_curve: point.on_curve,
        })
    }
}

fn flatten_composite_glyph(
    glyphs: &[ParsedGlyph],
    glyph: u16,
    transform: Affine,
    depth: usize,
) -> Result<Vec<Vec<Point>>, String> {
    if depth > 64 {
        return Err("component nesting is too deep".into());
    }
    match glyphs.get(glyph as usize) {
        Some(ParsedGlyph::Empty) => Ok(Vec::new()),
        Some(ParsedGlyph::Simple { contours }) => contours
            .iter()
            .map(|contour| {
                contour
                    .iter()
                    .copied()
                    .map(|point| transform.apply(point))
                    .collect()
            })
            .collect(),
        Some(ParsedGlyph::Composite { components }) => {
            let mut contours = Vec::new();
            for component in components {
                let read_fonts::tables::glyf::Anchor::Offset { x, y } = component.anchor else {
                    return Err("point-attached composite components cannot be flattened".into());
                };
                let component_transform = component.transform;
                let mut dx = f64::from(x);
                let mut dy = f64::from(y);
                if component
                    .flags
                    .contains(CompositeGlyphFlags::SCALED_COMPONENT_OFFSET)
                {
                    (dx, dy) = (
                        f64::from(component_transform.xx.to_f32()) * dx
                            + f64::from(component_transform.xy.to_f32()) * dy,
                        f64::from(component_transform.yx.to_f32()) * dx
                            + f64::from(component_transform.yy.to_f32()) * dy,
                    );
                }
                let component_affine = Affine {
                    xx: f64::from(component_transform.xx.to_f32()),
                    yx: f64::from(component_transform.yx.to_f32()),
                    xy: f64::from(component_transform.xy.to_f32()),
                    yy: f64::from(component_transform.yy.to_f32()),
                    dx,
                    dy,
                };
                contours.extend(flatten_composite_glyph(
                    glyphs,
                    component.glyph.to_u16(),
                    transform.compose(component_affine),
                    depth + 1,
                )?);
            }
            Ok(contours)
        }
        None => Err("composite references a missing glyph".into()),
    }
}

fn direct_metric_composites(masters: &[ParsedMaster], default_master: usize) -> BTreeSet<u16> {
    let Some(default) = masters.get(default_master) else {
        return BTreeSet::new();
    };
    default
        .glyphs
        .iter()
        .enumerate()
        .filter_map(|(glyph, default_glyph)| {
            let ParsedGlyph::Composite {
                components: default_components,
            } = default_glyph
            else {
                return None;
            };
            masters
                .iter()
                .enumerate()
                .any(|(master_index, master)| {
                    master_index != default_master
                        && matches!(
                            master.glyphs.get(glyph),
                            Some(ParsedGlyph::Composite { components })
                                if metric_source_flags_differ(default_components, components)
                        )
                })
                .then_some(glyph as u16)
        })
        .collect()
}

fn metric_source_flags_differ(
    base: &[read_fonts::tables::glyf::Component],
    target: &[read_fonts::tables::glyf::Component],
) -> bool {
    base.iter().zip(target).any(|(base, target)| {
        base.flags.contains(CompositeGlyphFlags::USE_MY_METRICS)
            != target.flags.contains(CompositeGlyphFlags::USE_MY_METRICS)
    })
}

fn rebuild_glyf_tables(
    font: &FontRef<'_>,
    prepared: &Prepared<'_>,
) -> Result<RewrittenGlyfTables, BuildError> {
    let master = &prepared.request.masters[prepared.default_master];
    let glyf = font.glyf().map_err(|error| invalid_font(master, error))?;
    let loca = font
        .loca(None)
        .map_err(|error| invalid_font(master, error))?;
    let mut builder = GlyfLocaBuilder::new();
    for glyph_id in 0..prepared.masters[prepared.default_master].glyph_count {
        if let Some(normalized) = prepared.normalized_simple_glyphs.get(&glyph_id) {
            builder
                .add_glyph(&normalized.default_glyph)
                .map_err(|error| BuildError::WriteTable {
                    table: "glyf",
                    reason: error.to_string(),
                })?;
            continue;
        }
        let mut glyph = loca
            .get_glyf(GlyphId::new(glyph_id as u32), &glyf)
            .map_err(|error| invalid_font(master, error))?
            .map_or(WriteGlyph::Empty, |glyph| {
                WriteGlyph::from_table_ref(&glyph)
            });
        if prepared.direct_metric_composites.contains(&glyph_id) {
            let WriteGlyph::Composite(composite) = &mut glyph else {
                return Err(BuildError::WriteTable {
                    table: "glyf",
                    reason: format!("glyph {glyph_id} was expected to be composite"),
                });
            };
            for component in composite.components_mut() {
                component.flags.use_my_metrics = false;
            }
            // The old program may refer to component metric state, which no
            // longer applies once the composite owns direct variable metrics.
            composite.set_instructions(&[]);
        }
        builder
            .add_glyph(&glyph)
            .map_err(|error| BuildError::WriteTable {
                table: "glyf",
                reason: error.to_string(),
            })?;
    }
    let (glyf, loca, loca_format) = builder.build();
    let mut head: Head = font
        .head()
        .map_err(|error| invalid_font(master, error))?
        .to_owned_table();
    head.index_to_loc_format = loca_format as i16;
    let mut maxp: Maxp = font
        .maxp()
        .map_err(|error| invalid_font(master, error))?
        .to_owned_table();
    let (max_points, max_contours) = prepared.normalized_simple_glyphs.values().fold(
        (0_u16, 0_u16),
        |(points, contours), normalized| {
            let glyph = &normalized.master_contours[prepared.default_master];
            let point_count = glyph.iter().map(Vec::len).sum::<usize>();
            (
                points.max(point_count.try_into().unwrap_or(u16::MAX)),
                contours.max(glyph.len().try_into().unwrap_or(u16::MAX)),
            )
        },
    );
    maxp.max_points = maxp.max_points.map(|value| value.max(max_points));
    maxp.max_contours = maxp.max_contours.map(|value| value.max(max_contours));

    Ok(RewrittenGlyfTables {
        glyf,
        loca,
        head,
        maxp,
    })
}

fn validate_all_glyphs(prepared: &Prepared<'_>) -> Result<(), BuildError> {
    for glyph in 0..prepared.masters[prepared.default_master].glyph_count {
        // Glyph zero is .notdef. Foundries commonly redraw it independently
        // for each static release, and a gvar table cannot express topology
        // changes. Keep the default glyph static if that happens; it is valid
        // OpenType behaviour and avoids rejecting an otherwise compatible
        // family because of its missing-glyph fallback.
        if glyph == 0 || prepared.frozen_glyphs.contains(&glyph) {
            continue;
        }
        let _ = glyph_coordinates_for_masters(prepared, glyph)?;
    }
    Ok(())
}

fn build_gvar_variations(
    prepared: &Prepared<'_>,
    model: &VariationModel,
) -> Result<Vec<GlyphVariations>, BuildError> {
    let glyph_count = prepared.masters[prepared.default_master].glyph_count;
    let mut variations = Vec::with_capacity(glyph_count as usize);
    for glyph in 0..glyph_count {
        if prepared.frozen_glyphs.contains(&glyph) {
            variations.push(GlyphVariations::new(GlyphId::new(glyph as u32), Vec::new()));
            continue;
        }
        let values = match glyph_coordinates_for_masters(prepared, glyph) {
            Ok(values) => values,
            Err(_) if glyph == 0 => {
                variations.push(GlyphVariations::new(GlyphId::new(glyph as u32), Vec::new()));
                continue;
            }
            Err(error) => return Err(error),
        };
        let tuples = model.deltas(&values)?;
        let deltas = tuples
            .into_iter()
            .filter(|(_, support, values)| {
                !support.is_default() && values.iter().any(|value| !value.is_zero())
            })
            .map(|(master_index, support, values)| {
                glyph_deltas(glyph, master_index, support, values, prepared)
            })
            .collect::<Result<Vec<_>, _>>()?;
        variations.push(GlyphVariations::new(GlyphId::new(glyph as u32), deltas));
    }
    Ok(variations)
}

/// Build a native HVAR table alongside gvar. Fontmake emits HVAR for advance
/// widths, while gvar retains its phantom-point deltas for geometry and side
/// bearings. Engines use HVAR for the horizontal advance, so using the same
/// variation model for both tables prevents a browser preview and a downloaded
/// font from disagreeing about spacing.
fn build_hvar(prepared: &Prepared<'_>, model: &VariationModel) -> Result<Hvar, BuildError> {
    let glyph_count = prepared.masters[prepared.default_master].glyph_count as usize;
    let values = prepared
        .masters
        .iter()
        .map(|master| {
            master
                .metrics
                .iter()
                .enumerate()
                .map(|(glyph, metric)| {
                    // mblode freezes both the outline and the width to the
                    // default donor. Do the same before constructing HVAR so
                    // a frozen contour cannot still reflow text.
                    let advance = if prepared.frozen_glyphs.contains(&(glyph as u16)) {
                        prepared.masters[prepared.default_master].metrics[glyph].advance
                    } else {
                        metric.advance
                    };
                    MetricValue(f64::from(advance))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let tuples = model.deltas(&values)?;
    let mut per_glyph = vec![Vec::new(); glyph_count];
    for (_, support, deltas) in tuples {
        if support.is_default() {
            continue;
        }
        let region = hvar_region(&support);
        for (glyph, delta) in deltas.into_iter().enumerate() {
            let delta = quantize_hvar_delta(delta.0, glyph as u16)?;
            if delta != 0 {
                per_glyph[glyph].push((region.clone(), delta));
            }
        }
    }

    // One direct row per glyph is larger than FontTools' optimised mapping, but
    // is fully native, deterministic and exactly preserves every advance-width
    // interpolation. Keep the explicit identity map: it matches Fontmake's
    // HVAR shape and makes the relationship clear to all consumers.
    let mut store =
        VariationStoreBuilder::new_with_implicit_indices(prepared.axis_tags.len() as u16);
    for deltas in per_glyph {
        store.add_deltas(deltas);
    }
    let (store, _) = store.build();
    let advance_width_mapping = (0..glyph_count as u32).collect::<DeltaSetIndexMap>();
    Ok(Hvar::new(store, Some(advance_width_mapping), None, None))
}

/// Fontmake creates GDEF even when a static donor did not carry one. Rebuild
/// its essential glyph classes from stable Unicode data, then retain a donor
/// GDEF verbatim when one was already present. Mark classification is needed
/// for shaping engines to apply mark positioning consistently after GPOS is
/// ported from the default donor.
fn build_gdef(default: &ParsedMaster) -> Gdef {
    let mark_glyphs: BTreeSet<_> = default
        .cmap
        .iter()
        .filter_map(|(codepoint, glyph)| {
            char::from_u32(*codepoint)
                .filter(|character| {
                    matches!(
                        get_general_category(*character),
                        GeneralCategory::NonspacingMark | GeneralCategory::SpacingMark
                    )
                })
                .map(|_| GlyphId16::new(*glyph))
        })
        .chain(
            default
                .glyph_names
                .iter()
                .enumerate()
                .filter_map(|(glyph, name)| {
                    name.as_deref()
                        .is_some_and(|name| name.ends_with("comb"))
                        .then_some(GlyphId16::new(glyph as u16))
                }),
        )
        .collect();
    let glyph_class_def = (!mark_glyphs.is_empty()).then(|| {
        mark_glyphs
            .into_iter()
            .map(|glyph| (glyph, GlyphClassDef::Mark as u16))
            .collect::<ClassDef>()
    });
    Gdef::new(glyph_class_def, None, None, None)
}

fn hvar_region(support: &Support) -> VariationRegion {
    VariationRegion::new(
        support
            .tents
            .iter()
            .map(|tent| {
                RegionAxisCoordinates::new(
                    F2Dot14::from_f32(tent.minimum as f32),
                    F2Dot14::from_f32(tent.peak as f32),
                    F2Dot14::from_f32(tent.maximum as f32),
                )
            })
            .collect(),
    )
}

fn quantize_hvar_delta(value: f64, glyph: u16) -> Result<i32, BuildError> {
    let rounded = value.round_ties_even();
    if !rounded.is_finite() || rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
        return Err(BuildError::WriteTable {
            table: "HVAR",
            reason: format!("glyph {glyph} has an advance delta outside HVAR's i32 range"),
        });
    }
    Ok(rounded as i32)
}

fn glyph_coordinates_for_masters(
    prepared: &Prepared<'_>,
    glyph: u16,
) -> Result<Vec<Vec<VariationPoint>>, BuildError> {
    if let Some(normalized) = prepared.normalized_simple_glyphs.get(&glyph) {
        // A repaired glyph replaces the default glyf record with a simple
        // outline. Its own advance is therefore authoritative, even when the
        // original static donor was a composite with USE_MY_METRICS. Resolving
        // the old component chain here would reject composites whose component
        // topology legitimately differs between releases (Ubuntu has one).
        return normalized
            .master_contours
            .iter()
            .enumerate()
            .map(|(master_index, contours)| {
                let mut points: Vec<_> = contours
                    .iter()
                    .flatten()
                    .map(|point| VariationPoint {
                        x: f64::from(point.x),
                        y: f64::from(point.y),
                    })
                    .collect();
                points.extend(phantom_points(
                    &prepared.masters[master_index].metrics[glyph as usize],
                ));
                Ok(points)
            })
            .collect();
    }
    let default = &prepared.masters[prepared.default_master];
    let base = &default.glyphs[glyph as usize];
    prepared
        .masters
        .iter()
        .enumerate()
        .map(|(master_index, master)| {
            let mut points: Vec<_> = match_glyph(base, &master.glyphs[glyph as usize])
                .map_err(|reason| BuildError::IncompatibleGlyph {
                    glyph,
                    master: prepared.request.masters[master_index].name.clone(),
                    reason,
                })?
                .into_iter()
                .map(|point| VariationPoint {
                    x: f64::from(point.x),
                    y: f64::from(point.y),
                })
                .collect();
            // Every gvar glyph, including a composite with USE_MY_METRICS,
            // has four phantom points after its outline/component points. If
            // we omit them then a required-delta run is decoded as applying
            // to all six points, which produces a malformed gvar record in
            // FontTools. The glyph's own hmtx metrics are the phantom basis;
            // HVAR supplies the matching advance-width variation.
            points.extend(phantom_points(&master.metrics[glyph as usize]));
            Ok(points)
        })
        .collect()
}

fn match_glyph(base: &ParsedGlyph, target: &ParsedGlyph) -> Result<Vec<Point>, String> {
    match (base, target) {
        (ParsedGlyph::Empty, ParsedGlyph::Empty) => Ok(Vec::new()),
        (
            ParsedGlyph::Simple {
                contours: base_contours,
            },
            ParsedGlyph::Simple {
                contours: target_contours,
            },
        ) => align_simple_contours(base_contours, target_contours),
        (
            ParsedGlyph::Composite {
                components: base_components,
            },
            ParsedGlyph::Composite {
                components: target_components,
            },
        ) => align_composite_components(base_components, target_components),
        _ => Err("glyph kind differs (empty, simple, or composite)".into()),
    }
}

fn align_composite_components(
    base: &[read_fonts::tables::glyf::Component],
    target: &[read_fonts::tables::glyf::Component],
) -> Result<Vec<Point>, String> {
    if base.len() != target.len() {
        return Err("composite component count differs".into());
    }
    base.iter()
        .zip(target)
        .map(|(base, target)| {
            if base.glyph != target.glyph {
                return Err("composite component glyphs differ".into());
            }
            if base.transform != target.transform {
                return Err("composite component transforms differ".into());
            }
            match (base.anchor, target.anchor) {
                (
                    read_fonts::tables::glyf::Anchor::Offset { .. },
                    read_fonts::tables::glyf::Anchor::Offset { x, y },
                ) => Ok(Point {
                    x,
                    y,
                    on_curve: true,
                }),
                (
                    read_fonts::tables::glyf::Anchor::Point {
                        base: base_base,
                        component: base_component,
                    },
                    read_fonts::tables::glyf::Anchor::Point {
                        base: target_base,
                        component: target_component,
                    },
                ) if base_base == target_base && base_component == target_component => {
                    Ok(Point::ZERO)
                }
                _ => Err("composite component anchors differ".into()),
            }
        })
        .collect()
}

#[derive(Clone)]
struct ContourAlignment {
    cost: i64,
    points: Vec<Point>,
}

fn align_simple_contours(base: &[Vec<Point>], target: &[Vec<Point>]) -> Result<Vec<Point>, String> {
    if base.len() != target.len() {
        return Err(format!(
            "contour count differs ({} vs {})",
            base.len(),
            target.len()
        ));
    }
    let count = base.len();
    if count == 0 {
        return Ok(Vec::new());
    }
    let candidates: Vec<Vec<Option<ContourAlignment>>> = base
        .iter()
        .map(|base_contour| {
            target
                .iter()
                .map(|target_contour| align_contour(base_contour, target_contour))
                .collect()
        })
        .collect();
    let assignment = min_cost_assignment(&candidates)?;
    assignment
        .into_iter()
        .enumerate()
        .flat_map(|(base_index, target_index)| {
            candidates[base_index][target_index]
                .as_ref()
                .unwrap()
                .points
                .clone()
        })
        .collect::<Vec<_>>()
        .pipe(Ok)
}

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}

impl<T> Pipe for T {}

fn align_contour(base: &[Point], target: &[Point]) -> Option<ContourAlignment> {
    if base.len() != target.len() {
        return None;
    }
    if base.is_empty() {
        return Some(ContourAlignment {
            cost: 0,
            points: Vec::new(),
        });
    }

    let mut best: Option<ContourAlignment> = None;
    for reversed in [false, true] {
        let points: Vec<_> = if reversed {
            target.iter().copied().rev().collect()
        } else {
            target.to_vec()
        };
        for shift in 0..points.len() {
            let aligned: Vec<_> = points
                .iter()
                .copied()
                .cycle()
                .skip(shift)
                .take(points.len())
                .collect();
            if base
                .iter()
                .zip(&aligned)
                .any(|(left, right)| left.on_curve != right.on_curve)
            {
                continue;
            }
            let cost = base
                .iter()
                .zip(&aligned)
                .fold(0_i64, |cost, (left, right)| {
                    let dx = i64::from(left.x) - i64::from(right.x);
                    let dy = i64::from(left.y) - i64::from(right.y);
                    cost.saturating_add(dx * dx + dy * dy)
                });
            if best.as_ref().is_none_or(|candidate| cost < candidate.cost) {
                best = Some(ContourAlignment {
                    cost,
                    points: aligned,
                });
            }
        }
    }
    best
}

fn min_cost_assignment(candidates: &[Vec<Option<ContourAlignment>>]) -> Result<Vec<usize>, String> {
    let count = candidates.len();
    let impossible = i64::MAX / 8;
    if candidates.iter().any(|row| row.iter().all(Option::is_none)) {
        return Err("a contour has no compatible point sequence".into());
    }
    // Hungarian minimum-cost assignment. It makes contour correspondence
    // deterministic even where marks or counters are reordered between masters.
    let mut potential_u = vec![0_i64; count + 1];
    let mut potential_v = vec![0_i64; count + 1];
    let mut matching = vec![0_usize; count + 1];
    let mut previous = vec![0_usize; count + 1];
    for row in 1..=count {
        matching[0] = row;
        let mut column = 0;
        let mut min_value = vec![impossible; count + 1];
        let mut used = vec![false; count + 1];
        loop {
            used[column] = true;
            let current_row = matching[column];
            let mut delta = impossible;
            let mut next_column = 0;
            for target in 1..=count {
                if used[target] {
                    continue;
                }
                let cost = candidates[current_row - 1][target - 1]
                    .as_ref()
                    .map_or(impossible, |candidate| candidate.cost);
                let reduced = cost
                    .saturating_sub(potential_u[current_row])
                    .saturating_sub(potential_v[target]);
                if reduced < min_value[target] {
                    min_value[target] = reduced;
                    previous[target] = column;
                }
                if min_value[target] < delta {
                    delta = min_value[target];
                    next_column = target;
                }
            }
            if delta == impossible {
                return Err("contours cannot be matched one-to-one".into());
            }
            for target in 0..=count {
                if used[target] {
                    potential_u[matching[target]] =
                        potential_u[matching[target]].saturating_add(delta);
                    potential_v[target] = potential_v[target].saturating_sub(delta);
                } else {
                    min_value[target] = min_value[target].saturating_sub(delta);
                }
            }
            column = next_column;
            if matching[column] == 0 {
                break;
            }
        }
        loop {
            let previous_column = previous[column];
            matching[column] = matching[previous_column];
            column = previous_column;
            if column == 0 {
                break;
            }
        }
    }
    let mut assignment = vec![0; count];
    for target in 1..=count {
        let row = matching[target];
        if row == 0 || candidates[row - 1][target - 1].is_none() {
            return Err("contours cannot be matched one-to-one".into());
        }
        assignment[row - 1] = target - 1;
    }
    Ok(assignment)
}

/// Hungarian minimum-cost assignment for the outline normalizer. Unlike the
/// strict matcher above, its costs are geometric contour signatures and every
/// pairing is allowed.
pub(super) fn hungarian_assignment(costs: &[Vec<i64>]) -> Result<Vec<usize>, String> {
    let count = costs.len();
    if count == 0 || costs.iter().any(|row| row.len() != count) {
        return Err("contours cannot be matched one-to-one".into());
    }
    let mut potential_u = vec![0_i64; count + 1];
    let mut potential_v = vec![0_i64; count + 1];
    let mut matching = vec![0_usize; count + 1];
    let mut previous = vec![0_usize; count + 1];
    for row in 1..=count {
        matching[0] = row;
        let mut column = 0;
        let mut min_value = vec![i64::MAX / 8; count + 1];
        let mut used = vec![false; count + 1];
        loop {
            used[column] = true;
            let current_row = matching[column];
            let mut delta = i64::MAX / 8;
            let mut next_column = 0;
            for target in 1..=count {
                if used[target] {
                    continue;
                }
                let reduced = costs[current_row - 1][target - 1]
                    .saturating_sub(potential_u[current_row])
                    .saturating_sub(potential_v[target]);
                if reduced < min_value[target] {
                    min_value[target] = reduced;
                    previous[target] = column;
                }
                if min_value[target] < delta {
                    delta = min_value[target];
                    next_column = target;
                }
            }
            if next_column == 0 {
                return Err("contours cannot be matched one-to-one".into());
            }
            for target in 0..=count {
                if used[target] {
                    potential_u[matching[target]] =
                        potential_u[matching[target]].saturating_add(delta);
                    potential_v[target] = potential_v[target].saturating_sub(delta);
                } else {
                    min_value[target] = min_value[target].saturating_sub(delta);
                }
            }
            column = next_column;
            if matching[column] == 0 {
                break;
            }
        }
        loop {
            let previous_column = previous[column];
            matching[column] = matching[previous_column];
            column = previous_column;
            if column == 0 {
                break;
            }
        }
    }
    let mut assignment = vec![0; count];
    for (target, row) in matching.iter().copied().enumerate().skip(1) {
        if row == 0 {
            return Err("contours cannot be matched one-to-one".into());
        }
        assignment[row - 1] = target - 1;
    }
    Ok(assignment)
}

fn phantom_points(metrics: &GlyphMetrics) -> [VariationPoint; 4] {
    let left_phantom = i32::from(metrics.x_min) - i32::from(metrics.side_bearing);
    let right_phantom = left_phantom + i32::from(metrics.advance);
    [
        VariationPoint {
            x: f64::from(left_phantom),
            y: 0.0,
        },
        VariationPoint {
            x: f64::from(right_phantom),
            y: 0.0,
        },
        VariationPoint::ZERO,
        VariationPoint::ZERO,
    ]
}

fn glyph_deltas(
    glyph: u16,
    master_index: usize,
    support: Support,
    values: Vec<VariationPoint>,
    prepared: &Prepared<'_>,
) -> Result<GlyphDeltas, BuildError> {
    let master_name = &prepared.request.masters[master_index].name;
    let deltas = values
        .into_iter()
        .map(|point| {
            let x = point.x.round_ties_even() as i64;
            let y = point.y.round_ties_even() as i64;
            if !(-32768..=32767).contains(&x) {
                return Err(BuildError::DeltaOutOfRange {
                    glyph,
                    master: master_name.clone(),
                    coordinate: "x",
                    value: x,
                });
            }
            if !(-32768..=32767).contains(&y) {
                return Err(BuildError::DeltaOutOfRange {
                    glyph,
                    master: master_name.clone(),
                    coordinate: "y",
                    value: y,
                });
            }
            Ok(GlyphDelta::required(x as i16, y as i16))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let tents = support
        .tents
        .into_iter()
        .map(|tent| {
            GvarTent::new(
                F2Dot14::from_f32(tent.peak as f32),
                Some((
                    F2Dot14::from_f32(tent.minimum as f32),
                    F2Dot14::from_f32(tent.maximum as f32),
                )),
            )
        })
        .collect();
    Ok(GlyphDeltas::new(tents, deltas))
}

fn append_variation_names(
    names: &mut Name,
    axes: &[Axis],
    masters: &[Master],
) -> Result<(Vec<NameId>, Vec<NameId>), BuildError> {
    let mut next_name_id = names
        .name_record
        .iter()
        .map(|record| record.name_id.to_u16())
        .max()
        .unwrap_or(NameId::LAST_RESERVED_NAME_ID.to_u16())
        .max(NameId::LAST_RESERVED_NAME_ID.to_u16())
        .saturating_add(1);
    let mut add_name = |value: &str| -> Result<NameId, BuildError> {
        if next_name_id > NameId::LAST_ALLOWED_NAME_ID.to_u16() {
            return Err(BuildError::WriteTable {
                table: "name",
                reason: "no name IDs remain for variation axes".into(),
            });
        }
        let name_id = NameId::new(next_name_id);
        names.name_record.push(NameRecord::new(
            3,
            1,
            0x0409,
            name_id,
            value.to_owned().into(),
        ));
        next_name_id = next_name_id.saturating_add(1);
        Ok(name_id)
    };
    let axis_ids = axes
        .iter()
        .map(|axis| add_name(&axis.name))
        .collect::<Result<Vec<_>, _>>()?;
    let instance_ids = masters
        .iter()
        .map(|master| add_name(&master.name))
        .collect::<Result<Vec<_>, _>>()?;
    names.name_record.sort();
    Ok((axis_ids, instance_ids))
}

fn build_fvar(
    axes: &[Axis],
    tags: &[Tag],
    name_ids: &[NameId],
    masters: &[Master],
    instance_name_ids: &[NameId],
) -> Fvar {
    let records = axes
        .iter()
        .zip(tags)
        .zip(name_ids)
        .map(|((axis, tag), name_id)| {
            VariationAxisRecord::new(
                *tag,
                Fixed::from_f64(f64::from(axis.minimum)),
                Fixed::from_f64(f64::from(axis.default)),
                Fixed::from_f64(f64::from(axis.maximum)),
                0,
                *name_id,
            )
        })
        .collect();
    let instances = masters
        .iter()
        .zip(instance_name_ids)
        .map(|(master, name_id)| InstanceRecord {
            subfamily_name_id: *name_id,
            flags: 0,
            coordinates: master
                .location
                .iter()
                .map(|location| Fixed::from_f64(f64::from(*location)))
                .collect(),
            post_script_name_id: None,
        })
        .collect();
    Fvar::new(AxisInstanceArrays::new(records, instances))
}

fn build_stat(
    tags: &[Tag],
    axis_name_ids: &[NameId],
    axes: &[Axis],
    masters: &[Master],
    instance_name_ids: &[NameId],
) -> Stat {
    let design_axes = tags
        .iter()
        .zip(axis_name_ids)
        .enumerate()
        .map(|(index, (tag, name_id))| AxisRecord::new(*tag, *name_id, index as u16))
        .collect();
    let mut axis_values = Vec::new();
    for (master, name_id) in masters.iter().zip(instance_name_ids) {
        let differing_axes: Vec<_> = master
            .location
            .iter()
            .zip(axes)
            .enumerate()
            .filter_map(|(axis, (value, definition))| {
                ((value - definition.default).abs() > f32::EPSILON).then_some(axis)
            })
            .collect();
        // STAT format 1 names an individual axis value. A corner master has
        // a compound style name, so omitting it is more truthful than attaching
        // that compound name independently to each axis.
        let axis = match differing_axes.as_slice() {
            [] if axes.len() == 1 => Some(0),
            [axis] => Some(*axis),
            _ => None,
        };
        let Some(axis) = axis else {
            continue;
        };
        let flags = if (master.location[axis] - axes[axis].default).abs() <= f32::EPSILON {
            AxisValueTableFlags::ELIDABLE_AXIS_VALUE_NAME
        } else {
            AxisValueTableFlags::empty()
        };
        axis_values.push(AxisValue::format_1(
            axis as u16,
            flags,
            *name_id,
            Fixed::from_f64(f64::from(master.location[axis])),
        ));
    }
    Stat::new(design_axes, axis_values, NameId::SUBFAMILY_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: i16, y: i16, on_curve: bool) -> Point {
        Point { x, y, on_curve }
    }

    fn master_with_named_empty_glyphs(names: &[&str], advances: &[u16]) -> ParsedMaster {
        ParsedMaster {
            glyph_count: names.len() as u16,
            units_per_em: 1_000,
            glyph_names: names.iter().map(|name| Some((*name).to_owned())).collect(),
            cmap: BTreeMap::new(),
            glyphs: names.iter().map(|_| ParsedGlyph::Empty).collect(),
            metrics: advances
                .iter()
                .map(|advance| GlyphMetrics {
                    x_min: 0,
                    advance: *advance,
                    side_bearing: 0,
                })
                .collect(),
        }
    }

    #[test]
    fn aligns_extra_and_reordered_donor_glyphs_to_the_default_bootstrap_order() {
        let default = master_with_named_empty_glyphs(&[".notdef", "a", "b"], &[500, 510, 520]);
        let target = master_with_named_empty_glyphs(
            &["b", "extra.private", ".notdef", "a"],
            &[820, 900, 800, 810],
        );

        let aligned = align_to_default_glyph_order(&default, target, "Other").unwrap();

        assert_eq!(aligned.glyph_count, 3);
        assert_eq!(
            aligned.glyph_names,
            vec![Some(".notdef".into()), Some("a".into()), Some("b".into())]
        );
        assert_eq!(
            aligned
                .metrics
                .iter()
                .map(|metric| metric.advance)
                .collect::<Vec<_>>(),
            vec![800, 810, 820]
        );
    }

    #[test]
    fn marks_missing_donor_glyphs_empty_for_the_freeze_stage() {
        let default = master_with_named_empty_glyphs(&[".notdef", "a", "b"], &[500, 510, 520]);
        let target = master_with_named_empty_glyphs(&[".notdef", "a"], &[800, 810]);

        let aligned = align_to_default_glyph_order(&default, target, "Other").unwrap();

        assert!(matches!(aligned.glyphs[2], ParsedGlyph::Empty));
        assert_eq!(aligned.metrics[2].advance, 520);
    }

    #[test]
    fn resolves_repair_selectors_from_the_cmap_when_post_names_are_missing() {
        let mut master =
            master_with_named_empty_glyphs(&[".notdef", "dollar", "S"], &[500, 510, 520]);
        master.glyph_names = vec![None, None, None];
        master.cmap.insert(u32::from('$'), 1);
        master.cmap.insert(u32::from('S'), 2);
        let repairs = vec![GlyphRepair {
            glyph: "U+0024".into(),
            strategy: GlyphRepairStrategy::OpenBar,
            letter: Some("S".into()),
            anchor: Some("left".into()),
            nub_overlap: None,
            min_protrude: None,
        }];

        assert_eq!(
            glyph_repair_index(&master, &repairs).unwrap().get(&1),
            Some(&0)
        );
    }

    #[test]
    fn aligns_rotated_contours() {
        let base = vec![vec![
            point(0, 0, true),
            point(100, 0, true),
            point(100, 100, true),
            point(0, 100, true),
        ]];
        let target = vec![vec![
            point(100, 0, true),
            point(100, 100, true),
            point(0, 100, true),
            point(0, 0, true),
        ]];
        assert_eq!(
            align_simple_contours(&base, &target).unwrap(),
            base.concat()
        );
    }

    #[test]
    fn matches_reordered_contours() {
        let first = vec![point(0, 0, true), point(10, 0, true)];
        let second = vec![point(100, 0, true), point(110, 0, true)];
        let aligned = align_simple_contours(
            &[first.clone(), second.clone()],
            &[second.clone(), first.clone()],
        )
        .unwrap();
        assert_eq!(aligned, [first, second].concat());
    }

    #[test]
    fn reads_weight_and_italic_metadata() {
        let regular_path = std::path::Path::new("/System/Library/Fonts/Monaco.ttf");
        if !regular_path.exists() {
            return;
        }
        let regular = analyze_static_font(&std::fs::read(regular_path).unwrap()).unwrap();
        assert!(regular.family.is_some());
        assert!(regular.weight.is_some());
        assert!(!regular.italic);

        let italic_path = std::path::Path::new("/System/Library/Fonts/SFNSItalic.ttf");
        if !italic_path.exists() {
            return;
        }
        let italic = analyze_static_font(&std::fs::read(italic_path).unwrap()).unwrap();
        assert!(italic.italic);
    }

    #[test]
    fn emits_a_readable_variable_font_from_static_ttf_masters() {
        let path = std::path::Path::new("/System/Library/Fonts/Monaco.ttf");
        if !path.exists() {
            return;
        }
        let bytes = std::fs::read(path).unwrap();
        let request = BuildRequest {
            axes: vec![Axis {
                tag: "wght".into(),
                name: "Weight".into(),
                minimum: 400.0,
                default: 400.0,
                maximum: 700.0,
            }],
            masters: vec![
                Master {
                    name: "Regular".into(),
                    location: vec![400.0],
                    bytes: bytes.clone(),
                },
                Master {
                    name: "Bold".into(),
                    location: vec![700.0],
                    bytes,
                },
            ],
            repairs: Vec::new(),
        };
        let built = build_variable_font(&request).unwrap();
        let parsed = FontRef::new(&built.font).unwrap();
        let fvar = parsed.fvar().unwrap();
        assert_eq!(fvar.axis_count(), 1);
        assert_eq!(fvar.axis_instance_arrays().unwrap().instances().len(), 2);
        assert!(parsed.gvar().is_ok());
        assert!(
            parsed
                .hvar()
                .unwrap()
                .advance_width_mapping()
                .is_some_and(|mapping| mapping.is_ok())
        );
        assert!(parsed.stat().is_ok());
        assert!(parsed.gdef().is_ok());
        assert!(parsed.data_for_tag(Tag::new(b"DSIG")).is_none());
        for tag in [b"cvt ", b"fpgm", b"prep", b"gasp"] {
            assert!(parsed.data_for_tag(Tag::new(tag)).is_none());
        }
    }
}
