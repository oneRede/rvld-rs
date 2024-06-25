use std::fs::File;
use std::io::prelude::*;

use crate::context::Context;

#[allow(dead_code)]
pub struct ElfFile<'a> {
    pub name: &'a str,
    pub contents: &'static [u8],
    pub files: Vec<*const ElfFile<'a>>,
}

#[allow(dead_code)]
pub fn must_new_file(file_name: &str) -> ElfFile {
    let mut f = File::open(&file_name).unwrap();
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).unwrap();
    let buf = Box::new(buffer);

    ElfFile {
        name: file_name,
        contents: Box::leak(buf),
        files: vec![],
    }
}

#[allow(dead_code)]
fn open_library(file_path: &str) -> Option<ElfFile<'_>> {
    let f = File::open(file_path);
    if f.is_err() {
        return None
    }
    let mut contents = Vec::new();

    f.unwrap().read_to_end(&mut contents).unwrap();
    let contents = Box::new(contents);

    Some(ElfFile {
        name: file_path,
        contents: Box::leak(contents),
        files: vec![],
    })
}

#[allow(dead_code)]
pub fn find_library<'a>(ctx: &Context, name: &'a str) -> Option<ElfFile<'a>> {
    for dir in &ctx.args.library_paths {
        let stem = String::from(dir) + "/lib" + name + ".a";
        let stem = Box::leak(Box::new(stem));
        let f = open_library(stem);
        match f {
            Some(f) => {
                return Some(f);
            }
            None => {
                return None;
            }
        }
    }
    None
}
