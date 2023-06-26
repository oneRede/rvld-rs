#[allow(dead_code)]
pub fn check_magic(contents: &[u8]) -> bool {
    let prefix = std::str::from_utf8(&contents[..4]).unwrap();
    if prefix == "\u{7f}ELF" {
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn write_magic(contents: &mut [u8]){
    contents.copy_from_slice("\u{7f}ELF".as_bytes())
}