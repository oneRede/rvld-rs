use std::fs::File;
use std::io::prelude::*;

#[allow(dead_code)]
pub struct _ElfFile<'a> {
    pub name: &'a str,
    pub contents: &'static [u8]
}

#[allow(dead_code)]
pub fn must_new_file(file_name: &str) -> _ElfFile{
    let mut f = File::open(&file_name).unwrap();
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).unwrap();
    let buf = Box::new(buffer);

    _ElfFile{
        name: file_name,
        contents: Box::leak(buf),
    }
}
