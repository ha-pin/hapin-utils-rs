# hapin-utils-rs

[简体中文](README.zh-CN.md) · [Hapin website](https://hapin.js.org/) · [TypeScript reference](https://github.com/ha-pin/hapin-utils) · [WASM downloads](https://github.com/ha-pin/hapin-utils-rs/releases/tag/v5.0.0)

[![Checks and release](https://github.com/ha-pin/hapin-utils-rs/actions/workflows/release.yml/badge.svg)](https://github.com/ha-pin/hapin-utils-rs/actions/workflows/release.yml)

Native Rust implementation of the **hapin-utils 5.0.0** conversion behavior, with `context-v1` segmentation, structured source units, diagnostics and legacy compatibility functions. The Rust crate version is **5.0.0**. Native conversion, native builds and native tests require neither JavaScript/Node.js nor a dictionary. Optional WebAssembly packages expose the same core to browsers, Node.js and compatible bundlers.

## Contents

- [Installation and availability](#installation)
- [Conversion](#conversion)
- [Analysis and candidate selection](#analysis-and-candidate-selection)
- [Options and diagnostics](#options-and-diagnostics)
- [Unicode and JSON interoperability](#unicode-and-json-interoperability)
- [Legacy API mapping](#legacy-api-mapping)
- [Development and verification](#development-and-verification)
- [Automatic publishing](#automatic-publishing)
- [WebAssembly packages](#webassembly-packages)
- [License](#license)

## Installation

As verified on **2026-10-22**, [WASM v5.0.0](https://github.com/ha-pin/hapin-utils-rs/releases/tag/v5.0.0) is available; the Rust crate has not yet been published to crates.io. A successful GitHub workflow or GitHub Release alone does not mean the crate was uploaded.

Use the released Git tag for a native Rust dependency:

```toml
[dependencies]
hapin-utils-rs = { git = "https://github.com/ha-pin/hapin-utils-rs", tag = "v5.0.0" }
```

Use `branch = "main"` instead of `tag` to follow development, or `rev` to select a specific reviewed commit. Commit your application's `Cargo.lock` to record the resolved Git revision.

After version 5.0.0 is actually available on [crates.io](https://crates.io/crates/hapin-utils-rs), the registry dependency is:

```toml
[dependencies]
hapin-utils-rs = "5.0.0"
```

The registry example is conditional on publication; use Git while the crate is unavailable. The crate imports as `hapin_utils_rs`. It uses Rust 2021; CI tests the stable Rust toolchain on Linux, macOS and Windows. No minimum Rust version is promised by this release.

## Conversion

```rust
use hapin_utils_rs::{convert, ConvertOptions, Source, Target};

let result = convert("арзанға", ConvertOptions::new(Source::Cyrillic, Target::Arabic))?;
assert_eq!(result.text.as_deref(), Some("ارزانعا"));

let result = convert("shanghai", ConvertOptions::new(Source::Hapin, Target::Arabic))?;
assert_eq!(result.text.as_deref(), Some("شاڭحاي"));
# Ok::<(), hapin_utils_rs::Error>(())
```

The seven supported directions are:

| Source | Targets |
| --- | --- |
| Hapin | Arabic, IPA_CNA |
| Arabic | Hapin, IPA_CNA |
| Cyrillic | Hapin, Arabic, IPA_CNA |

Use `Source::{Hapin, Arabic, Cyrillic}` and `Target::{Hapin, Arabic, IpaCna}`. Other combinations return `Ok(ConversionResult)` with `analysis.status == Status::Error`, `text == None` and an `unsupported-conversion` diagnostic. Cyrillic output is not provided by the reference API.

Conversion consumes original letter units directly. For example, Cyrillic `нғ`, `ңх` and `нг` remain separate source-letter sequences even though simplified Hapin spellings can collide. IPA_CNA is the same broad transcription as the TypeScript library, not a complete phonetic or morphological analyzer.

## Analysis and candidate selection

```rust
use hapin_utils_rs::{analyze_hapin, convert, AnalyzeOptions, ConvertOptions,
                     LetterId, Source, Status, Target};

let mut analysis = analyze_hapin("ngh", AnalyzeOptions::default())?;
let ids: Vec<_> = analysis.units.iter().filter_map(|u| u.id).collect();
assert_eq!(ids, vec![LetterId::N, LetterId::Gh]);
assert_eq!(analysis.status, Status::Ambiguous);

// Choose the returned NG + H alternative without reparsing the visible spelling.
analysis.units = analysis.alternatives[0].units.clone();
let result = convert(&analysis, ConvertOptions::new(Source::Hapin, Target::Arabic))?;
assert_eq!(result.text.as_deref(), Some("ڭح"));
# Ok::<(), hapin_utils_rs::Error>(())
```

`analyze_hapin` returns `Analysis`: source, original input, units, alternatives, diagnostics, rule version and status. `convert` accepts `&str`, `&String` or `&Analysis`. An existing analysis retains its rule version, units and diagnostics; its source must agree with `ConvertOptions.from`.

The context rules reproduce the reference priorities:

- `ngh`: prefer N + GH, with nine exact source-attested NG + H word forms such as `shanghai`. Extensions such as `shanghaixyz` do not inherit that exception.
- `ngul`: prefer N + G and expose NG as an alternative. `ngye` / `ngxe` retain competing NG and N + G analyses.
- Exact source-label conflicts, S + H word exceptions, `ashat`, `shsh`, `hh` and `tsh` overlaps retain the reference defaults and local alternatives.

`SEGMENTATION_RULES` / `segmentation_rules()` expose the versioned evidence metadata. Support counts are source-label observations, not held-out accuracy. They are fixed rules, not runtime dictionary lookup or statistical training.

Each alternative replaces **one** conflict in the preferred path; alternatives do not enumerate combinations of all conflicts. `max_candidates` includes the preferred path and must be 1–64 (default 8). Omitted alternatives produce a `candidate-limit` warning; a limit of 1 does not make ambiguous input unambiguous.

## Options and diagnostics

| Rust option | Default | Behavior |
| --- | --- | --- |
| `ConvertOptions.analysis.rules` | `RulesVersion::ContextV1` | `Legacy` selects the old Hapin splitting priority. |
| `analysis.input_format` | `InputFormat::Plain` | `LegacyBoundaries` consumes existing U+200B as machine boundaries. |
| `analysis.max_candidates` | `8` | Bounds the preferred path plus local alternatives. |
| `errors` | `ErrorsMode::Preserve` | `Strict` rejects all non-info diagnostics. |
| `formatting` | `Formatting::Preserve` | `Legacy` applies the old ASCII-space formatting. |

```rust
use hapin_utils_rs::{convert, ConvertOptions, ErrorsMode, Source, Target};

let mut options = ConvertOptions::new(Source::Hapin, Target::Arabic);
options.errors = ErrorsMode::Strict;
let result = convert("angye", options)?;
assert!(result.text.is_none());
assert!(result.analysis.diagnostics.iter().any(|d| d.code == "strict-input"));
# Ok::<(), hapin_utils_rs::Error>(())
```

Invalid candidate limits or invalid supplied unit spans return `Err(Error)`. Rust enums prevent unsupported runtime option names; Serde rejects unknown enum strings. Unsupported directions, source mismatches and strict-input failures are returned as diagnostic-bearing results with no partial text.

Preserve mode keeps whitespace, quoted literals, unknown characters and ordinary U+200B characters. Backslash-escaped quotes and unclosed quotes terminate safely; unclosed quotes are preserved and diagnosed. New conversion does not generate hidden boundary markers. Plain-text apostrophes are punctuation, not an input boundary syntax.

Warnings can exist with `Status::Ok`; inspect `diagnostics`, not just `status`. `Ambiguous` indicates competing segmentation and still includes a preferred output in preserve mode. Choosing a candidate retains the original warnings, so strict conversion will still reject them unless the caller explicitly reviews and resolves those diagnostics.

`ConversionResult.losses` reports `source-distinction` and `unencoded-boundary`. A Hapin output string may lose original spelling or boundaries. Keep the structured analysis for subsequent conversion rather than converting through a Hapin display string.

## Unicode and JSON interoperability

All `start` / `end` spans are **UTF-16 code-unit offsets** with an exclusive end, matching JavaScript, not UTF-8 byte indices or scalar indices. For `a😀y`, the `y` span is `[3, 4)`. Use `unit.raw` for source text; do not directly index a Rust string with these offsets. Rust accepts valid UTF-8 strings; unpaired JavaScript surrogate code units cannot be represented by `&str`.

`Analysis`, `ConversionResult`, units, diagnostics and options implement Serde. Their JSON uses the reference's camelCase fields and enum spellings, including `rulesVersion`, `sourceLetter`, `maxCandidates`, `context-v1` and `ipa-cna`. In Rust the result contains `result.analysis`; serialization flattens its fields to the same JSON shape as JavaScript.

## Legacy API mapping

Default wrappers take `&str` and return `String`. Their `_with_options` variants also take `LegacyOptions { clean, easy }` (default: true / false). `easy` affects only Arabic → Hapin. The Hapin wrappers intentionally retain legacy splitting: `transform_hapin_to_arabic("ngh") == "ڭح"`; use `convert` for context rules.

| JavaScript export | Rust default function |
| --- | --- |
| `transformHapinToArabic` | `transform_hapin_to_arabic` |
| `transformHapinToIPA_CNA` | `transform_hapin_to_ipa_cna` |
| `transformArabicToHapin` | `transform_arabic_to_hapin` |
| `transformArabicToIPA_CNA` | `transform_arabic_to_ipa_cna` |
| `transformCyrillicToHapin` | `transform_cyrillic_to_hapin` |
| `transformCyrillicToArabic` | `transform_cyrillic_to_arabic` |
| `transformCyrillicToIPA_CNA` | `transform_cyrillic_to_ipa_cna` |
| `transformHapinToArabicArray` | `transform_hapin_to_arabic_array` |
| `transformArabicToHapinArray` | `transform_arabic_to_hapin_array` |
| `transformCyrillicToHapinArray` | `transform_cyrillic_to_hapin_array` |

```rust
use hapin_utils_rs::{transform_arabic_to_hapin_with_options, LegacyOptions};
let result = transform_arabic_to_hapin_with_options(
    "جاقسى! ەرلان", LegacyOptions { clean: true, easy: true });
assert_eq!(result, "j'a'q's'e'! ye'r'l'a'n");
```

`clean: false` preserves the historical U+200B serialization, including source-specific separator placement. Array wrappers return `Vec<String>` by splitting that serialization; they are not a replacement for `Analysis.units`. Word-local weak vowels, IPA corrections, punctuation and quoted-text behavior match the corrected 5.0.0 compatibility implementation.

The prototype's `transform_arabic_to_hapin`, `transform_hapin_to_arabic`, `transform_cyrillic_to_hapin` module paths and `utils::utf8` helpers remain available. These old helpers count Unicode scalars; structured API spans use UTF-16. The unfinished prototype scanners and Hello World binary have been replaced by the library implementation.

## Development and verification

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo test --locked --all-features --all-targets
cargo test --locked --doc
cargo build --locked --release
cargo doc --locked --no-deps
cargo run --locked --example convert
```

Use stable Rust for native work. The full WASM/release checks additionally use **Python 3.11+**, **Node.js 24** and **wasm-pack 0.15.0**. `cargo test --locked` checks the default native feature set; the commands above also compile and test the optional WASM feature on the host.

CI runs format, Clippy, tests, doctests and a release build on Linux, macOS and Windows. English README Rust examples are doctests. PRs call `.github/workflows/check.yml`; main pushes call the same checks through `.github/workflows/release.yml`. Release credentials are only used by the publishing job, after validation.

Committed fixtures are generated by executing the actual TypeScript 5.0.0 package: **2,325 full structured/API cases** plus **1,600 seeded differential cases**. They cover all seven directions, all legacy wrappers and arrays, rule metadata, candidate limits, strict mode, selected analyses, UTF-16 spans, quotes, weak vowels, literals, source distinctions and serialization. Rust also tests malformed supplied analyses and adversarial long input. Fixtures and their source commit are recorded under `tests/fixtures/`; no desktop word list is required.

To intentionally regenerate the reference fixtures (maintainers only):

```sh
# First build a clean hapin-utils 5.0.0 checkout using its own build command.
node scripts/generate-parity-fixtures.cjs /absolute/path/to/hapin-utils
cargo test --locked
```

Review any fixture changes independently: regenerating snapshots alone does not demonstrate correctness. The reference generator is never called by Cargo or CI. Native conversion and its tests run entirely in Rust; the optional WASM adapter has separate Node-based interoperability tests.

## Automatic publishing

Pushing `main` runs `.github/workflows/release.yml`. It calls the same cross-platform checks used by pull requests, builds and tests WASM, and only then publishes missing versions. `workflow_dispatch` can retry the current main version without making another commit.

One-time crates.io setup:

1. Sign in to [crates.io](https://crates.io/me), verify your email and create an API token. For the first release it must permit creating `hapin-utils-rs`; later releases require publishing permission for that crate. Choose an expiration and rotate the token when needed.
2. In [repository Actions secrets](https://github.com/ha-pin/hapin-utils-rs/settings/secrets/actions), add a repository secret named **`CARGO_REGISTRY_TOKEN`** containing that token. Do not commit it or paste it into logs.
3. Push a version change to main, or open Actions → **Publish crate and WASM release** → **Run workflow** after configuring the secret. GitHub Releases use the built-in `GITHUB_TOKEN`, with `contents: write` limited to the release job; no personal GitHub token is required.

If the crates.io secret is absent, the workflow emits a warning and records **not published** in its summary; it does not claim a successful crate publication. WASM release can still succeed. Once the secret is set, rerun the workflow: existing WASM releases are left intact and the missing crate publication is attempted.

`Cargo.toml` is the version source. To publish the next release, change its package version, run `cargo check` to update `Cargo.lock`, and commit both to main. An existing crates.io version is skipped because it is immutable. An existing public GitHub Release is also skipped; its files are not replaced by later same-version commits. Bump the version when changing release contents. Prerelease Cargo versions create prerelease GitHub Releases. For a failed partial release, rerun the original workflow; an existing tag must point to that exact commit.

## WebAssembly packages

[Download v5.0.0](https://github.com/ha-pin/hapin-utils-rs/releases/tag/v5.0.0). Choose the target for your runtime:

| Target | Package | Initialization |
| --- | --- | --- |
| Browser / explicit ES module loading | [web](https://github.com/ha-pin/hapin-utils-rs/releases/download/v5.0.0/hapin-utils-rs-5.0.0-wasm-web.tar.gz) | `await init()` before calling the API. |
| Node.js / CommonJS | [nodejs](https://github.com/ha-pin/hapin-utils-rs/releases/download/v5.0.0/hapin-utils-rs-5.0.0-wasm-nodejs.tar.gz) | `require()` loads the WASM synchronously. |
| Bundlers configured to import `.wasm` | [bundler](https://github.com/ha-pin/hapin-utils-rs/releases/download/v5.0.0/hapin-utils-rs-5.0.0-wasm-bundler.tar.gz) | The bundler handles the generated WASM import. |

The release is named `v<version>` and contains:

- `hapin-utils-rs-<version>-wasm-web.tar.gz`
- `hapin-utils-rs-<version>-wasm-nodejs.tar.gz`
- `hapin-utils-rs-<version>-wasm-bundler.tar.gz`
- `SHA256SUMS`

Each target archive contains its `.wasm` binary, matching JavaScript glue, TypeScript declarations, package metadata, README and license. Download and extract the appropriate archive as a directory; keep its JS and WASM files together. This workflow uploads to GitHub Release; it does not publish an npm package.

The archives contain files at their root, not a `wasm-web` or `wasm-nodejs` parent directory.

Download [SHA256SUMS](https://github.com/ha-pin/hapin-utils-rs/releases/download/v5.0.0/SHA256SUMS) next to all three archives and verify before extraction:

```sh
sha256sum --check SHA256SUMS       # Linux
# macOS alternative:
shasum -a 256 --check SHA256SUMS
```

Extract each target into a separate folder:

```sh
mkdir -p wasm-web wasm-nodejs wasm-bundler
tar -xzf hapin-utils-rs-5.0.0-wasm-web.tar.gz -C wasm-web
tar -xzf hapin-utils-rs-5.0.0-wasm-nodejs.tar.gz -C wasm-nodejs
tar -xzf hapin-utils-rs-5.0.0-wasm-bundler.tar.gz -C wasm-bundler
```

For a browser, serve the extracted web directory via HTTP with the WASM MIME type `application/wasm`. Avoid opening the module through a `file://` URL. Initialize once before using any API:

```js
import init, { convert, analyzeHapin, segmentationRules } from './wasm-web/hapin_utils_rs.js';
await init();
console.log(convert('арзанға', { from: 'cyrillic', to: 'arabic' }).text);
console.log(analyzeHapin('ngh', {}).alternatives);
console.log(segmentationRules());
```

For Node.js, use the nodejs archive:

```js
const { convert } = require('./wasm-nodejs/hapin_utils_rs.js');
console.log(convert('арзанға', { from: 'cyrillic', to: 'arabic' }).text);
```

For a compatible bundler, import from the extracted bundler package:

```js
import { convert } from './wasm-bundler/hapin_utils_rs.js';
console.log(convert('арзанға', { from: 'cyrillic', to: 'arabic' }).text);
```

The bundler must support the generated static `.wasm` import (with its WASM integration or plugin). If it cannot, use the web package and explicit initialization instead. The nodejs package uses CommonJS; in an application with `"type": "module"`, load it through Node's `createRequire` rather than treating its JS as an ES module.

The WASM facade exports `convert`, `analyzeHapin`, `segmentationRules()` and the seven camelCase `transform*` functions plus three Array functions from the TypeScript API. It returns the same serializable object shapes. `convert` accepts text or an analysis object; missing optional analysis options use the defaults. WASM adapter/runtime tests use Node, while the default native Rust crate still has no JavaScript runtime requirement. The `wasm` Cargo feature is optional.

JavaScript options are flat and camelCase; Rust nests the analysis options in `ConvertOptions.analysis`:

| Rust | WASM JavaScript |
| --- | --- |
| `analysis.rules` | `rules` |
| `analysis.input_format` | `inputFormat` |
| `analysis.max_candidates` | `maxCandidates` |
| `errors`, `formatting`, `from`, `to` | Same field names. |

```js
const result = convert('angye', {
  from: 'hapin', to: 'arabic', rules: 'context-v1',
  inputFormat: 'plain', maxCandidates: 8,
  errors: 'strict', formatting: 'preserve',
});
console.log(result.text); // null: unresolved segmentation is rejected in strict mode
console.log(result.diagnostics);
```

WASM `convert` returns `diagnostics`, `units` and the other analysis fields at the top level; native Rust accesses these through `result.analysis`. `segmentationRules()` is a function in the WASM facade, while the TypeScript reference also exposes its rules as exported data. Invalid WASM options or malformed supplied analyses throw; diagnostic-bearing failures such as strict input return `text: null` as described above.

For local WASM verification, install the prerequisites above and run:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
python3 scripts/build-wasm.py
node scripts/test-wasm.cjs
python3 scripts/package-wasm.py
python3 -m unittest discover -s scripts -p 'test_*.py'
```

The real Node and web WASM bindings each run the 2,325 full oracle cases before artifacts can be uploaded. Release planning fails on registry/network errors other than a genuine 404; invalid assets or a tag pointing to another commit stop release creation. Release assets are assembled and verified in a draft before it is made public. Failed drafts can be recovered through the authenticated release listing, even when the by-tag endpoint does not return them. The configured tag author remains Herbert He.

## License

MIT. Maintained by Herbert He <Herbert.He0229@gmail.com>.
