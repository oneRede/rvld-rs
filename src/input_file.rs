use crate::elf::{Ehdr, Shdr, Sym};
use crate::elf::{Sym_Size, EHDR_SIZE, SHDR_SIZE};
use crate::file::_ElfFile;
use crate::magic::check_magic;
use crate::utils::{fatal, read_ehdr, read_shdr, read_sym};

const SHN_XINDEX: u16 = 0xffff;

#[allow(dead_code)]
pub struct InputFile<'a> {
    pub file: _ElfFile<'a>,
    pub elf_sections: Vec<Shdr>,
}
#[allow(dead_code)]
pub struct _InputFile<'a> {
    pub file: _ElfFile<'a>,
    pub elf_sections: Vec<Shdr>,
    pub elf_syms: Vec<Sym>,
    pub first_global: Option<i64>,
    pub sh_strtab: Option<&'a [u8]>,
    pub symbol_strtab: Option<&'a [u8]>,
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

#[allow(dead_code)]
pub fn _new_input_file(file: _ElfFile) -> _InputFile {
    let mut f = _InputFile {
        file: file,
        elf_sections: Vec::new(),
        elf_syms: Vec::new(),
        first_global: None,
        sh_strtab: None,
        symbol_strtab: None,
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

    let mut sh_strtab = ehdr.sh_strndx as i64;
    if ehdr.sh_strndx == SHN_XINDEX {
        sh_strtab = shdr.link as i64;
    }
    f.sh_strtab = Some(f.get_bytes_from_idx(sh_strtab));
    return f;
}

impl<'a> _InputFile<'a> {
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

    fn get_bytes_from_idx(&self, idx: i64) -> &'a [u8] {
        &self.get_bytes_from_shdr(&self.elf_sections[idx as usize])
    }

    #[allow(dead_code)]
    fn fillup_elf_syms(&mut self, shdr: Shdr) {
        let mut bs = self.get_bytes_from_shdr(&shdr);
        let nums = bs.len() / Sym_Size;
        for _ in 0..nums {
            self.elf_syms.push(read_sym(&bs[..Sym_Size]));
            bs = &bs[Sym_Size..]
        }
    }

    #[allow(dead_code)]
    fn find_section(&self, ty: u32) -> Option<Shdr> {
        for i in 0..self.elf_sections.len() {
            let shdr = self.elf_sections[i];
            if shdr.shdr_type == ty {
                return Some(shdr);
            }
        }
        return None;
    }
}
