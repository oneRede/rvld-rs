use std::mem;

#[allow(dead_code)]
pub const EHDR_SIZE: usize = mem::size_of::<Ehdr>();
#[allow(dead_code)]
pub const SHDR_SIZE: usize = mem::size_of::<Shdr>();

#[allow(dead_code)]
pub struct Ehdr {
    ident: [u8; 16],
    tehdr_ype: u16,
    machine: u16,
    version: u32,
    entry: u64,
    ph_off: u64,
    sh_off: u64,
    flags: u32,
    eh_size: u16,
    ph_ent_size: u16,
    ph_num: u16,
    sh_ent_size: u16,
    sh_num: u16,
    sh_strndx: u16,
}
#[allow(dead_code)]
pub struct Shdr {
    name: u32,
    shdr_ype: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addr_align: u64,
    ent_size: u64,
}
