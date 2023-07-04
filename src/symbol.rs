use crate::context::Context;
use crate::elf::Sym;
use crate::section_fragment::SectionFragment;
use crate::{input_section::InputSection, object_file::ObjectFile};

#[allow(dead_code)]
pub struct Symbol<'a> {
    pub object_file: Option<*mut ObjectFile<'a>>,
    pub name: &'a str,
    pub value: u64,
    pub symidx: i32,
    pub got_tp_id: i32,
    
    pub input_section: Option<*mut InputSection<'a>>,
    pub section_fragment: Option<*mut SectionFragment>,
    
    pub flags: u32,
}

#[allow(dead_code)]
impl<'a> Symbol<'a> {
    pub fn new(name: &'static str) -> Symbol<'a> {
        Symbol {
            name: name,
            value: 0,
            symidx: 0,
            got_tp_id: 0,
            object_file: None,
            input_section: None,
            section_fragment: None,
            flags: 0,
        }
    }

    pub fn get_symbol_by_name(ctx: &Context<'a>, name: &str) -> *mut Symbol<'a> {
        ctx.symbol_map[name]
    }

    pub fn elf_sym(&self) -> Sym {
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

    pub fn clear(&self) {
        // nothing
    }

    pub fn set_input_section(&'a mut self, isec: *mut InputSection<'a>){
        self.input_section = Some(isec);
        self.section_fragment = None;
    }

    pub fn set_section_fragment(&'a mut self, frag: *mut SectionFragment){
        self.input_section = None;
        self.section_fragment = Some(frag);
    }

    pub fn get_addr(&self) -> u64 {
        if self.section_fragment.is_none() {
            return unsafe { self.section_fragment.unwrap().as_ref().unwrap().get_addr()  + self.value};
        };
        if self.input_section.is_none() {
            return unsafe { self.input_section.unwrap().as_ref().unwrap().get_addr()  + self.value};
        };
        return self.value
    }

    pub fn get_got_tp_addr(&self, ctx: &Context) -> u64{
        ctx.got.chunk.shdr.addr + (self.got_tp_id as u64 * 8u64)
    }
}
