use std::fs::File;
use std::io::prelude::*;

#[allow(dead_code)]
pub struct ElfFile {
    pub name: String,
    pub contents: Vec<u8>,
}

pub struct _ElfFile {
    pub name: &'static str,
    pub contents: &'static [u8]
}

#[allow(dead_code)]
fn must_new_file(file_name: String) -> ElfFile {
    let mut f = File::open(&file_name).unwrap();
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).unwrap();
    ElfFile {
        name: file_name,
        contents: buffer,
    }
}

fn _must_new_file(file_name: &'static str) -> _ElfFile{
    let mut f = File::open(&file_name).unwrap();
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).unwrap();
    let buf = Box::new(buffer);

    _ElfFile{
        name: file_name,
        contents: Box::leak(buf),
    }
}
