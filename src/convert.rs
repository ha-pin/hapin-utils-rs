use crate::parser::parse;
use crate::render::{render, RenderOptions};
use crate::{
    Analysis, AnalyzeOptions, ConversionResult, ConvertOptions, Diagnostic, Error, ErrorsMode,
    Formatting, LetterId, Severity, Source, Status, Target, UnitKind,
};

pub enum ConvertInput<'a> {
    Text(&'a str),
    Analysis(&'a Analysis),
}
impl<'a> From<&'a str> for ConvertInput<'a> {
    fn from(value: &'a str) -> Self {
        Self::Text(value)
    }
}
impl<'a> From<&'a String> for ConvertInput<'a> {
    fn from(value: &'a String) -> Self {
        Self::Text(value)
    }
}
impl<'a> From<&'a Analysis> for ConvertInput<'a> {
    fn from(value: &'a Analysis) -> Self {
        Self::Analysis(value)
    }
}

/// Analyze Hapin using context-v1 by default; spans match JavaScript UTF-16 offsets.
pub fn analyze_hapin(input: &str, options: AnalyzeOptions) -> Result<Analysis, Error> {
    options.validate()?;
    Ok(parse(input, Source::Hapin, options))
}
fn validate_analysis(analysis: &Analysis) -> Result<(), Error> {
    // Some positions inside surrogate pairs are intentionally not string boundaries.
    let mut boundaries = vec![None; analysis.input.encode_utf16().count() + 1];
    let mut offset = 0;
    for (byte, c) in analysis.input.char_indices() {
        boundaries[offset] = Some(byte);
        offset += c.len_utf16();
    }
    boundaries[offset] = Some(analysis.input.len());
    let mut previous_end = 0;
    for u in &analysis.units {
        let start = boundaries.get(u.start).copied().flatten();
        let end = boundaries.get(u.end).copied().flatten();
        if u.start < previous_end
            || u.end < u.start
            || (u.kind == UnitKind::Letter && u.id.is_none())
        {
            return Err(Error::InvalidSourceUnit);
        }
        match (start, end) {
            (Some(start), Some(end)) if analysis.input[start..end] == u.raw => (),
            _ => return Err(Error::InvalidSourceUnit),
        }
        previous_end = u.end;
    }
    Ok(())
}
fn whole_error(analysis: &Analysis, code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(
        code,
        message,
        Severity::Error,
        0,
        analysis.input.encode_utf16().count(),
        &analysis.input,
    )
}
fn ids(analysis: &Analysis) -> Vec<LetterId> {
    analysis
        .units
        .iter()
        .filter(|u| u.kind == UnitKind::Letter)
        .filter_map(|u| u.id)
        .collect()
}

/// Convert text or a validated analysis, preserving source letter boundaries.
/// Invalid options/spans return Err; unsupported directions and strict input
/// produce a result with status Error and text None, matching hapin-utils 5.0.0.
pub fn convert<'a>(
    input: impl Into<ConvertInput<'a>>,
    options: ConvertOptions,
) -> Result<ConversionResult, Error> {
    options.analysis.validate()?;
    let mut analysis = match input.into() {
        ConvertInput::Text(input) => parse(input, options.from, options.analysis),
        ConvertInput::Analysis(analysis) => {
            validate_analysis(analysis)?;
            analysis.clone()
        }
    };
    let mut losses = vec![];
    if !matches!(
        (options.from, options.to),
        (Source::Hapin, Target::Arabic | Target::IpaCna)
            | (Source::Arabic, Target::Hapin | Target::IpaCna)
            | (
                Source::Cyrillic,
                Target::Hapin | Target::Arabic | Target::IpaCna
            )
    ) {
        let diagnostic = whole_error(
            &analysis,
            "unsupported-conversion",
            "This source/target direction is not supported.",
        );
        analysis.diagnostics.push(diagnostic);
    }
    if analysis.source != options.from {
        let diagnostic = whole_error(
            &analysis,
            "source-mismatch",
            "Analysis source does not match from.",
        );
        analysis.diagnostics.push(diagnostic);
    }
    for u in &analysis.units {
        let source = u.source_letter.as_deref().unwrap_or("");
        if u.kind == UnitKind::SourceMark
            || source.chars().any(|c| matches!(c, 'ٵ' | 'ٶ' | 'ٷ' | 'ٸ'))
            || (analysis.source == Source::Cyrillic
                && source
                    .to_lowercase()
                    .chars()
                    .any(|c| matches!(c, 'и' | 'э')))
        {
            let mut d = Diagnostic::new(
                "source-distinction",
                "Source spelling distinction is not represented in the target text.",
                Severity::Info,
                u.start,
                u.end,
                &u.raw,
            );
            d.kind = Some(u.kind);
            d.id = u.id;
            d.source_letter = u.source_letter.clone();
            d.rule_id = u.rule_id.clone();
            losses.push(d);
        }
    }
    if options.errors == ErrorsMode::Strict
        && analysis
            .diagnostics
            .iter()
            .any(|d| d.severity != Severity::Info)
    {
        let diagnostic = whole_error(
            &analysis,
            "strict-input",
            "Unresolved input requires an explicit choice or correction.",
        );
        analysis.diagnostics.push(diagnostic);
    }
    let failed = analysis
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error);
    let text = if failed {
        None
    } else {
        Some(render(
            &analysis,
            options.to,
            RenderOptions {
                legacy: options.formatting == Formatting::Legacy,
                ..RenderOptions::default()
            },
        ))
    };
    if let Some(ref text) = text {
        if options.to == Target::Hapin {
            let display = parse(
                text,
                Source::Hapin,
                AnalyzeOptions {
                    rules: analysis.rules_version,
                    ..AnalyzeOptions::default()
                },
            );
            if display.status == Status::Ambiguous || ids(&display) != ids(&analysis) {
                losses.push(Diagnostic::new("unencoded-boundary", "Simplified Hapin text does not uniquely encode the source letter boundaries. Keep the structured units.", Severity::Info, 0, analysis.input.encode_utf16().count(), &analysis.input));
            }
        }
    }
    if failed {
        analysis.status = Status::Error;
    }
    Ok(ConversionResult {
        analysis,
        text,
        losses,
    })
}
