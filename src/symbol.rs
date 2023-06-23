use crate::context::Context;
use crate::elf::Sym;
use crate::{input_section::InputSection, object_file::ObjectFile};

#[allow(dead_code)]
pub struct Symbol<'a> {
    object_file: Option<*mut ObjectFile<'a>>,
    input_section: Option<InputSection<'a>>,
    name: &'static str,
    value: u64,
    symidx: i32,
}

#[allow(dead_code)]
impl<'a> Symbol<'a> {
    fn new(name: &'static str) -> Symbol<'a> {
        Symbol {
            object_file: None,
            input_section: None,
            name: name,
            value: 0,
            symidx: 0,
        }
    }

    fn get_symbol_by_name(&self, ctx: Context<'a>, name: &str) -> *const Symbol<'a> {
        &ctx.symbol_map[name] as *const Symbol<'a>
    }

    fn elf_sym(&self) -> Sym {
        assert!(
            self.symidx
                < unsafe {
                    (self.object_file.unwrap().as_ref().unwrap())
                        .input_file
                        .as_ref()
                        .unwrap()
                }
                .elf_syms
                .len() as i32
        );
        unsafe {
            (self.object_file.unwrap().as_ref().unwrap())
                .input_file
                .as_ref()
                .unwrap()
        }
        .elf_syms[self.symidx as usize]
    }

    fn clear(&self) {
        // nothing
    }
}
