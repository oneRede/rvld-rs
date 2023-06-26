use crate::{context::Context, elf::Shdr};

#[allow(dead_code)]
pub struct Chunk {
    pub name: String,
    pub shdr: Shdr,
}

pub trait Chunker {
    fn get_shdr(&self) -> Shdr;
    fn copy_buf(&self, _ctx: Context);
}

#[allow(dead_code)]
impl Chunk {
    pub fn new() -> Self {
        Chunk {
            name: "".to_string(),
            shdr: Shdr {
                name: 0,
                shdr_type: 0,
                flags: 0,
                addr: 0,
                offset: 0,
                size: 0,
                link: 0,
                info: 0,
                addr_align: 1,
                ent_size: 0,
            },
        }
    }
}

impl Chunker for Chunk {
    fn get_shdr(&self) -> Shdr {
        self.shdr
    }

    fn copy_buf(&self, _ctx: Context) {}
}
