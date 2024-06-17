use std::collections::HashMap;

use crate::{
    chunk::Chunk,
    elf::EF_RISCV_RVC,
    got_section::GotSection,
    machine_type::{MachineType, MACHINE_TYPE_NONE},
    merged_section::MergedSection,
    object_file::ObjectFile,
    output_ehdr::OutputEhdr,
    output_phdr::OutputPhdr,
    output_section::OutputSection,
    output_shdr::OutputShdr,
    symbol::Symbol,
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
    pub got: GotSection<'a>,

    pub tp_addr: u64,
    pub output_sections: *mut Vec<*mut OutputSection<'a>>,

    pub objs: Vec<*mut ObjectFile<'a>>,
    pub chunks: Option<*mut Vec<*mut Chunk>>,
    pub symbol_map: HashMap<&'static str, *mut Symbol<'a>>,
    pub merged_sections: Vec<*mut MergedSection>,
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
            got: GotSection::new(),

            tp_addr: 0,
            output_sections: Box::leak(Box::new(vec![])),

            objs: vec![],
            chunks: None,
            symbol_map: HashMap::new(),
            merged_sections: vec![],
        }
    }

    pub fn get_entry_addr(&self) -> u64 {
        for osec in unsafe { self.output_sections.as_ref().unwrap() } {
            if unsafe { &osec.as_ref().unwrap().chunk.as_ref().unwrap().name } == ".text" {
                return unsafe { osec.as_ref().unwrap().chunk.as_ref().unwrap().shdr.addr };
            }
        }
        return 0;
    }

    pub fn get_flags(&self) -> u32 {
        assert!(self.objs.len() > 0);
        let mut flags = unsafe {
            self.objs[0]
                .as_ref()
                .unwrap()
                .input_file
                .as_ref()
                .unwrap()
                .get_ehdr()
                .flags
        };
        for obj in &self.objs[1..] {
            if unsafe {
                obj.as_ref()
                    .unwrap()
                    .input_file
                    .as_ref()
                    .unwrap()
                    .get_ehdr()
                    .flags
            } & EF_RISCV_RVC
                != 0
            {
                flags |= EF_RISCV_RVC;
                break;
            }
        }
        return flags;
    }
}
