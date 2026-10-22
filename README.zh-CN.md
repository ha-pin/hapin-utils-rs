# hapin-utils-rs

[English](README.md) · [Hapin 官网](https://hapin.js.org/)

完整移植 **hapin-utils 5.0.0** 对外转换行为的原生 Rust 工具包，包含 `context-v1` 切分、结构化字母单元、诊断、损失报告及旧版兼容接口。Rust 包版本为 **5.0.0**。运行、构建和测试均不依赖 JavaScript、Node.js 或桌面词库。

## 安装

首次 crates.io 发布完成前，使用 Git 依赖：

```toml
[dependencies]
hapin-utils-rs = { git = "https://github.com/ha-pin/hapin-utils-rs", branch = "main" }
```

应用需要固定依赖时，改用已审阅提交的 `rev`。Rust 导入名为 `hapin_utils_rs`。项目使用 Rust 2021 edition，CI 在 Linux、macOS、Windows 的 stable 工具链上验证；本版本尚未声明最低 Rust 版本。

## 基本转换

```rust
use hapin_utils_rs::{convert, ConvertOptions, Source, Target};

fn main() -> Result<(), hapin_utils_rs::Error> {
    let result = convert("арзанға", ConvertOptions::new(Source::Cyrillic, Target::Arabic))?;
    assert_eq!(result.text.as_deref(), Some("ارزانعا"));
    Ok(())
}
```

| 输入 | 支持的输出 |
| --- | --- |
| Hapin | Arabic、IPA_CNA |
| Arabic | Hapin、IPA_CNA |
| Cyrillic | Hapin、Arabic、IPA_CNA |

共七种方向。枚举使用 `Source::{Hapin, Arabic, Cyrillic}` 和 `Target::{Hapin, Arabic, IpaCna}`。其他组合返回带 `unsupported-conversion` 诊断的结果，`analysis.status` 为 `Status::Error`、`text` 为 `None`，不返回半成品文本。与 TypeScript 一致，本 API 不提供 Cyrillic 输出。

转换直接读取原文字母单元，不借助会丢失边界的 Hapin 字符串中转。因此 `нғ`、`ңх`、`нг` 能保留不同字母组合。IPA_CNA 复现参考库的宽式转写，不包含完整语音分析或形态分析。

## 分析、切分和候选

```rust
use hapin_utils_rs::{analyze_hapin, convert, AnalyzeOptions, ConvertOptions, Source, Target};

fn main() -> Result<(), hapin_utils_rs::Error> {
    let mut analysis = analyze_hapin("ngh", AnalyzeOptions::default())?;
    // 默认首选 N + GH；切换到返回的 NG + H 局部候选。
    analysis.units = analysis.alternatives[0].units.clone();
    let result = convert(&analysis, ConvertOptions::new(Source::Hapin, Target::Arabic))?;
    assert_eq!(result.text.as_deref(), Some("ڭح"));
    Ok(())
}
```

`analyze_hapin` 返回原文、单元、候选、诊断、规则版本和状态。`convert` 接受 `&str`、`&String` 或 `&Analysis`。传入结构化分析时直接复用其单元、诊断和规则版本，`from` 必须与分析的 `source` 一致。

规则与 TypeScript 5.0.0 完全对应：

- `ngh` 默认 N + GH；`shanghai` 等九个完整词形保留 NG + H 的来源证据。不会向任意前缀扩展例外。
- `ngul` 默认 N + G 并保留 NG 候选；`ngye` / `ngxe` 保留竞争分析。
- 原有词形冲突、S + H 例外、`ashat`、`shsh`、`hh`、`tsh` 按参考优先级切分并报告歧义。

`SEGMENTATION_RULES` / `segmentation_rules()` 提供对应的规则证据元数据。计数是来源标签支持数，不是独立测试集准确率，也不表示运行时查询词库或训练模型。

每个候选仅替换首选路径中的**一个局部冲突**，不会枚举所有冲突的组合。`max_candidates` 包含首选路径，取值 1–64，默认 8。超出上限报告 `candidate-limit`；设置为 1 不会消除歧义诊断。

## 选项、状态与错误

| Rust 字段 | 默认值 | 说明 |
| --- | --- | --- |
| `ConvertOptions.analysis.rules` | `RulesVersion::ContextV1` | 可选 `Legacy`，使用旧 Hapin 切分优先级。 |
| `analysis.input_format` | `InputFormat::Plain` | `LegacyBoundaries` 把已有 U+200B 当作机器边界。 |
| `analysis.max_candidates` | `8` | 首选路径与局部候选的总上限。 |
| `errors` | `ErrorsMode::Preserve` | `Strict` 拒绝所有非 info 诊断。 |
| `formatting` | `Formatting::Preserve` | `Legacy` 应用旧 ASCII 空格处理方式。 |

候选上限错误、外部传入单元的范围错误通过 `Err(Error)` 返回。无效选项名称由 Rust 枚举或 Serde 反序列化拒绝。方向不支持、来源不匹配和严格模式失败则返回有诊断、无输出文本的 `Ok(ConversionResult)`。

默认保留空白、引号字面量、未知字符及普通 U+200B。转义引号和未闭合引号均能正常结束扫描；未闭合内容原样保留并报告诊断。新接口不主动生成隐藏边界；普通 apostrophe 是标点，不是新的边界输入语法。

`Status::Ok` 仍可能包含 warning，必须同时检查 `diagnostics`。`Ambiguous` 在保留模式下会产生首选文本。选用候选不会自动清除原诊断；严格模式仍会拒绝它们，需要调用方在明确审阅和消除诊断后再执行严格转换。

`losses` 包含 `source-distinction` 与 `unencoded-boundary`：简化 Hapin 文本不一定能表示全部原文拼写和字母边界，后续转换应保留结构化分析。

## Unicode 与 JSON

所有位置为与 JavaScript 相同的 **UTF-16 code unit** `[start, end)`，不是 UTF-8 字节或 Unicode 标量索引。例如 `a😀y` 中 `y` 的位置为 `[3, 4)`。直接使用 `unit.raw` 取得原文，不要把这些数字用于 Rust 字符串字节切片。Rust `&str` 必须是合法 UTF-8，因此不表示 JavaScript 单独存在的未配对 surrogate。

分析、转换结果、选项、单元和诊断都支持 Serde。JSON 使用 TypeScript 对应的 camelCase 字段和枚举值，例如 `rulesVersion`、`sourceLetter`、`maxCandidates`、`context-v1`、`ipa-cna`。Rust 中分析位于 `result.analysis`，序列化时则展开到与 JavaScript 一致的顶层对象。

## 旧版兼容接口

| TypeScript | Rust |
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

七个转换函数默认接受 `&str`、返回 `String`；对应的 `_with_options` 函数额外接受 `LegacyOptions { clean, easy }`，默认 `clean: true, easy: false`。`easy` 仅作用于 Arabic → Hapin。

`clean: false` 保留旧接口按来源区分的 U+200B 序列化行为；Array 函数按这些历史分隔符返回 `Vec<String>`，不等同于完整分词。获取明确字母单元请用 `Analysis.units`。

Hapin 旧接口保留旧切分：`transform_hapin_to_arabic("ngh")` 返回 `ڭح`；使用 `convert` 才应用上下文规则。原文字母边界、弱音按词作用域、IPA、标点、引号与旧空格行为，都与已修复的 TypeScript 5.0.0 一致。

原型项目的三个转换模块路径和 `utils::utf8` 辅助函数仍可使用。旧辅助函数按 Unicode 标量计数，区别于结构化 API 的 UTF-16 位置。原型中未完成的扫描器和 Hello World 程序已替换为库实现。

## 开发与验证

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo build --locked --release
cargo doc --locked --no-deps
cargo run --locked --example convert
```

CI 在 Linux、macOS、Windows 上运行格式检查、Clippy、测试、文档示例和 release 构建。英文 README 中的 Rust 示例作为 doctest 编译执行。

已提交 **2,325 个完整结构化/API 对照案例**及 **1,600 个确定性随机差异案例**，由真实 TypeScript 5.0.0 执行生成，覆盖全部方向、兼容接口与数组、规则元数据、严格模式、候选、UTF-16 位置、弱音、引号、来源差异和序列化。另有 Rust 原生的非法输入及长字符串测试。来源版本、提交与随机种子记录在 `tests/fixtures/metadata.json`。

维护者如需重新生成快照，先构建干净的 hapin-utils 5.0.0，再执行：

```sh
node scripts/generate-parity-fixtures.cjs /absolute/path/to/hapin-utils
cargo test --locked
```

快照变更必须独立审阅，不能只通过重新生成快照消除测试失败。此参考快照生成脚本不参与 Cargo 构建或 CI；原生转换与测试均用 Rust，新增 WASM 绑定另有基于 Node 的互操作测试。

## 主分支自动发布与 WASM

推送 `main` 会触发 `.github/workflows/release.yml`：复用 PR 的跨平台检查，构建并测试 WASM，再发布尚不存在的版本。支持 `workflow_dispatch`，配置好凭据后可手动重跑，无须制造空提交。

首次配置：

1. 登录 [crates.io](https://crates.io/me)，验证邮箱并创建 API token。首次发布必须允许创建 `hapin-utils-rs`；后续需要该 crate 的版本发布权限。设置有效期并按需轮换。
2. 打开[仓库 Actions secrets](https://github.com/ha-pin/hapin-utils-rs/settings/secrets/actions)，新增 Repository secret **`CARGO_REGISTRY_TOKEN`**，值填入 token。不要提交到 Git 或输出到日志。
3. 推送版本变更到 main，或者在 Actions → **Publish crate and WASM release** → **Run workflow** 手动执行。GitHub Release 使用内置 `GITHUB_TOKEN`，仅 Release job 有 `contents: write`，不需要额外 GitHub PAT。

缺少 token 时，workflow 会发出 warning，并在运行摘要中明确记录 crate **尚未发布**；WASM Release 仍可独立完成。配置好 secret 后重跑即可补发 crate，不会覆盖已存在的 WASM Release。

版本号取自 `Cargo.toml`。发布新版本时修改 package version，执行 `cargo check` 更新 `Cargo.lock`，将二者一起提交 main。crates.io 的已有版本不会重复发布，GitHub 已公开 Release 的文件也不会被同版本后续提交覆盖。修改发布内容应提高版本号。预发布版本会生成 prerelease Release。部分发布失败时应重跑原 workflow；已有标签必须指向原运行的提交，不能偷偷移动标签。

Release 标签为 `v<version>`，资产包括：

- `hapin-utils-rs-<version>-wasm-web.tar.gz`
- `hapin-utils-rs-<version>-wasm-nodejs.tar.gz`
- `hapin-utils-rs-<version>-wasm-bundler.tar.gz`
- `SHA256SUMS`

每个压缩包包含匹配的 `.wasm`、JavaScript 胶水代码、TypeScript 声明、package metadata、README 与 LICENSE。选择对应目标后完整解压，保持 JS 和 WASM 文件在一起。本流程只上传 GitHub Release，不发布 npm 包。

浏览器使用 web 包，通过 HTTP 服务访问，并为 WASM 配置 `application/wasm` MIME：

```js
import init, { convert, analyzeHapin, segmentationRules } from './wasm-web/hapin_utils_rs.js';
await init();
console.log(convert('арзанға', { from: 'cyrillic', to: 'arabic' }).text);
console.log(analyzeHapin('ngh', {}).alternatives);
console.log(segmentationRules());
```

Node.js 使用 nodejs 包：

```js
const { convert } = require('./wasm-nodejs/hapin_utils_rs.js');
console.log(convert('арзанға', { from: 'cyrillic', to: 'arabic' }).text);
```

WASM 暴露 `convert`、`analyzeHapin`、`segmentationRules()`，以及 TypeScript 同名的七个 `transform*` 和三个 Array 函数；结果对象结构一致。`convert` 接受文本或分析对象。WASM 绑定互操作测试使用 Node；默认原生 crate 不依赖 JS 运行时，WASM 依赖仅通过可选 `wasm` feature 引入。

本地验证：

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
python3 scripts/build-wasm.py
node scripts/test-wasm.cjs
python3 scripts/package-wasm.py
python3 -m unittest discover -s scripts -p 'test_*.py'
```

Node 和 web 真实 WASM 绑定分别执行 2,325 个完整对照案例，通过后才上传构建资产。仅真实 404 被当作版本不存在，其他网络／权限故障会失败；资产损坏或标签指向其他提交时也会停止。Release 先以草稿组装和校验，再公开，标签作者仍使用 Herbert He。

## 许可

MIT。维护者 Herbert He <Herbert.He0229@gmail.com>。
