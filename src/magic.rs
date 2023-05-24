#[allow(dead_code)]
pub fn check_magic(contents: &Vec<u8>) -> bool {
    let prefix = std::str::from_utf8(&contents[..8]).unwrap();
    if prefix == "\\177ELF"{
        true
    } else {
        false
    }
}
