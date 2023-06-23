use std::vec;

use crate::{
    elf::{Shdr, SHT_GROUP, SHT_NULL, SHT_REL, SHT_RELA, SHT_STRTAB, SHT_SYMTAB, SHT_SYMTAB_SHNDX},
    file::ElfFile,
    input_file::{new_input_file, InputFile},
    input_section::InputSection,
    mergeablesection::MergeableSection,
};

#[allow(dead_code)]
pub struct ObjectFile<'a> {
    pub input_file: *mut InputFile<'a>,
    pub symtab_sec: Option<Shdr>,
    pub symbol_shndx_sec: Vec<u32>,
    pub input_sections: Vec<InputSection<'a>>,
    pub mergeable_sections: Vec<MergeableSection>,
}

#[allow(dead_code)]
pub fn new_object_file(elf_file: ElfFile, _is_alive: bool) -> ObjectFile {
    let input_file = new_input_file(elf_file);
    let object_file = ObjectFile {
        input_file: input_file,
        symtab_sec: None,
        symbol_shndx_sec: vec![],
        input_sections: vec![],
        mergeable_sections: vec![],
    };
    object_file
}

#[allow(dead_code)]
impl<'a> ObjectFile<'a> {
    pub fn parse(&mut self) {
        self.symtab_sec =
            unsafe { self.input_file.as_mut().unwrap() }.find_section(SHT_SYMTAB as u32);
        match self.symtab_sec {
            None => {}
            Some(shdr) => {
                unsafe { self.input_file.as_mut().unwrap() }.first_global = Some(shdr.info as i64);
                unsafe { self.input_file.as_mut().unwrap() }.fillup_elf_syms(shdr);
                unsafe { self.input_file.as_mut().unwrap() }.symbol_strtab = Some(
                    unsafe { self.input_file.as_ref().unwrap() }
                        .get_bytes_from_idx(shdr.link as i64),
                );
            }
        }
    }

    pub fn initialize_sections(&'a mut self) {
        for i in 0..unsafe { self.input_file.as_ref().unwrap() }
            .elf_sections
            .len()
        {
            let shdr = unsafe { self.input_file.as_ref().unwrap() }.elf_sections[i];
            match shdr.shdr_type {
                SHT_GROUP | SHT_SYMTAB | SHT_STRTAB | SHT_RELA | SHT_NULL | SHT_REL => {
                    break;
                }
                SHT_SYMTAB_SHNDX => {
                    self.fillup_symtab_shndx_sec(shdr);
                }
                _ => {
                    self.input_sections[i] = InputSection::new(self, i);
                }
            }
        }
    }

    pub fn fillup_symtab_shndx_sec(&mut self, shdr: Shdr) {
        let bs = unsafe { self.input_file.as_ref().unwrap() }.get_bytes_from_shdr(&shdr);
        self.symbol_shndx_sec = bs.into_iter().map(|n| *n as u32).collect();
    }
}
