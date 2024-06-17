use crate::{
    chunk::Chunk,
    context::Context,
    elf::{Shdr, SHDR_SIZE},
    utils::write,
};

#[allow(dead_code)]
pub struct OutputShdr {
    pub chunk: Chunk,
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

    pub fn update_shdr(&mut self, _ctx: Context) {
        self.chunk.shdr.size = 1 * SHDR_SIZE as u64;
    }

    pub fn copy_buf(&self, mut ctx: Context) {
        let base = &mut ctx.buf[*(&self.chunk.shdr.offset) as usize..];
        write(base, Shdr::new());
    }
}
