use std::collections::HashMap;

use crate::{
    chunk::Chunk,
    constent::{SHF_COMPRESSED, SHF_GROUP, SHF_MERGE, SHF_STRINGS},
    context::Context,
    output::get_output_name,
    section_fragment::SectionFragment,
};

#[allow(dead_code)]
pub struct MergedSection {
    chunk: Chunk,
    map: HashMap<String, *mut SectionFragment>,
}

impl MergedSection {
    #[allow(dead_code)]
    fn new(name: String, flags: u64, ty: u32) -> Self {
        let mut chunk = Chunk::new();
        chunk.name = name;
        chunk.shdr.flags = flags;
        chunk.shdr.shdr_type = ty;

        MergedSection {
            chunk: chunk,
            map: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn insert(&mut self, key: String, p2_align: u32) {
        let sf = self.map.remove(&key);
        match sf {
            Some(v) => {
                if (unsafe { &(*v) }).p2_align < p2_align {
                    let frag = Box::new(SectionFragment::new(self as *mut MergedSection));
                    let frag = Box::leak(frag);
                    frag.p2_align = p2_align;
                    self.map.insert(key, frag);
                }
            }
            None => {
                let frag = Box::new(SectionFragment::new(self as *mut MergedSection));
                let frag = Box::leak(frag);
                self.map.insert(key, frag);
            }
        }
    }
}

#[allow(dead_code)]
fn get_merged_section_instance<'a>(ctx: &'a mut Context, name: String, ty: u32, flags: u64) -> Option<&'a MergedSection>{
    let name = get_output_name(&name, flags);
    let flags = flags & SHF_GROUP & SHF_MERGE & SHF_STRINGS & SHF_COMPRESSED;

    let find = || -> Option<&MergedSection>{
        for osec in &ctx.merged_sections {
            if name == osec.chunk.name
                && flags == osec.chunk.shdr.flags
                && ty == osec.chunk.shdr.shdr_type
            {
                return Some(osec);
            }
        }
        return None
    };

    let osce = find();
    match osce {
        Some(ms) => {
            return Some(ms)
        },
        None => {return None}
    }
}
