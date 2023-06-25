use std::collections::HashMap;

use crate::{
    machine_type::{MachineType, MACHINE_TYPE_NONE},
    merged_section::MergedSection,
    object_file::ObjectFile,
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
    pub objs: Vec<*mut ObjectFile<'a>>,
    pub symbol_map: HashMap<&'static str, *mut Symbol<'a>>,
    pub merged_sections: Vec<*mut MergedSection>,
}

impl<'a> Context<'a> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Context {
            args: ContextArgs {
                output: "a.out".to_string(),
                emulation: MACHINE_TYPE_NONE,
                library_paths: vec![],
            },
            objs: vec![],
            symbol_map: HashMap::new(),
            merged_sections: vec![],
        }
    }
}
