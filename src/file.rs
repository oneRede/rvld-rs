use std::fs::{File, self};
use std::io::prelude::*;

#[allow(dead_code)]
pub struct ElfFile<'a> {
    pub name: &'a str,
    pub contents: &'static [u8],
    pub files: Vec<* const ElfFile<'a>>,
}

#[allow(dead_code)]
pub fn must_new_file(file_name: &str) -> ElfFile{
    let mut f = File::open(&file_name).unwrap();
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).unwrap();
    let buf = Box::new(buffer);

    ElfFile{
        name: file_name,
        contents: Box::leak(buf),
        files: vec![],
    }
}

// func OpenLibrary(filepath string) *File {
// 	contents, err := os.ReadFile(filepath)
// 	if err != nil {
// 		return nil
// 	}

// 	return &File{
// 		Name:     filepath,
// 		Contents: contents,
// 	}
// }

// func FindLibrary(ctx *Context, name string) *File {
// 	for _, dir := range ctx.Args.LibraryPaths {
// 		stem := dir + "/lib" + name + ".a"
// 		if f := OpenLibrary(stem); f != nil {
// 			return f
// 		}
// 	}

// 	utils.Fatal("library not found")
// 	return nil
// }

fn open_library(file_path: &str) -> ElfFile<'_>{
    let mut f = File::open(file_path).unwrap();
    let mut contents = Vec::new();

    f.read_to_end(&mut contents).unwrap();
    let contents = Box::new(contents);
    
    ElfFile{
        name: file_path,
        contents: Box::leak(contents),
        files: vec![],

    }
}