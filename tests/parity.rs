use hapin_utils_rs::*;
use serde_json::{json, Value};

fn legacy(input: &str, from: Source, to: Target, clean: bool, easy: bool) -> String {
    let options = LegacyOptions { clean, easy };
    match (from, to) {
        (Source::Hapin, Target::Arabic) => transform_hapin_to_arabic_with_options(input, options),
        (Source::Hapin, Target::IpaCna) => transform_hapin_to_ipa_cna_with_options(input, options),
        (Source::Arabic, Target::Hapin) => transform_arabic_to_hapin_with_options(input, options),
        (Source::Arabic, Target::IpaCna) => {
            transform_arabic_to_ipa_cna_with_options(input, options)
        }
        (Source::Cyrillic, Target::Hapin) => {
            transform_cyrillic_to_hapin_with_options(input, options)
        }
        (Source::Cyrillic, Target::Arabic) => {
            transform_cyrillic_to_arabic_with_options(input, options)
        }
        (Source::Cyrillic, Target::IpaCna) => {
            transform_cyrillic_to_ipa_cna_with_options(input, options)
        }
        _ => unreachable!(),
    }
}
fn wrapper(name: &str) -> (Source, Target) {
    match name {
        "transformHapinToArabic" => (Source::Hapin, Target::Arabic),
        "transformHapinToIPA_CNA" => (Source::Hapin, Target::IpaCna),
        "transformArabicToHapin" => (Source::Arabic, Target::Hapin),
        "transformArabicToIPA_CNA" => (Source::Arabic, Target::IpaCna),
        "transformCyrillicToHapin" => (Source::Cyrillic, Target::Hapin),
        "transformCyrillicToArabic" => (Source::Cyrillic, Target::Arabic),
        "transformCyrillicToIPA_CNA" => (Source::Cyrillic, Target::IpaCna),
        _ => panic!("unknown oracle wrapper {name}"),
    }
}
#[test]
fn structured_results_match_typescript_5_0_0() {
    let mut count = 0;
    for (index, line) in include_str!("fixtures/parity.jsonl").lines().enumerate() {
        let row: Value = serde_json::from_str(line).unwrap();
        let input = row["input"].as_str().unwrap_or("");
        let options = row["options"].clone();
        let actual = match row["op"].as_str().unwrap() {
            "analyze" => serde_json::to_value(
                analyze_hapin(input, serde_json::from_value(options).unwrap()).unwrap(),
            )
            .unwrap(),
            "convert" => serde_json::to_value(
                convert(input, serde_json::from_value(options).unwrap()).unwrap(),
            )
            .unwrap(),
            "analysis-convert" => {
                let analysis: Analysis = serde_json::from_value(row["input"].clone()).unwrap();
                serde_json::to_value(
                    convert(&analysis, serde_json::from_value(options).unwrap()).unwrap(),
                )
                .unwrap()
            }
            "legacy" => {
                let (from, to) = wrapper(options["name"].as_str().unwrap());
                json!(legacy(
                    input,
                    from,
                    to,
                    options["clean"].as_bool().unwrap(),
                    options["easy"].as_bool().unwrap()
                ))
            }
            "array" => json!(match options["name"].as_str().unwrap() {
                "transformHapinToArabicArray" => transform_hapin_to_arabic_array(input),
                "transformArabicToHapinArray" => transform_arabic_to_hapin_array(input),
                "transformCyrillicToHapinArray" => transform_cyrillic_to_hapin_array(input),
                _ => unreachable!(),
            }),
            "rules" => serde_json::to_value(segmentation_rules()).unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(
            actual,
            row["expected"],
            "oracle line {}: op={} input={:?} options={}",
            index + 1,
            row["op"],
            row["input"],
            row["options"]
        );
        count += 1;
    }
    let metadata: Value = serde_json::from_str(include_str!("fixtures/metadata.json")).unwrap();
    assert_eq!(
        count,
        metadata["structuredCases"].as_u64().unwrap() as usize
    );
}
#[test]
fn seeded_differential_cases_match_typescript() {
    let mut count = 0;
    for (index, line) in include_str!("fixtures/differential.jsonl")
        .lines()
        .enumerate()
    {
        let row: Value = serde_json::from_str(line).unwrap();
        let input = row["input"].as_str().unwrap();
        let options: ConvertOptions = serde_json::from_value(row["options"].clone()).unwrap();
        let actual = convert(input, options).unwrap();
        let summary = json!({
            "text": actual.text,
            "status": actual.analysis.status,
            "codes": actual.analysis.diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>(),
            "losses": actual.losses.iter().map(|d| &d.code).collect::<Vec<_>>(),
            "legacyClean": legacy(input, options.from, options.to, true, false),
            "legacySeparated": legacy(input, options.from, options.to, false, false),
        });
        let expected = json!({ "text":row["text"], "status":row["status"], "codes":row["codes"], "losses":row["losses"], "legacyClean":row["legacyClean"], "legacySeparated":row["legacySeparated"] });
        assert_eq!(
            summary,
            expected,
            "seeded oracle line {}: input={input:?} options={:?}",
            index + 1,
            options
        );
        count += 1;
    }
    assert_eq!(count, 1600);
}
