use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("At least two static TrueType masters are required")]
    TooFewMasters,
    #[error("At least one variation axis is required")]
    NoAxes,
    #[error("Axis {axis} must use a four-character OpenType tag")]
    InvalidAxisTag { axis: String },
    #[error("Axis tag {axis} is configured more than once")]
    DuplicateAxisTag { axis: String },
    #[error("Axis {axis} must have min < default < max or min <= default <= max")]
    InvalidAxisRange { axis: String },
    #[error("Master {master} has {actual} axis coordinates; expected {expected}")]
    WrongLocationLength {
        master: String,
        expected: usize,
        actual: usize,
    },
    #[error("Master {master} has a coordinate outside the configured range for axis {axis}")]
    LocationOutsideAxisRange { master: String, axis: String },
    #[error("Exactly one master must be located at the axis defaults; found {count}")]
    InvalidDefaultMasterCount { count: usize },
    #[error("Master {master} is not a valid static TrueType glyf font: {reason}")]
    InvalidTrueType { master: String, reason: String },
    #[error(
        "Master {master} already contains variable-font data ({table}); static TTF masters are required"
    )]
    VariableInput { master: String, table: String },
    #[error("Master {master} has {actual} glyphs, but the default master has {expected}")]
    GlyphCountMismatch {
        master: String,
        expected: u16,
        actual: u16,
    },
    #[error("Master {master} uses {actual} units per em, but the default master uses {expected}")]
    UnitsPerEmMismatch {
        master: String,
        expected: u16,
        actual: u16,
    },
    #[error("Glyph {glyph} in master {master} cannot be interpolated: {reason}")]
    IncompatibleGlyph {
        glyph: u16,
        master: String,
        reason: String,
    },
    #[error(
        "Glyph {glyph} in master {master} produces a {coordinate}-delta of {value}, outside gvar's i16 range"
    )]
    DeltaOutOfRange {
        glyph: u16,
        master: String,
        coordinate: &'static str,
        value: i64,
    },
    #[error("The supplied masters duplicate the same normalized design-space location")]
    DuplicateLocation,
    #[error("The supplied masters do not include a valid default location")]
    MissingDefault,
    #[error("Unable to build {table}: {reason}")]
    WriteTable { table: &'static str, reason: String },
    #[error("The generated font still contains collapsing glyphs after 40 repair passes")]
    FreezeLoopDidNotConverge,
    #[error("Glyph repair for {glyph} is invalid: {reason}")]
    InvalidGlyphRepair { glyph: String, reason: String },
}
