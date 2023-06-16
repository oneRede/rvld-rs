use crate::{
    elf::Shdr,
    file::ElfFile,
    input_file::{new_input_file, InputFile},
};

const SHT_SYMTAB: u16 = 2;

#[allow(dead_code)]
pub struct ObjectFile<'a> {
    pub input_file: InputFile<'a>,
    pub symtab_sec: Option<Shdr>,
}

#[allow(dead_code)]
pub fn new_object_file(elf_file: ElfFile) -> ObjectFile {
    let input_file = new_input_file(elf_file);
    let object_file = ObjectFile {
        input_file: input_file,
        symtab_sec: None,
    };
    object_file
}

impl<'a> ObjectFile<'a> {
    #[allow(dead_code)]
    pub fn parse(&mut self) {
        self.symtab_sec = self.input_file.find_section(SHT_SYMTAB as u32);
        match self.symtab_sec {
            None => {}
            Some(shdr) => {
                self.input_file.first_global = Some(shdr.info as i64);
                self.input_file.fillup_elf_syms(shdr);
                self.input_file.symbol_strtab =
                    Some(self.input_file.get_bytes_from_idx(shdr.link as i64));
            }
        }
    }
}
