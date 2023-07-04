use crate::{chunk::Chunk, input_section::InputSection, elf::{Shdr, SHT_NOBITS, SHF_GROUP, SHF_COMPRESSED, SHF_LINK_ORDER}, context::Context, output::get_output_name};

#[allow(dead_code)]
pub struct OutputSection<'a> {
    pub chunk: Chunk,
    pub members: Vec<*mut InputSection<'a>>,
    pub idx: u32,
}

#[allow(dead_code)]
impl<'a> OutputSection<'a> {
    pub fn new(name: String, ty: u32, flags: u64, idx: u32) -> Self {
        let mut shdr = Shdr::new();
        shdr.shdr_type = ty;
        shdr.flags = flags;

        let mut chunk = Chunk::new();
        chunk.shdr = shdr;
        chunk.name = name;

        Self { chunk: chunk, members: vec![], idx: idx }

    }

    pub fn copy_buf(&self, ctx: Context<'a>){
        if self.chunk.shdr.shdr_type == SHT_NOBITS{
            return
        }

        let base = &ctx.buf[self.chunk.shdr.offset as usize..];
        let base = Box::leak(base.into_iter().map(|n| -> u8 {*n}).collect());
        for isec in &self.members{
            unsafe { isec.as_mut().unwrap().write_to(&ctx, &mut base[isec.as_ref().unwrap().offset as usize..]) };
        }
    }

    pub fn get_output_section(mut ctx: Context<'a>, mut name:String, ty: u64, mut flags: u64) -> *mut OutputSection{
        name = get_output_name(&name, flags);
        flags = flags &! SHF_GROUP &! SHF_COMPRESSED &! SHF_LINK_ORDER;

        let find = || -> Option<*mut OutputSection<'_>>{
            for osec in &ctx.output_sections{
                if name == unsafe { osec.as_ref().unwrap() }.chunk.name && ty == unsafe { osec.as_ref().unwrap().chunk.shdr.shdr_type.into() } 
                && flags == unsafe { osec.as_ref().unwrap().chunk.shdr.flags } {
                    return Some(*osec)
                }
            }
            None
        };

        let osec = find();
        if !osec.is_none() {
            return osec.unwrap()
        }
        let os_len = ctx.output_sections.len() as u32;
        let osec: *mut OutputSection = Box::leak(Box::new(Self::new(String::from(&name), ty as u32, flags, os_len)));
        ctx.output_sections.push(osec);
        return osec
    }
}
