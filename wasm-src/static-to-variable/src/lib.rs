//! Browser-facing entry points for YABE's static TrueType to variable-font engine.
//!
//! The implementation deliberately stays in Rust/WASM. Fontations provides the
//! OpenType parsers and writers; all master validation, contour alignment and
//! variation-model calculation live in this crate.

mod error;
mod model;
mod ttf;
mod variation_model;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

pub use error::BuildError;
pub use model::{BuildRequest, BuildResult};

#[wasm_bindgen(js_name = buildVariableFont)]
pub fn build_variable_font(request: JsValue) -> Result<Uint8Array, JsValue> {
    let request: BuildRequest = serde_wasm_bindgen::from_value(request)
        .map_err(|error| JsValue::from_str(&format!("Invalid conversion request: {error}")))?;
    let result = ttf::build_variable_font(&request)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    Ok(Uint8Array::from(result.font.as_slice()))
}

#[wasm_bindgen(js_name = validateVariableFontRequest)]
pub fn validate_variable_font_request(request: JsValue) -> Result<JsValue, JsValue> {
    let request: BuildRequest = serde_wasm_bindgen::from_value(request)
        .map_err(|error| JsValue::from_str(&format!("Invalid conversion request: {error}")))?;
    let result =
        ttf::validate_request(&request).map_err(|error| JsValue::from_str(&error.to_string()))?;
    serde_wasm_bindgen::to_value(&result).map_err(|error| {
        JsValue::from_str(&format!("Could not serialize validation report: {error}"))
    })
}

#[wasm_bindgen(js_name = analyzeStaticFont)]
pub fn analyze_static_font(bytes: &[u8]) -> Result<JsValue, JsValue> {
    let result = ttf::analyze_static_font(bytes).map_err(|error| JsValue::from_str(&error))?;
    serde_wasm_bindgen::to_value(&result)
        .map_err(|error| JsValue::from_str(&format!("Could not serialize font analysis: {error}")))
}

#[wasm_bindgen]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
