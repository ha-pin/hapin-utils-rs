# Changelog

## 5.0.0

- Replace the unfinished 0.1.0 prototype with a native Rust implementation matching the public conversion behavior of TypeScript hapin-utils 5.0.0.
- Implement all seven conversion directions, three legacy arrays and legacy clean/easy options.
- Add context-v1 analysis, local alternatives, versioned rules, diagnostics, strict conversion and source-loss reports.
- Preserve source letter boundaries, UTF-16 offsets, Unicode literals, layout, weak vowels and word-local IPA behavior.
- Support conversion of validated structured analyses and Serde JSON interoperability.
- Add English/Simplified Chinese READMEs, tested examples, cross-platform CI and 3,925 TypeScript oracle cases.
- Remove the placeholder binary and unused utf8_slice dependency; retain prototype module paths and Unicode scalar helpers.

The source is pushed to GitHub; a crates.io publication is a separate maintainer action.
