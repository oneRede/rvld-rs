use crate::elf::Shdr;

#[allow(dead_code)]
pub struct Chunk {
    pub name: String,
    pub shdr: Shdr,
}

impl Chunk {
    #[allow(dead_code)]
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
