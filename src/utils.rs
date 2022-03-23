pub fn get_char (str: &str, idx: usize) -> char {
    let oc = utf8_slice::slice(str, idx, idx + 1).chars().next();
    let c = match oc {
        Some(oc) => {
            oc
        },
        None => todo!(),
    };

    return c;
}