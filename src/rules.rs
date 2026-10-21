//! Versioned context-v1 segmentation evidence; counts are source-label support, not accuracy.
use crate::LetterId;
use serde::Serialize;

pub(crate) const NG_H_WORDS: &[&str] = &[
    "tunghwa",
    "junghwa",
    "hungha",
    "hunghwa",
    "shanghai",
    "shanghaida",
    "shanghaigha",
    "shinghaw",
    "wanghung",
];
pub(crate) const S_H_WORDS: &[&str] = &[
    "ashana",
    "ashanada",
    "ashanadan",
    "ashanase",
    "ashanagha",
    "ashanane",
    "ashananeng",
    "eshanada",
    "eshanagha",
    "yeshat",
    "jarashan",
];
pub(crate) fn conflict_offset(word: &str) -> Option<usize> {
    Some(match word {
        "aldenge" => 4,
        "angmye" => 1,
        "angemye" => 1,
        "ang" => 1,
        "berdyengye" => 6,
        "jyekyengye" => 6,
        "jyengye" => 3,
        "jyengyem" => 3,
        "songe" => 2,
        "syekyengye" => 6,
        "mange" => 2,
        "yerkengye" => 5,
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleStatus {
    SourceAttested,
    Prior,
    Candidate,
    Ambiguous,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SegmentationRule {
    pub id: &'static str,
    pub status: RuleStatus,
    pub pattern: &'static str,
    pub preferred: &'static [LetterId],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterexamples: Option<usize>,
}
pub const SEGMENTATION_RULES: &[SegmentationRule] = &[
    SegmentationRule {
        id: "ngh-source-exception",
        status: RuleStatus::SourceAttested,
        pattern: "ngh",
        preferred: &[LetterId::Ng, LetterId::H],
        support: Some(9),
        counterexamples: None,
    },
    SegmentationRule {
        id: "ngh-prior",
        status: RuleStatus::Prior,
        pattern: "ngh",
        preferred: &[LetterId::N, LetterId::Gh],
        support: Some(458),
        counterexamples: Some(9),
    },
    SegmentationRule {
        id: "ngul-context",
        status: RuleStatus::Candidate,
        pattern: "ngul",
        preferred: &[LetterId::N, LetterId::G],
        support: Some(27),
        counterexamples: None,
    },
    SegmentationRule {
        id: "ngye-conflict",
        status: RuleStatus::Ambiguous,
        pattern: "ngye/ngxe",
        preferred: &[LetterId::Ng],
        support: Some(387),
        counterexamples: Some(238),
    },
    SegmentationRule {
        id: "sh-source-exception",
        status: RuleStatus::SourceAttested,
        pattern: "sh",
        preferred: &[LetterId::S, LetterId::H],
        support: None,
        counterexamples: None,
    },
    SegmentationRule {
        id: "source-spelling-conflict",
        status: RuleStatus::Ambiguous,
        pattern: "word-form",
        preferred: &[LetterId::Ng],
        support: None,
        counterexamples: None,
    },
    SegmentationRule {
        id: "sh-word-conflict",
        status: RuleStatus::Ambiguous,
        pattern: "ashat",
        preferred: &[LetterId::Sh],
        support: None,
        counterexamples: None,
    },
    SegmentationRule {
        id: "shsh-conflict",
        status: RuleStatus::Ambiguous,
        pattern: "shsh",
        preferred: &[LetterId::Shsh],
        support: None,
        counterexamples: None,
    },
    SegmentationRule {
        id: "hh-conflict",
        status: RuleStatus::Ambiguous,
        pattern: "hh",
        preferred: &[LetterId::Hh],
        support: None,
        counterexamples: None,
    },
    SegmentationRule {
        id: "tsh-overlap",
        status: RuleStatus::Ambiguous,
        pattern: "tsh",
        preferred: &[LetterId::T, LetterId::Sh],
        support: None,
        counterexamples: None,
    },
];

pub fn segmentation_rules() -> &'static [SegmentationRule] {
    SEGMENTATION_RULES
}
