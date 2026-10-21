use hapin_utils_rs::*;

#[test]
fn invalid_options_and_mutated_units_return_errors() {
    for max_candidates in [0, 65, usize::MAX] {
        let options = AnalyzeOptions {
            max_candidates,
            ..AnalyzeOptions::default()
        };
        assert_eq!(
            analyze_hapin("ngh", options),
            Err(Error::InvalidMaxCandidates)
        );
        let mut conversion = ConvertOptions::new(Source::Arabic, Target::Hapin);
        conversion.analysis = options;
        assert_eq!(convert("ا", conversion), Err(Error::InvalidMaxCandidates));
    }
    let original = analyze_hapin("a😀y", AnalyzeOptions::default()).unwrap();
    assert_eq!((original.units[2].start, original.units[2].end), (3, 4));
    let options = ConvertOptions::new(Source::Hapin, Target::Arabic);
    let mut cases = vec![];
    let mut a = original.clone();
    a.units[0].id = None;
    cases.push(a);
    let mut a = original.clone();
    a.units[1].raw = "wrong".into();
    cases.push(a);
    let mut a = original.clone();
    a.units[1].start = 2;
    cases.push(a); // Middle of a surrogate pair.
    let mut a = original.clone();
    a.units[1].start = 0;
    cases.push(a); // Overlapping units.
    let mut a = original.clone();
    a.units[0].end = usize::MAX;
    cases.push(a);
    let mut a = original.clone();
    a.units[2].end = 1;
    cases.push(a);
    for a in cases {
        assert_eq!(convert(&a, options), Err(Error::InvalidSourceUnit));
    }
}
#[test]
fn enum_deserialization_rejects_unsupported_runtime_values() {
    assert!(serde_json::from_str::<RulesVersion>("\"future\"").is_err());
    assert!(serde_json::from_str::<UnitKind>("\"invalid\"").is_err());
    assert!(serde_json::from_str::<LetterId>("\"invalid\"").is_err());
    assert!(serde_json::from_str::<ConvertOptions>(r#"{"from":"unknown","to":"arabic"}"#).is_err());
}
#[test]
fn explicit_candidate_keeps_source_boundaries_and_diagnostics() {
    let mut analysis = analyze_hapin("ngh", AnalyzeOptions::default()).unwrap();
    let options = ConvertOptions::new(Source::Hapin, Target::Arabic);
    assert_eq!(
        convert(&analysis, options).unwrap().text.as_deref(),
        Some("نع")
    );
    analysis.units = analysis.alternatives[0].units.clone();
    let result = convert(&analysis, options).unwrap();
    assert_eq!(result.text.as_deref(), Some("ڭح"));
    assert_eq!(result.analysis.status, Status::Ambiguous);
    let mut strict = options;
    strict.errors = ErrorsMode::Strict;
    assert_eq!(convert(&analysis, strict).unwrap().text, None);
}
#[test]
fn default_legacy_functions_and_prototype_paths_are_available() {
    assert_eq!(transform_hapin_to_arabic("ngh"), "ڭح");
    assert_eq!(transform_hapin_to_ipa_cna("xa xo xe xu"), "æ y i ø");
    assert_eq!(transform_arabic_to_hapin("نع"), "ngh");
    assert_eq!(transform_arabic_to_ipa_cna("نع"), "nʁ");
    assert_eq!(transform_cyrillic_to_hapin("нғ"), "ngh");
    assert_eq!(transform_cyrillic_to_arabic("нғ"), "نع");
    assert_eq!(transform_cyrillic_to_ipa_cna("нғ"), "nʁ");
    assert_eq!(
        hapin_utils_rs::transform_hapin_to_arabic::transform_hapin_to_arabic("ngh"),
        "ڭح"
    );
    assert_eq!(utils::utf8::get_char("a😀ب", 1), '😀');
    assert_eq!(utils::utf8::get_string_length("a😀ب"), 3);
}
#[test]
fn every_scalar_is_preserved_in_source_spans() {
    let input = "a😀y \"a\\\"b\"\nngh\u{200b}xa\u{301}\u{85}\u{feff}𐀀";
    for source in [Source::Hapin, Source::Arabic, Source::Cyrillic] {
        let analysis = convert(input, ConvertOptions::new(source, Target::IpaCna))
            .unwrap()
            .analysis;
        let raw: String = analysis.units.iter().map(|u| u.raw.as_str()).collect();
        assert_eq!(raw, input);
        let mut end = 0;
        for u in &analysis.units {
            assert_eq!(u.start, end);
            assert_eq!(u.end - u.start, u.raw.encode_utf16().count());
            end = u.end;
        }
        assert_eq!(end, input.encode_utf16().count());
    }
}
#[test]
fn long_adversarial_input_terminates_with_bounded_alternatives() {
    let input = "ngh shsh hh tsh ngul ngye ".repeat(4000);
    let analysis = analyze_hapin(&input, AnalyzeOptions::default()).unwrap();
    assert_eq!(analysis.alternatives.len(), 7);
    assert_eq!(
        analysis
            .diagnostics
            .iter()
            .filter(|d| d.code == "candidate-limit")
            .count(),
        1
    );
    let quote = format!("\"{}", "😀\\\"".repeat(10_000));
    let analysis = analyze_hapin(&quote, AnalyzeOptions::default()).unwrap();
    assert_eq!(analysis.units.len(), 1);
    assert_eq!(analysis.units[0].raw, quote);
    assert_eq!(analysis.diagnostics[0].code, "unclosed-quote");
}
