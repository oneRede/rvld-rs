use crate::{chunk::Chunk, input_section::InputSection, elf::{Shdr, SHT_NOBITS, SHF_GROUP, SHF_COMPRESSED, SHF_LINK_ORDER}, context::Context, output::get_output_name};

#[allow(dead_code)]
pub struct OutputSection<'a> {
    pub chunk: Chunk,
    pub members: Vec<*mut InputSection<'a>>,
    pub idx: u32,
}

// func NewOutputSection(
// 	name string, typ uint32, flags uint64, idx uint32) *OutputSection {
// 	o := &OutputSection{Chunk: NewChunk()}
// 	o.Name = name
// 	o.Shdr.Type = typ
// 	o.Shdr.Flags = flags
// 	o.Idx = idx
// 	return o
// }

// func (o *OutputSection) CopyBuf(ctx *Context) {
// 	if o.Shdr.Type == uint32(elf.SHT_NOBITS) {
// 		return
// 	}

// 	base := ctx.Buf[o.Shdr.Offset:]
// 	for _, isec := range o.Members {
// 		isec.WriteTo(ctx, base[isec.Offset:])
// 	}
// }

// func GetOutputSection(
// 	ctx *Context, name string, typ, flags uint64) *OutputSection {
// 	name = GetOutputName(name, flags)
// 	flags = flags &^ uint64(elf.SHF_GROUP) &^
// 		uint64(elf.SHF_COMPRESSED) &^ uint64(elf.SHF_LINK_ORDER)

// 	find := func() *OutputSection {
// 		for _, osec := range ctx.OutputSections {
// 			if name == osec.Name && typ == uint64(osec.Shdr.Type) &&
// 				flags == osec.Shdr.Flags {
// 				return osec
// 			}
// 		}
// 		return nil
// 	}

// 	if osec := find(); osec != nil {
// 		return osec
// 	}

// 	osec := NewOutputSection(name, uint32(typ), flags,
// 		uint32(len(ctx.OutputSections)))
// 	ctx.OutputSections = append(ctx.OutputSections, osec)
// 	return osec
// }

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

    pub fn copy_buf(&self, ctx: Context){
        if self.chunk.shdr.shdr_type == SHT_NOBITS{
            return
        }

        let base = &ctx.buf[self.chunk.shdr.offset as usize..];
        let base = Box::leak(base.into_iter().map(|n| -> u8 {*n}).collect());
        for isec in &self.members{
            unsafe { isec.as_mut().unwrap().write_to(&ctx, &mut base[isec.as_ref().unwrap().offset as usize..]) };
        }
    }

    pub fn get_output_section(ctx: Context, mut name:String, ty: u64, mut flags: u64){
        name = get_output_name(&name, flags);
        flags = flags &! SHF_GROUP &! SHF_COMPRESSED &! SHF_LINK_ORDER;

        let find = || {
            // for osec in ctx.
        };
    }


}
