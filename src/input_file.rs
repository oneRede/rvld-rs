use crate::elf::{Ehdr, Shdr, Sym};
use crate::elf::{EHDR_SIZE, SHDR_SIZE, SYM_SIZE};
use crate::file::ElfFile;
use crate::magic::check_magic;
use crate::symbol::Symbol;
use crate::utils::{fatal, read};

const SHN_XINDEX: u16 = 0xffff;

#[allow(dead_code)]
pub struct InputFile<'a> {
    pub file: ElfFile<'a>,
    pub elf_sections: Vec<Shdr>,
    pub elf_syms: Vec<Sym>,
    pub first_global: Option<i64>,
    pub sh_strtab: Option<&'a [u8]>,
    pub symbol_strtab: Option<&'a [u8]>,
    pub is_alive: bool,
    pub symbols: Vec<Symbol<'a>>,
    pub local_symbols: Vec<Symbol<'a>>
}

#[allow(dead_code)]
pub fn new_input_file(file: ElfFile) -> InputFile {
    let mut f = InputFile {
        file: file,
        elf_sections: Vec::new(),
        elf_syms: Vec::new(),
        first_global: None,
        sh_strtab: None,
        symbol_strtab: None,
        is_alive: false,
        symbols: vec![],
        local_symbols: vec![],
    };

    if f.file.contents.len() < EHDR_SIZE {
        fatal("file too small");
    }
    if !check_magic(f.file.contents) {
        fatal("not an ELF file")
    }
    let ehdr: Ehdr = read(f.file.contents);
    let contents = &f.file.contents[ehdr.sh_off as usize..];
    let shdr: Shdr = read(contents);

    let mut num_sections = ehdr.sh_num as i64;
    if num_sections == 0 {
        num_sections = shdr.size as i64;
    }

    f.elf_sections.push(shdr);
    for i in 0..(num_sections - 1) {
        let idx_shdr = (i + 1) as usize * SHDR_SIZE;
        let contents = &contents[idx_shdr..];
        let shdr: Shdr = read(contents);
        f.elf_sections.push(shdr);
        num_sections -= 1;
    }

    let mut sh_strtab = ehdr.sh_strndx as i64;
    if ehdr.sh_strndx == SHN_XINDEX {
        sh_strtab = shdr.link as i64;
    }
    f.sh_strtab = Some(f.get_bytes_from_idx(sh_strtab));
    return f;
}

impl<'a> InputFile<'a> {
    fn get_bytes_from_shdr(&self, shdr: &Shdr) -> &'a [u8] {
        let end = (shdr.offset + shdr.size) as usize;
        if self.file.contents.len() < end {
            fatal(&format!(
                "section header is out of range: {:?}",
                shdr.offset
            ));
        }

        &self.file.contents[shdr.offset as usize..end]
    }

    pub fn get_bytes_from_idx(&self, idx: i64) -> &'a [u8] {
        &self.get_bytes_from_shdr(&self.elf_sections[idx as usize])
    }

    #[allow(dead_code)]
    pub fn fillup_elf_syms(&mut self, shdr: Shdr) {
        let mut bs = self.get_bytes_from_shdr(&shdr);
        let nums = bs.len() / SYM_SIZE;
        for _ in 0..nums {
            self.elf_syms.push(read(&bs[..SYM_SIZE]));
            bs = &bs[SYM_SIZE..]
        }
    }

    #[allow(dead_code)]
    pub fn find_section(&self, ty: u32) -> Option<Shdr> {
        for i in 0..self.elf_sections.len() {
            let shdr = self.elf_sections[i];
            if shdr.shdr_type == ty {
                return Some(shdr);
            }
        }
        return None;
    }
}
