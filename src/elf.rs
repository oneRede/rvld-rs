use std::mem;

#[allow(dead_code)]
pub const EHDR_SIZE: usize = mem::size_of::<Ehdr>();
#[allow(dead_code)]
pub const SHDR_SIZE: usize = mem::size_of::<Shdr>();
#[allow(dead_code)]
pub const SYM_SIZE: usize = mem::size_of::<Sym>();

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ehdr {
    pub ident: [u8; 16],
    pub hdr_type: u16,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub ph_off: u64,
    pub sh_off: u64,
    pub flags: u32,
    pub eh_size: u16,
    pub ph_ent_size: u16,
    pub ph_num: u16,
    pub sh_ent_size: u16,
    pub sh_num: u16,
    pub sh_strndx: u16,
}
#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Shdr {
    pub name: u32,
    pub shdr_type: u32,
    pub flags: u64,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addr_align: u64,
    pub ent_size: u64,
}
#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sym {
    pub name: u32,
    pub info: u8,
    pub other: u8,
    pub shndx: u16,
    pub val: u64,
    pub size: u64,
}

pub fn elf_get_name(str_tab: &[u8], offset: u32) -> &str{
    let sep = 0u8;
    let offset = offset as usize;
    let len = str_tab.binary_search(&sep).unwrap();
    return std::str::from_utf8(&str_tab[offset..(offset + len)]).unwrap();
}
