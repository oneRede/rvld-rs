#[allow(dead_code)]
pub fn check_magic(contents: &[u8]) -> bool {
    let prefix = std::str::from_utf8(&contents[..4]).unwrap();
    if prefix == "\u{7f}ELF" {
        true
    } else {
        false
    }
}
