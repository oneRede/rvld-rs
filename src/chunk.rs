use crate::{context::Context, elf::Shdr};

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd)]
pub struct Chunk {
    pub name: String,
    pub shdr: Shdr,
    pub shndx: i64,
}

#[allow(dead_code)]
pub trait Chunker {
    fn get_name(&self) -> String;
    fn get_shdr(&self) -> Shdr;
    fn update_shdr(_ctx: Context);
    fn get_shndx(&self) -> i64;
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
            shndx: 0
        }
    }
}

impl Chunker for Chunk {
    fn get_name(&self) -> String {
        String::from(&self.name)
    }
    fn get_shdr(&self) -> Shdr {
        self.shdr
    }

    fn update_shdr(_ctx: Context) {
        
    }

    fn get_shndx(&self) -> i64 {
        self.shndx
    }

    fn copy_buf(&self, _ctx: Context) {}
}