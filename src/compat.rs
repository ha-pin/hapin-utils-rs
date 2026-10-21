//! Compatibility functions retain the legacy segmentation and serialization.
use crate::parser::parse;
use crate::render::{render, RenderOptions};
use crate::{AnalyzeOptions, InputFormat, RulesVersion, Source, Target};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyOptions {
    /// False emits historical U+200B separators.
    pub clean: bool,
    /// Arabic-to-Hapin only: display apostrophes between serialized units.
    pub easy: bool,
}
impl Default for LegacyOptions {
    fn default() -> Self {
        Self {
            clean: true,
            easy: false,
        }
    }
}
fn transform(input: &str, from: Source, to: Target, options: LegacyOptions) -> String {
    let analysis = parse(
        input,
        from,
        AnalyzeOptions {
            rules: RulesVersion::Legacy,
            input_format: InputFormat::LegacyBoundaries,
            ..AnalyzeOptions::default()
        },
    );
    render(
        &analysis,
        to,
        RenderOptions {
            legacy: true,
            separators: !options.clean,
            easy: options.easy,
        },
    )
}
macro_rules! wrapper {
    ($name:ident, $with:ident, $from:ident, $to:ident) => {
        pub fn $name(input: &str) -> String {
            $with(input, LegacyOptions::default())
        }
        pub fn $with(input: &str, options: LegacyOptions) -> String {
            transform(input, Source::$from, Target::$to, options)
        }
    };
}
wrapper!(
    transform_hapin_to_arabic,
    transform_hapin_to_arabic_with_options,
    Hapin,
    Arabic
);
wrapper!(
    transform_hapin_to_ipa_cna,
    transform_hapin_to_ipa_cna_with_options,
    Hapin,
    IpaCna
);
wrapper!(
    transform_arabic_to_hapin,
    transform_arabic_to_hapin_with_options,
    Arabic,
    Hapin
);
wrapper!(
    transform_arabic_to_ipa_cna,
    transform_arabic_to_ipa_cna_with_options,
    Arabic,
    IpaCna
);
wrapper!(
    transform_cyrillic_to_hapin,
    transform_cyrillic_to_hapin_with_options,
    Cyrillic,
    Hapin
);
wrapper!(
    transform_cyrillic_to_arabic,
    transform_cyrillic_to_arabic_with_options,
    Cyrillic,
    Arabic
);
wrapper!(
    transform_cyrillic_to_ipa_cna,
    transform_cyrillic_to_ipa_cna_with_options,
    Cyrillic,
    IpaCna
);
macro_rules! array_wrapper {
    ($name:ident, $with:ident) => {
        pub fn $name(input: &str) -> Vec<String> {
            $with(
                input,
                LegacyOptions {
                    clean: false,
                    easy: false,
                },
            )
            .split('\u{200b}')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
        }
    };
}
array_wrapper!(
    transform_hapin_to_arabic_array,
    transform_hapin_to_arabic_with_options
);
array_wrapper!(
    transform_arabic_to_hapin_array,
    transform_arabic_to_hapin_with_options
);
array_wrapper!(
    transform_cyrillic_to_hapin_array,
    transform_cyrillic_to_hapin_with_options
);
