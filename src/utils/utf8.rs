use utf8_slice;

pub fn get_char (str: &str, idx: u32) -> char {
    let oc = utf8_slice::slice(str, idx.try_into().unwrap(), (idx + 1).try_into().unwrap()).chars().next();
    let c = match oc {
        Some(oc) => {
            oc
        },
        None => todo!(),
    };

    return c;
}

pub fn get_string_length (s: &str) -> u32 {
	utf8_slice::len(s) as u32
}