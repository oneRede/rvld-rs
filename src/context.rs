use std::collections::HashMap;

use crate::{
    machine_type::{MachineType, MACHINE_TYPE_NONE},
    merged_section::MergedSection,
    object_file::ObjectFile,
    symbol::Symbol, output_ehdr::OutputEhdr, chunk::{Chunk}, elf::{Sym}, output_shdr::OutputShdr, output_phdr::OutputPhdr,
};

#[allow(dead_code)]
pub struct ContextArgs {
    pub output: String,
    pub emulation: MachineType,
    pub library_paths: Vec<String>,
}

#[allow(dead_code)]
pub struct Context<'a> {
    pub args: ContextArgs,
    pub buf: Vec<u8>,
    pub ehdr: OutputEhdr,
    pub shdr: OutputShdr,
    pub phdr: OutputPhdr,
    pub tp_addr: u64,
    pub objs: Vec<*mut ObjectFile<'a>>,
    pub chunks: Option<Vec<*mut Chunk>>,
    pub symbol_map: HashMap<&'static str, *mut Symbol<'a>>,
    pub merged_sections: Vec<*mut MergedSection>,
    pub internal_obj: Option<*mut ObjectFile<'a>>,
    pub internal_esyms: Vec<Sym>,
}

#[allow(dead_code)]
impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            args: ContextArgs {
                output: "a.out".to_string(),
                emulation: MACHINE_TYPE_NONE,
                library_paths: vec![],
            },
            buf: vec![],
            ehdr: OutputEhdr::new(),
            shdr: OutputShdr::new(),
            phdr: OutputPhdr::new(),
            tp_addr: 0,
            objs: vec![],
            chunks: None,
            symbol_map: HashMap::new(),
            merged_sections: vec![],
            internal_esyms: vec![],
            internal_obj: None
        }
    }
}
