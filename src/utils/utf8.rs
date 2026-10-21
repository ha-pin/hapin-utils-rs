//! Prototype compatibility helpers. Indices here count Unicode scalars, unlike API spans.
/// Return the scalar at `idx`. Panics if the index is outside the string.
pub fn get_char(input: &str, idx: u32) -> char {
    input
        .chars()
        .nth(idx as usize)
        .expect("character index out of bounds")
}
/// Count Unicode scalars, preserving the prototype helper's behavior.
pub fn get_string_length(input: &str) -> u32 {
    input
        .chars()
        .count()
        .try_into()
        .expect("string has more than u32::MAX characters")
}
