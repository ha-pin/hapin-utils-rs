use crate::letters::{arabic_letter, cyrillic_letter, HAPIN_SEQUENCES};
use crate::rules::{conflict_offset, NG_H_WORDS, S_H_WORDS};
use crate::{
    Alternative, Analysis, AnalyzeOptions, Diagnostic, InputFormat, LetterId, RulesVersion,
    Severity, Source, Status, Unit, UnitKind,
};
use unicode_general_category::{get_general_category, GeneralCategory as Gc};

// ECMAScript \s, rather than Rust's broader Unicode White_Space property.
fn whitespace(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}
fn word_char(c: char) -> bool {
    matches!(
        get_general_category(c),
        Gc::UppercaseLetter
            | Gc::LowercaseLetter
            | Gc::TitlecaseLetter
            | Gc::ModifierLetter
            | Gc::OtherLetter
            | Gc::NonspacingMark
            | Gc::SpacingMark
            | Gc::EnclosingMark
    )
}
fn punctuation(c: char) -> bool {
    c.is_ascii_digit()
        || matches!(
            get_general_category(c),
            Gc::ConnectorPunctuation
                | Gc::DashPunctuation
                | Gc::OpenPunctuation
                | Gc::ClosePunctuation
                | Gc::InitialPunctuation
                | Gc::FinalPunctuation
                | Gc::OtherPunctuation
                | Gc::MathSymbol
                | Gc::CurrencySymbol
                | Gc::ModifierSymbol
                | Gc::OtherSymbol
        )
}
struct Divergence {
    at: usize,
    remove: usize,
    units: Vec<Unit>,
    rule_id: String,
}
struct Scanner<'a> {
    input: &'a str,
    source: Source,
    chars: Vec<char>,
    bytes: Vec<usize>,
    utf16: Vec<usize>,
    units: Vec<Unit>,
    diagnostics: Vec<Diagnostic>,
    divergences: Vec<Divergence>,
    max: usize,
    index: usize,
}
impl<'a> Scanner<'a> {
    fn new(input: &'a str, source: Source, max: usize) -> Self {
        let mut bytes = Vec::new();
        let mut utf16 = vec![0];
        let mut chars = Vec::new();
        for (byte, c) in input.char_indices() {
            bytes.push(byte);
            chars.push(c);
            utf16.push(utf16.last().unwrap() + c.len_utf16());
        }
        bytes.push(input.len());
        Self {
            input,
            source,
            chars,
            bytes,
            utf16,
            units: vec![],
            diagnostics: vec![],
            divergences: vec![],
            max,
            index: 0,
        }
    }
    fn raw(&self, start: usize, end: usize) -> &str {
        &self.input[self.bytes[start]..self.bytes[end]]
    }
    fn unit(&self, kind: UnitKind, start: usize, end: usize) -> Unit {
        Unit {
            kind,
            start: self.utf16[start],
            end: self.utf16[end],
            raw: self.raw(start, end).into(),
            id: None,
            rule_id: None,
            source_letter: None,
        }
    }
    fn letter(&self, id: LetterId, start: usize, length: usize, rule: Option<&str>) -> Unit {
        let mut u = self.unit(UnitKind::Letter, start, start + length);
        u.id = Some(id);
        u.rule_id = rule.map(String::from);
        if self.source != Source::Hapin {
            u.source_letter = Some(u.raw.clone());
        }
        u
    }
    fn diagnostic(
        &mut self,
        code: &str,
        message: &str,
        start: usize,
        end: usize,
        severity: Severity,
    ) {
        self.diagnostics.push(Diagnostic::new(
            code,
            message,
            severity,
            self.utf16[start],
            self.utf16[end],
            self.raw(start, end),
        ));
    }
    fn diverge(&mut self, preferred: Vec<Unit>, alternate: Vec<Unit>, rule: &str) {
        let end = preferred.last().unwrap().end;
        if self.divergences.len() < self.max - 1 {
            self.divergences.push(Divergence {
                at: self.units.len(),
                remove: preferred.len(),
                units: alternate,
                rule_id: rule.into(),
            });
        } else if !self.diagnostics.iter().any(|d| d.code == "candidate-limit") {
            self.diagnostics.push(Diagnostic::new(
                "candidate-limit",
                "Additional local alternatives were omitted.",
                Severity::Warning,
                self.utf16[self.index],
                end,
                self.raw(
                    self.index,
                    self.index
                        + preferred
                            .iter()
                            .map(|u| u.raw.chars().count())
                            .sum::<usize>(),
                ),
            ));
        }
        self.diagnostics.push(Diagnostic::new(
            "ambiguous-segmentation",
            &format!("Preferred {rule}; another source-letter segmentation is possible."),
            Severity::Warning,
            self.utf16[self.index],
            end,
            &self.input[self.bytes[self.index]
                ..self.bytes[self.index] + preferred.iter().map(|u| u.raw.len()).sum::<usize>()],
        ));
        self.units.extend(preferred);
    }
    fn pair(
        &self,
        first: LetterId,
        first_len: usize,
        second: LetterId,
        second_len: usize,
        rule: Option<&str>,
    ) -> Vec<Unit> {
        vec![
            self.letter(first, self.index, first_len, rule),
            self.letter(second, self.index + first_len, second_len, None),
        ]
    }
    fn context(&mut self, tail: &str, word: &str, word_start: usize) -> bool {
        use LetterId::*;
        let i = self.index;
        let (preferred, alternate, rule, length) = if tail.starts_with("ngh") {
            if NG_H_WORDS.contains(&word) {
                (
                    self.pair(Ng, 2, H, 1, Some("ngh-source-exception")),
                    self.pair(N, 1, Gh, 2, None),
                    "ngh-source-exception",
                    3,
                )
            } else {
                (
                    self.pair(N, 1, Gh, 2, Some("ngh-prior")),
                    self.pair(Ng, 2, H, 1, None),
                    "ngh-prior",
                    3,
                )
            }
        } else if tail.starts_with("ng") && conflict_offset(word) == Some(i - word_start) {
            (
                vec![self.letter(Ng, i, 2, Some("source-spelling-conflict"))],
                self.pair(N, 1, G, 1, None),
                "source-spelling-conflict",
                2,
            )
        } else if tail.starts_with("ngul") {
            (
                self.pair(N, 1, G, 1, Some("ngul-context")),
                vec![self.letter(Ng, i, 2, None)],
                "ngul-context",
                2,
            )
        } else if tail.starts_with("ngye") || tail.starts_with("ngxe") {
            (
                vec![self.letter(Ng, i, 2, Some("ngye-conflict"))],
                self.pair(N, 1, G, 1, None),
                "ngye-conflict",
                2,
            )
        } else if tail.starts_with("shsh") {
            (
                vec![self.letter(Shsh, i, 4, Some("shsh-conflict"))],
                self.pair(Sh, 2, Sh, 2, None),
                "shsh-conflict",
                4,
            )
        } else if tail.starts_with("hh") {
            (
                vec![self.letter(Hh, i, 2, Some("hh-conflict"))],
                self.pair(H, 1, H, 1, None),
                "hh-conflict",
                2,
            )
        } else if tail.starts_with("sh") && word == "ashat" {
            (
                vec![self.letter(Sh, i, 2, Some("sh-word-conflict"))],
                self.pair(S, 1, H, 1, None),
                "sh-word-conflict",
                2,
            )
        } else if tail.starts_with("sh") && S_H_WORDS.contains(&word) {
            (
                self.pair(S, 1, H, 1, Some("sh-source-exception")),
                vec![self.letter(Sh, i, 2, None)],
                "sh-source-exception",
                2,
            )
        } else {
            return false;
        };
        self.diverge(preferred, alternate, rule);
        self.index += length;
        true
    }
}

pub(crate) fn parse(input: &str, source: Source, options: AnalyzeOptions) -> Analysis {
    let mut s = Scanner::new(input, source, options.max_candidates);
    let mut weak = false;
    let mut word_start = 0;
    let mut word_end = 0;
    let mut current_word = String::new();
    let mut pending_mark: Option<Unit> = None;
    while s.index < s.chars.len() {
        let start = s.index;
        let c = s.chars[start];
        if c == '"' {
            s.index += 1;
            let mut closed = false;
            while s.index < s.chars.len() {
                let q = s.chars[s.index];
                if q == '\\' && s.index + 1 < s.chars.len() {
                    s.index += 2;
                    continue;
                }
                s.index += 1;
                if q == '"' {
                    closed = true;
                    break;
                }
            }
            s.units.push(s.unit(UnitKind::Literal, start, s.index));
            if !closed {
                s.diagnostic(
                    "unclosed-quote",
                    "Unclosed quote preserved verbatim.",
                    start,
                    s.index,
                    Severity::Warning,
                );
            }
            weak = false;
            pending_mark = None;
            continue;
        }
        if whitespace(c) {
            while s.index < s.chars.len() && whitespace(s.chars[s.index]) {
                s.index += 1;
            }
            s.units.push(s.unit(UnitKind::Whitespace, start, s.index));
            weak = false;
            pending_mark = None;
            continue;
        }
        if c == '\u{200b}' {
            let kind = if options.input_format == InputFormat::LegacyBoundaries {
                UnitKind::Boundary
            } else {
                UnitKind::Literal
            };
            s.units.push(s.unit(kind, start, start + 1));
            s.index += 1;
            continue;
        }
        if source == Source::Arabic && matches!(c, 'ٴ' | 'ء' | 'ٵ' | 'ٶ' | 'ٷ' | 'ٸ') {
            let compact = match c {
                'ٵ' => Some(LetterId::Xa),
                'ٶ' => Some(LetterId::Xo),
                'ٷ' => Some(LetterId::Xu),
                'ٸ' => Some(LetterId::Xe),
                _ => None,
            };
            if let Some(id) = compact {
                s.units.push(s.letter(id, start, 1, None));
            } else {
                let mark = s.unit(UnitKind::SourceMark, start, start + 1);
                s.units.push(mark.clone());
                pending_mark = Some(mark);
                weak = true;
            }
            s.index += 1;
            continue;
        }
        let lower = c.to_lowercase().collect::<String>();
        if source == Source::Cyrillic && matches!(lower.as_str(), "ъ" | "ь") {
            s.units.push(s.unit(UnitKind::SourceMark, start, start + 1));
            s.diagnostic(
                "unrepresented-letter",
                "Hard/soft sign has no Hapin letter equivalent.",
                start,
                start + 1,
                Severity::Info,
            );
            s.index += 1;
            continue;
        }
        if source != Source::Hapin {
            let letter = if lower.chars().count() == 1 {
                let c = lower.chars().next().unwrap();
                if source == Source::Arabic {
                    arabic_letter(c)
                } else {
                    cyrillic_letter(c)
                }
            } else {
                None
            };
            if let Some(mut id) = letter {
                if weak {
                    let front = match id {
                        LetterId::A => Some(LetterId::Xa),
                        LetterId::O => Some(LetterId::Xo),
                        LetterId::U => Some(LetterId::Xu),
                        LetterId::E => Some(LetterId::Xe),
                        _ => None,
                    };
                    if let Some(front) = front {
                        id = front;
                        weak = false;
                        pending_mark = None;
                    }
                }
                s.units.push(s.letter(id, start, 1, None));
                s.index += 1;
                continue;
            }
        } else {
            if s.index >= word_end || s.index < word_start {
                word_start = s.index;
                word_end = s.index;
                while word_end < s.chars.len() && s.chars[word_end].is_ascii_alphabetic() {
                    word_end += 1;
                }
                current_word = s.raw(word_start, word_end).to_lowercase();
            }
            let tail: String = s
                .chars
                .iter()
                .skip(s.index)
                .take(4)
                .flat_map(|c| c.to_lowercase())
                .collect();
            if tail.starts_with("tsh") {
                if options.rules == RulesVersion::ContextV1 {
                    s.diverge(
                        s.pair(LetterId::T, 1, LetterId::Sh, 2, Some("tsh-overlap")),
                        s.pair(LetterId::Ts, 2, LetterId::H, 1, None),
                        "tsh-overlap",
                    );
                    s.index += 3;
                } else {
                    s.units
                        .push(s.letter(LetterId::T, start, 1, Some("alphabet")));
                    s.index += 1;
                }
                continue;
            }
            if options.rules == RulesVersion::ContextV1
                && s.context(&tail, &current_word, word_start)
            {
                continue;
            }
            if let Some(&(sequence, id)) = HAPIN_SEQUENCES
                .iter()
                .find(|(seq, _)| tail.starts_with(seq))
            {
                s.units
                    .push(s.letter(id, start, sequence.len(), Some("alphabet")));
                s.index += sequence.len();
                continue;
            }
            if lower == "x" {
                s.units.push(s.letter(LetterId::X, start, 1, None));
                s.diagnostic(
                    "standalone-modifier",
                    "Standalone x is a legacy weak-tone modifier.",
                    start,
                    start + 1,
                    Severity::Warning,
                );
                s.index += 1;
                continue;
            }
        }
        let kind = if punctuation(c) {
            UnitKind::Punctuation
        } else {
            UnitKind::Unknown
        };
        s.units.push(s.unit(kind, start, start + 1));
        if kind == UnitKind::Unknown {
            s.diagnostic(
                "unknown-character",
                "Unrecognized input preserved verbatim.",
                start,
                start + 1,
                Severity::Warning,
            );
        }
        if !word_char(c) {
            weak = false;
            pending_mark = None;
        }
        s.index += 1;
    }
    if let Some(mark) = pending_mark {
        s.diagnostics.push(Diagnostic::new(
            "dangling-modifier",
            "Weak-tone symbol without a following eligible vowel.",
            Severity::Warning,
            mark.start,
            mark.end,
            &mark.raw,
        ));
    }
    let alternatives = s
        .divergences
        .iter()
        .map(|d| {
            let mut units = s.units[..d.at].to_vec();
            units.extend_from_slice(&d.units);
            units.extend_from_slice(&s.units[d.at + d.remove..]);
            Alternative {
                units,
                rule_id: d.rule_id.clone(),
            }
        })
        .collect();
    let status = if s
        .diagnostics
        .iter()
        .any(|d| d.code == "ambiguous-segmentation")
    {
        Status::Ambiguous
    } else {
        Status::Ok
    };
    Analysis {
        source,
        input: input.into(),
        units: s.units,
        alternatives,
        diagnostics: s.diagnostics,
        rules_version: options.rules,
        status,
    }
}
