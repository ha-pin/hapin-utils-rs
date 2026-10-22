#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod compat;
mod convert;
mod letters;
mod parser;
mod render;
pub mod rules;
mod types;

pub use compat::*;
pub use convert::{analyze_hapin, convert, ConvertInput};
pub use rules::{segmentation_rules, RuleStatus, SegmentationRule, SEGMENTATION_RULES};
pub use types::*;

// Preserve the original prototype's public module paths.
pub mod transform_arabic_to_hapin {
    pub use crate::{
        transform_arabic_to_hapin, transform_arabic_to_hapin_array,
        transform_arabic_to_hapin_with_options,
    };
}
pub mod transform_hapin_to_arabic {
    pub use crate::{
        transform_hapin_to_arabic, transform_hapin_to_arabic_array,
        transform_hapin_to_arabic_with_options,
    };
}
pub mod transform_cyrillic_to_hapin {
    pub use crate::{
        transform_cyrillic_to_hapin, transform_cyrillic_to_hapin_array,
        transform_cyrillic_to_hapin_with_options,
    };
}
pub mod utils;

#[cfg(feature = "wasm")]
mod wasm;
