use std::{collections::HashMap};

use crate::{
    chunk::Chunk,
    context::Context,
    elf::{SHF_COMPRESSED, SHF_GROUP, SHF_MERGE, SHF_STRINGS},
    output::get_output_name,
    section_fragment::SectionFragment,
};

#[allow(dead_code)]
pub struct MergedSection {
    pub chunk: Chunk,
    pub map: HashMap<String, *mut SectionFragment>,
}

#[allow(dead_code)]
impl MergedSection {
    pub fn new(name: String, flags: u64, ty: u32) -> Self {
        let mut chunk = Chunk::new();
        chunk.name = name;
        chunk.shdr.flags = flags;
        chunk.shdr.shdr_type = ty;

        MergedSection {
            chunk: chunk,
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, p2_align: u32) -> Option<*mut SectionFragment>{
        let sf = self.map.remove(key);
        match sf {
            Some(v) => {
                if (unsafe { &(*v) }).p2_align < p2_align {
                    let frag = Box::new(SectionFragment::new(self as *mut MergedSection));
                    let frag = Box::leak(frag);
                    frag.p2_align = p2_align;
                    self.map.insert(String::from(key), frag);
                }
            }
            None => {
                let frag = Box::new(SectionFragment::new(self as *mut MergedSection));
                let frag = Box::leak(frag);
                self.map.insert(String::from(key), frag);
            }
        }
        sf
    }
}

#[allow(dead_code)]
pub fn get_merged_section_instance<'a>(
    ctx: &'a mut Context,
    name: &str,
    ty: u32,
    flags: u64,
) -> Option<*mut MergedSection> {
    let name = get_output_name(&name, flags);
    let flags = flags & SHF_GROUP & SHF_MERGE & SHF_STRINGS & SHF_COMPRESSED;

    let find = || -> Option<*mut MergedSection> {
        for osec in &ctx.merged_sections {
            if &name == unsafe { &osec.as_ref().unwrap().chunk.name }
                && flags == unsafe { osec.as_ref().unwrap()}.chunk.shdr.flags
                && ty == unsafe { osec.as_ref().unwrap()}.chunk.shdr.shdr_type
            {
                return Some(osec.clone());
            }
        }
        return None;
    };

    let osce = find();
    match osce {
        Some(ms) => return Some(ms as *mut MergedSection),
        None => return None,
    }
}
