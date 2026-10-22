//! JavaScript facade; the conversion logic remains in the native Rust core.
use crate::*;
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn value<T: Serialize>(input: &T) -> Result<JsValue, JsValue> {
    input
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
fn error(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

#[wasm_bindgen(js_name = analyzeHapin)]
pub fn analyze(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let options = if options.is_undefined() || options.is_null() {
        AnalyzeOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(error)?
    };
    value(&analyze_hapin(input, options).map_err(error)?)
}
#[wasm_bindgen(js_name = convert)]
pub fn convert_js(input: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
    let options: ConvertOptions = serde_wasm_bindgen::from_value(options).map_err(error)?;
    let result = if let Some(input) = input.as_string() {
        convert(input.as_str(), options)
    } else {
        let analysis: Analysis = serde_wasm_bindgen::from_value(input).map_err(error)?;
        convert(&analysis, options)
    };
    value(&result.map_err(error)?)
}
#[wasm_bindgen(js_name = segmentationRules)]
pub fn rules() -> Result<JsValue, JsValue> {
    value(&segmentation_rules())
}

#[wasm_bindgen(js_name = transformHapinToArabic)]
pub fn js_transform_hapin_to_arabic(input: &str, clean: Option<bool>) -> String {
    transform_hapin_to_arabic_with_options(
        input,
        LegacyOptions {
            clean: clean.unwrap_or(true),
            easy: false,
        },
    )
}

#[wasm_bindgen(js_name = transformHapinToIPA_CNA)]
pub fn js_transform_hapin_to_ipa_cna(input: &str, clean: Option<bool>) -> String {
    transform_hapin_to_ipa_cna_with_options(
        input,
        LegacyOptions {
            clean: clean.unwrap_or(true),
            easy: false,
        },
    )
}

#[wasm_bindgen(js_name = transformArabicToHapin)]
pub fn js_transform_arabic_to_hapin(
    input: &str,
    clean: Option<bool>,
    easy: Option<bool>,
) -> String {
    transform_arabic_to_hapin_with_options(
        input,
        LegacyOptions {
            clean: clean.unwrap_or(true),
            easy: easy.unwrap_or(false),
        },
    )
}

#[wasm_bindgen(js_name = transformArabicToIPA_CNA)]
pub fn js_transform_arabic_to_ipa_cna(input: &str, clean: Option<bool>) -> String {
    transform_arabic_to_ipa_cna_with_options(
        input,
        LegacyOptions {
            clean: clean.unwrap_or(true),
            easy: false,
        },
    )
}

#[wasm_bindgen(js_name = transformCyrillicToHapin)]
pub fn js_transform_cyrillic_to_hapin(input: &str, clean: Option<bool>) -> String {
    transform_cyrillic_to_hapin_with_options(
        input,
        LegacyOptions {
            clean: clean.unwrap_or(true),
            easy: false,
        },
    )
}

#[wasm_bindgen(js_name = transformCyrillicToArabic)]
pub fn js_transform_cyrillic_to_arabic(input: &str, clean: Option<bool>) -> String {
    transform_cyrillic_to_arabic_with_options(
        input,
        LegacyOptions {
            clean: clean.unwrap_or(true),
            easy: false,
        },
    )
}

#[wasm_bindgen(js_name = transformCyrillicToIPA_CNA)]
pub fn js_transform_cyrillic_to_ipa_cna(input: &str, clean: Option<bool>) -> String {
    transform_cyrillic_to_ipa_cna_with_options(
        input,
        LegacyOptions {
            clean: clean.unwrap_or(true),
            easy: false,
        },
    )
}

#[wasm_bindgen(js_name = transformHapinToArabicArray)]
pub fn js_transform_hapin_to_arabic_array(input: &str) -> Result<JsValue, JsValue> {
    value(&transform_hapin_to_arabic_array(input))
}

#[wasm_bindgen(js_name = transformArabicToHapinArray)]
pub fn js_transform_arabic_to_hapin_array(input: &str) -> Result<JsValue, JsValue> {
    value(&transform_arabic_to_hapin_array(input))
}

#[wasm_bindgen(js_name = transformCyrillicToHapinArray)]
pub fn js_transform_cyrillic_to_hapin_array(input: &str) -> Result<JsValue, JsValue> {
    value(&transform_cyrillic_to_hapin_array(input))
}
