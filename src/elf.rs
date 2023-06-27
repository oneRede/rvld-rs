use std::mem;

#[allow(dead_code)]
pub const EHDR_SIZE: usize = mem::size_of::<Ehdr>();
#[allow(dead_code)]
pub const SHDR_SIZE: usize = mem::size_of::<Shdr>();
#[allow(dead_code)]
pub const SYM_SIZE: usize = mem::size_of::<Sym>();
#[allow(dead_code)]
pub const AR_HDR_SIZE: usize = mem::size_of::<ArHdr>();
#[allow(dead_code)]
pub const ELF_ABS: u16 = 0;
#[allow(dead_code)]
pub const ELF_UNDEF: u16 = 0;
pub const PHDR_SIZE: usize = mem::size_of::<Phdr>();
pub const RELA_SIZE: usize = mem::size_of::<Rela>();

pub const SHF_GROUP: u64 = 0;
pub const SHF_COMPRESSED: u64 = 0;
pub const SHF_MERGE: u64 = 0;
pub const SHF_STRINGS: u64 = 0;
pub const SHF_COMMON: u64 = 0;

pub const SHT_GROUP: u32 = 0;
pub const SHT_SYMTAB: u32 = 1;
pub const SHT_STRTAB: u32 = 2;
pub const SHT_REL: u32 = 3;
pub const SHT_RELA: u32 = 4;
pub const SHT_NULL: u32 = 5;
pub const SHT_SYMTAB_SHNDX: u32 = 6;
pub const SHN_XINDEX: u16 = 7;

pub const EI_CLASS: u8 = 0;
pub const EI_DATA: u8 = 0;
pub const EI_VERSION: u8 = 0;
pub const EI_OSABI: u8 = 0;
pub const EI_ABIVERSION: u8 = 0;

pub const ELFCLASS64: u8 = 0;
pub const ELFDATA2LSB: u8 = 0;

pub const EV_CURRENT: u32 = 0;

pub const ET_EXEC: u16 = 0;

pub const EM_RISCV: u16 = 0;

pub const SHT_NOBITS:u32 = 0;

pub const SHF_ALLOC: u64 = 0;

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
impl Ehdr {
    pub fn new() -> Self {
        Ehdr {
            ident: [0; 16],
            hdr_type: 0,
            machine: 0,
            version: 0,
            entry: 0,
            ph_off: 0,
            sh_off: 0,
            flags: 0,
            eh_size: 0,
            ph_ent_size: 0,
            ph_num: 0,
            sh_ent_size: 0,
            sh_num: 0,
            sh_strndx: 0,
        }
    }
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
impl Shdr{
    pub fn new() -> Self{
        Shdr{
            name: 0,
            shdr_type: 0,
            flags: 0,
            addr: 0,
            offset: 0,
            size: 0,
            link: 0,
            info: 0,
            addr_align: 0,
            ent_size: 0,
        }
    }
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Phdr {
    pub p_type: u32,
    pub falgs: u32,
    pub offset: u64,
    pub v_addr: u64,
    pub p_addr: u64,
    pub file_size: u64,
    pub mem_size: u64,
    pub align: u64,
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
impl Sym {
    pub fn is_abs(&self) -> bool {
        self.shndx == ELF_ABS
    }

    pub fn is_undef(&self) -> bool {
        self.shndx == ELF_UNDEF
    }

    pub fn is_common(&self) -> bool {
        self.shndx == SHF_COMMON as u16
    }
}

#[allow(dead_code)]
pub fn elf_get_name<'a>(str_tab: &'a [u8], offset: u32) -> &'a str {
    let offset = offset as usize;
    let len = binary_search(&str_tab[offset..], 0).unwrap();
    return std::str::from_utf8(&str_tab[offset..(offset + len)]).unwrap();
}

#[allow(dead_code)]
fn binary_search(data: &[u8], sep: u8) -> Option<usize> {
    for i in 0..data.len() {
        if data[i] == sep {
            return Some(i);
        }
    }
    None
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArHdr<'a> {
    name: &'a [u8; 16],
    date: &'a [u8; 12],
    uid: &'a [u8; 6],
    gid: &'a [u8; 6],
    mode: &'a [u8; 8],
    size: &'a [u8; 10],
    fmag: &'a [u8; 2],
}

#[allow(dead_code)]
impl<'a> ArHdr<'a> {
    pub fn has_prefix(&self, s: &str) -> bool {
        return s.starts_with(std::str::from_utf8(self.name).unwrap());
    }

    pub fn is_str_tab(&self) -> bool {
        return self.has_prefix("// ");
    }

    pub fn is_symtab(&self) -> bool {
        return self.has_prefix("/ ") || self.has_prefix("/SYM64");
    }

    pub fn get_size(&self) -> usize {
        let size = str::parse::<usize>(
            std::str::from_utf8(self.size)
                .unwrap()
                .strip_suffix(" ")
                .unwrap(),
        )
        .unwrap();
        size
    }

    pub fn read_name(&self, str_tab: &'a str) -> &'a str {
        if unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(
                (self as *const ArHdr) as *const u8,
                60,
            ))
            .unwrap()
            .starts_with("// ")
        } {
            let start = str::parse::<usize>(
                std::str::from_utf8(self.size)
                    .unwrap()
                    .strip_suffix(" ")
                    .unwrap(),
            )
            .unwrap();
            let end = start + str_tab.rfind("/\n").unwrap();

            return &str_tab[start..end];
        }
        let end: usize = std::str::from_utf8(self.name).unwrap().rfind("\\").unwrap();
        return std::str::from_utf8(&self.name[..end]).unwrap();
    }
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rela{
    offset: u64,
    ty: u32,
    sym: Sym,
    addend: i64,
}

#[test]
fn test_binary_seach() {
    let data = &[1u8, 2, 3, 4, 5, 6];
    let sep = 2u8;
    assert_eq!(binary_search(data, sep), Some(1));
}
