use crate::elf::{Ehdr, Shdr};
use crate::elf::{EHDR_SIZE, SHDR_SIZE};
use crate::file::_ElfFile;
use crate::magic::check_magic;
use crate::utils::{fatal, read_ehdr, read_shdr};

#[allow(dead_code)]
pub struct InputFile<'a> {
    pub file: _ElfFile<'a>,
    pub elf_sections: Vec<Shdr>,
}

#[allow(dead_code)]
pub fn new_input_file(file: _ElfFile) -> InputFile {
    let mut f = InputFile {
        file: file,
        elf_sections: Vec::new(),
    };

    if f.file.contents.len() < EHDR_SIZE {
        fatal("file too small");
    }
    if !check_magic(f.file.contents) {
        fatal("not an ELF file")
    }
    let ehdr: Ehdr = read_ehdr(f.file.contents);
    let contents = &f.file.contents[ehdr.sh_off as usize..];
    let shdr: Shdr = read_shdr(contents);

    let mut num_sections = ehdr.sh_num as i64;
    if num_sections == 0 {
        num_sections = shdr.size as i64;
    }

    for _ in 0..num_sections {
        let contents = &contents[SHDR_SIZE..];
        let shdr: Shdr = read_shdr(contents);
        f.elf_sections.push(shdr);
        num_sections -= 1;
    }
    return f;
}
