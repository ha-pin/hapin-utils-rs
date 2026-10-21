//! Typed inputs and serializable results. All spans use UTF-16 code units.
use serde::{Deserialize, Serialize};
use std::{error::Error as StdError, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Hapin,
    Arabic,
    Cyrillic,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    Hapin,
    Arabic,
    IpaCna,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RulesVersion {
    #[serde(rename = "legacy")]
    Legacy,
    #[default]
    #[serde(rename = "context-v1")]
    ContextV1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LetterId {
    A,
    B,
    T,
    J,
    H,
    D,
    R,
    Z,
    S,
    Sh,
    Gh,
    F,
    Q,
    K,
    L,
    M,
    N,
    O,
    E,
    I,
    P,
    Ch,
    Ng,
    G,
    Hh,
    V,
    U,
    W,
    Ye,
    Xa,
    Xo,
    Xe,
    Xu,
    Shsh,
    Ts,
    Iw,
    Ia,
    Io,
    X,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitKind {
    Letter,
    Literal,
    Unknown,
    Whitespace,
    Punctuation,
    Boundary,
    SourceMark,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub kind: UnitKind,
    pub start: usize,
    pub end: usize,
    pub raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<LetterId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_letter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub start: usize,
    pub end: usize,
    pub raw: String,
    // Source-distinction losses retain the originating unit metadata, as in JS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<UnitKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<LetterId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_letter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
}
impl Diagnostic {
    pub(crate) fn new(
        code: &str,
        message: &str,
        severity: Severity,
        start: usize,
        end: usize,
        raw: &str,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            start,
            end,
            raw: raw.into(),
            kind: None,
            id: None,
            source_letter: None,
            rule_id: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alternative {
    pub units: Vec<Unit>,
    pub rule_id: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Ambiguous,
    Error,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub source: Source,
    pub input: String,
    pub units: Vec<Unit>,
    pub alternatives: Vec<Alternative>,
    pub diagnostics: Vec<Diagnostic>,
    pub rules_version: RulesVersion,
    pub status: Status,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputFormat {
    #[default]
    Plain,
    LegacyBoundaries,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AnalyzeOptions {
    pub rules: RulesVersion,
    pub input_format: InputFormat,
    pub max_candidates: usize,
}
impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            rules: RulesVersion::ContextV1,
            input_format: InputFormat::Plain,
            max_candidates: 8,
        }
    }
}
impl AnalyzeOptions {
    pub(crate) fn validate(self) -> Result<(), Error> {
        if !(1..=64).contains(&self.max_candidates) {
            return Err(Error::InvalidMaxCandidates);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorsMode {
    #[default]
    Preserve,
    Strict,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Formatting {
    #[default]
    Preserve,
    Legacy,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertOptions {
    pub from: Source,
    pub to: Target,
    #[serde(flatten)]
    pub analysis: AnalyzeOptions,
    #[serde(default)]
    pub errors: ErrorsMode,
    #[serde(default)]
    pub formatting: Formatting,
}
impl ConvertOptions {
    pub fn new(from: Source, to: Target) -> Self {
        Self {
            from,
            to,
            analysis: AnalyzeOptions::default(),
            errors: ErrorsMode::Preserve,
            formatting: Formatting::Preserve,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionResult {
    #[serde(flatten)]
    pub analysis: Analysis,
    pub text: Option<String>,
    pub losses: Vec<Diagnostic>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidMaxCandidates,
    InvalidSourceUnit,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidMaxCandidates => "maxCandidates must be an integer between 1 and 64",
            Self::InvalidSourceUnit => "Invalid source unit or span",
        })
    }
}
impl StdError for Error {}
