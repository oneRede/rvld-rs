use crate::{
    chunk::{Chunk, Chunker},
    context::Context,
    elf::Shdr,
    utils::write,
};

#[allow(dead_code)]
pub struct OutputShdr {
    chunk: Chunk,
}

#[allow(dead_code)]
impl OutputShdr {
    pub fn new() -> Self {
        let mut shdr = Shdr::new();
        shdr.addr_align = 8;

        let mut chunk = Chunk::new();
        chunk.shdr = shdr;
        Self { chunk: chunk }
    }

    pub fn update_shdr(&self, mut ctx: Context) {
        let base = &mut ctx.buf[*(&self.chunk.shdr.offset) as usize..];
        write(base, Shdr::new());

        for chunk in ctx.chunks.unwrap() {
            let shndx = unsafe { chunk.as_ref().unwrap().get_shndx() } as usize;
            if shndx > 0 {
                write(&mut base[shndx..], unsafe {
                    chunk.as_ref().unwrap().get_shdr()
                })
            }
        }
    }
}
