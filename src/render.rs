use crate::letters::{arabic_output, ipa_output, punctuation};
use crate::{Analysis, LetterId, Source, Target, Unit, UnitKind};

#[derive(Default, Clone, Copy)]
pub(crate) struct RenderOptions {
    pub legacy: bool,
    pub separators: bool,
    pub easy: bool,
}
fn vowel(id: LetterId) -> bool {
    matches!(
        id,
        LetterId::A
            | LetterId::E
            | LetterId::O
            | LetterId::U
            | LetterId::Ye
            | LetterId::Xa
            | LetterId::Xo
            | LetterId::Xe
            | LetterId::Xu
    )
}
fn front(id: LetterId) -> bool {
    matches!(
        id,
        LetterId::Ye | LetterId::Xa | LetterId::Xo | LetterId::Xe | LetterId::Xu
    )
}
fn ipa_letters(letters: &[LetterId]) -> Vec<String> {
    let first = letters
        .iter()
        .find(|&&id| vowel(id) || matches!(id, LetterId::Ia | LetterId::Io));
    let is_front = first.map_or_else(
        || {
            letters
                .iter()
                .any(|id| matches!(id, LetterId::G | LetterId::K))
        },
        |&id| front(id),
    );
    letters
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            let previous = i.checked_sub(1).and_then(|j| letters.get(j)).copied();
            let next = letters.get(i + 1).copied();
            if matches!(id, LetterId::I | LetterId::W)
                && !previous.is_some_and(vowel)
                && !previous.is_some_and(|id| matches!(id, LetterId::Ia | LetterId::Io))
                && !next.is_some_and(vowel)
            {
                let prefix = match (id, is_front) {
                    (LetterId::I, true) => "i",
                    (LetterId::I, false) => "ə",
                    (_, true) => "y",
                    (_, false) => "u",
                };
                format!("{prefix}{}", ipa_output(id))
            } else {
                ipa_output(id).to_owned()
            }
        })
        .collect()
}
fn collapse_spaces(raw: &str) -> String {
    let mut space = false;
    raw.chars()
        .filter(|&c| {
            let keep = c != ' ' || !space;
            space = c == ' ';
            keep
        })
        .collect()
}
fn flush(
    word: &[&Unit],
    source: Source,
    target: Target,
    options: RenderOptions,
    result: &mut Vec<String>,
) {
    if word.is_empty() {
        return;
    }
    let letters: Vec<_> = word
        .iter()
        .filter(|u| u.kind == UnitKind::Letter)
        .map(|u| u.id.unwrap())
        .collect();
    let omit = letters
        .iter()
        .any(|id| matches!(id, LetterId::K | LetterId::G | LetterId::Ye));
    let weak = letters.iter().any(|id| {
        matches!(
            id,
            LetterId::X | LetterId::Xa | LetterId::Xe | LetterId::Xo | LetterId::Xu
        )
    });
    let ipa = if target == Target::IpaCna {
        ipa_letters(&letters)
    } else {
        vec![]
    };
    let mut text = String::new();
    let mut index = 0;
    for u in word {
        if u.kind == UnitKind::Boundary {
            if options.separators {
                text.push('\u{200b}');
            }
            continue;
        }
        if u.kind != UnitKind::Letter {
            continue;
        }
        let id = u.id.unwrap();
        text.push_str(match target {
            Target::Hapin => id.as_str(),
            Target::Arabic => arabic_output(id),
            Target::IpaCna => &ipa[index],
        });
        index += 1;
        if options.separators && source != Source::Hapin {
            text.push('\u{200b}');
        }
    }
    if target == Target::Arabic && weak && !omit {
        text.insert(0, 'ٴ');
    }
    result.push(text);
}
/// Renders source-letter units directly, without a serialized Hapin intermediate.
pub(crate) fn render(analysis: &Analysis, target: Target, options: RenderOptions) -> String {
    let mut result: Vec<String> = vec![];
    let mut word = vec![];
    for u in &analysis.units {
        if matches!(
            u.kind,
            UnitKind::Letter | UnitKind::Boundary | UnitKind::SourceMark
        ) {
            word.push(u);
            continue;
        }
        flush(&word, analysis.source, target, options, &mut result);
        word.clear();
        if u.kind == UnitKind::Whitespace {
            result.push(if options.legacy {
                collapse_spaces(&u.raw)
            } else {
                u.raw.clone()
            });
            if options.separators && analysis.source == Source::Cyrillic {
                result.push("\u{200b}".into());
            }
        } else {
            let text = if u.kind == UnitKind::Punctuation {
                punctuation(&u.raw, target == Target::Arabic)
            } else {
                &u.raw
            };
            if options.legacy
                && u.kind == UnitKind::Punctuation
                && text
                    .chars()
                    .next()
                    .is_some_and(|c| matches!(c, '!' | '#'..='/' | ':'..='@'))
            {
                while result
                    .last()
                    .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c == ' '))
                {
                    result.pop();
                }
            }
            result.push(text.into());
            if options.separators
                && ((analysis.source != Source::Hapin && u.kind != UnitKind::Literal)
                    || (analysis.source == Source::Cyrillic && u.kind == UnitKind::Literal))
            {
                result.push("\u{200b}".into());
            }
        }
    }
    flush(&word, analysis.source, target, options, &mut result);
    let mut text = result.concat();
    if options.easy && analysis.source == Source::Arabic && target == Target::Hapin {
        let mut chunks = String::new();
        // Precompute the next non-mark unit for linear-time easy serialization.
        let mut next_non_mark = vec![None; analysis.units.len()];
        let mut next = None;
        for (i, u) in analysis.units.iter().enumerate().rev() {
            next_non_mark[i] = next;
            if u.kind != UnitKind::SourceMark {
                next = Some(u);
            }
        }
        for (i, u) in analysis.units.iter().enumerate() {
            if u.kind == UnitKind::SourceMark {
                continue;
            }
            let value = match u.kind {
                UnitKind::Letter => u.id.unwrap().as_str(),
                UnitKind::Punctuation => punctuation(&u.raw, false),
                _ => &u.raw,
            };
            chunks.push_str(value);
            if next_non_mark[i].is_some_and(|n| n.kind != UnitKind::Whitespace)
                && u.kind != UnitKind::Whitespace
                && u.kind != UnitKind::Literal
            {
                chunks.push('\'');
            }
            if options.separators && u.kind != UnitKind::Whitespace {
                chunks.push('\u{200b}');
            }
        }
        text = chunks;
    }
    if options.legacy && analysis.source == Source::Arabic {
        text = text.trim_matches(' ').into();
    }
    text
}
