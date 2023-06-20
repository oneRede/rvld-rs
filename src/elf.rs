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

// func (s *Sym) IsAbs() bool {
// 	return s.Shndx == uint16(elf.SHN_ABS)
// }

// func (s *Sym) IsUndef() bool {
// 	return s.Shndx == uint16(elf.SHN_UNDEF)
// }

impl Sym{
    fn is_abs(&self) -> bool {
        self.shndx == ELF_ABS
    }

    fn is_undef(&self) -> bool{
        self.shndx == ELF_UNDEF
    }
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
pub fn elf_get_name(str_tab: &[u8], offset: u32) -> &str {
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

impl<'a> ArHdr<'a> {
    #[allow(dead_code)]
    pub fn has_prefix(&self, s: &str) -> bool {
        return s.starts_with(std::str::from_utf8(self.name).unwrap());
    }

    #[allow(dead_code)]
    pub fn is_str_tab(&self) -> bool {
        return self.has_prefix("// ");
    }

    #[allow(dead_code)]
    pub fn is_symtab(&self) -> bool {
        return self.has_prefix("/ ") || self.has_prefix("/SYM64");
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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



#[test]
fn test_binary_seach() {
    let data = &[1u8, 2, 3, 4, 5, 6];
    let sep = 2u8;
    assert_eq!(binary_search(data, sep), Some(1));
}
