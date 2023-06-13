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

#[allow(dead_code)]
pub fn elf_get_name(str_tab: &[u8], offset: u32) -> &str{
    let offset = offset as usize;
    let len = binary_search(&str_tab[offset..], 0).unwrap();
    return std::str::from_utf8(&str_tab[offset..(offset + len)]).unwrap();
}

#[allow(dead_code)]
fn binary_search(data: &[u8], sep: u8) -> Option<usize>{
    for i in 0..data.len() {
        if data[i] == sep {
            return Some(i)
        }
    }
    None
}

#[test]
fn test_binary_seach(){
    let data = &[1u8,2,3,4,5,6];
    let sep = 2u8;
    assert_eq!(binary_search(data, sep), Some(1));
}
