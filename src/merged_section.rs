use std::{collections::HashMap, cmp::Ordering};

use crate::{
    chunk::Chunk,
    context::Context,
    elf::{SHF_COMPRESSED, SHF_GROUP, SHF_MERGE, SHF_STRINGS},
    output::get_output_name,
    section_fragment::SectionFragment, utils::align_to,
};

#[allow(dead_code)]
pub struct MergedSection {
    pub chunk: *mut Chunk,
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
            chunk: Box::leak(Box::new(chunk)),
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

    pub fn assign_offsets(&mut self){
        struct Fragment{
            key: String,
            val: *mut SectionFragment
        }

        let mut fragments: Vec<Fragment> = vec![];
        for (key,sec_f)  in &self.map{
            fragments.push(Fragment { key: String::from(key), val: *sec_f })
        }

        fragments.sort_by(|a: &Fragment, b:&Fragment| -> Ordering{
            if unsafe{a.val.as_ref().unwrap().p2_align != b.val.as_ref().unwrap().p2_align}{
                return Ordering::Less
            }
            if a.key != b.key {
                return Ordering::Less
            }
            Ordering::Less
        });

        let mut offset = 0u64;
        let mut p2_align = 0u64;

        for frag in fragments{
            offset = align_to(offset, 1<<unsafe { frag.val.as_ref().unwrap().p2_align });
            unsafe { frag.val.as_mut().unwrap().offset = offset as u32 };
            offset += frag.key.len() as u64;
            if p2_align < unsafe { frag.val.as_ref().unwrap().p2_align as u64} {
                p2_align = unsafe { frag.val.as_ref().unwrap().p2_align  as u64};
            }
        }

        unsafe { self.chunk.as_mut().unwrap().shdr.size = align_to(offset, 1<<p2_align) };
        unsafe { self.chunk.as_mut().unwrap().shdr.addr_align = 1 << p2_align };
    }

    pub fn copy_buf(&self, ctx: &mut Context){
        let buf = &mut ctx.buf[unsafe { self.chunk.as_ref().unwrap().shdr.offset as usize }..];
        for (key, _sec_f) in &self.map{
            buf.copy_from_slice(key.as_bytes());
        }
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
            if &name == unsafe { &osec.as_ref().unwrap().chunk.as_ref().unwrap().name }
                && flags == unsafe { osec.as_ref().unwrap().chunk.as_ref().unwrap().shdr.flags}
                && ty == unsafe { osec.as_ref().unwrap().chunk.as_ref().unwrap().shdr.shdr_type}
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