use hapin_utils_rs::{convert, ConvertOptions, Source, Target};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = convert(
        "арзанға",
        ConvertOptions::new(Source::Cyrillic, Target::Arabic),
    )?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
