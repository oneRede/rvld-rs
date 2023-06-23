use crate::{
    elf::{elf_get_name, Shdr, SHDR_SIZE, SHF_COMPRESSED},
    object_file::ObjectFile,
};
#[allow(dead_code)]
pub struct InputSection<'a> {
    object_file: *mut ObjectFile<'a>,
    contents: &'static [u8],
    shndx: usize,
    sh_size: u32,
    is_alive: bool,
    p2_align: u8,
}

#[allow(dead_code)]
impl<'a> InputSection<'a> {
    pub fn new(object_file: *mut ObjectFile<'a>, shndx: usize) -> Self {
        let shdr = unsafe { (object_file.as_ref()).unwrap().input_file.as_ref().unwrap() }
            .elf_sections[shndx];
        let contents = &unsafe { (object_file.as_ref()).unwrap().input_file.as_ref().unwrap() }
            .file
            .contents[shdr.offset as usize..(shdr.offset + SHDR_SIZE as u64) as usize];
        assert!(shdr.flags & SHF_COMPRESSED == 0);
        let sh_size = shdr.size;
        let to_p2_align = |align: u64| -> u8 {
            if align == 0 {
                return 0;
            }
            return u64::trailing_zeros(align) as u8;
        };
        let p2_align = to_p2_align(shdr.addr_align);
        InputSection {
            object_file,
            contents,
            shndx,
            sh_size: sh_size as u32,
            is_alive: true,
            p2_align,
        }
    }

    fn shdr(&self) -> Shdr {
        assert!(
            self.shndx
                < unsafe {
                    (self.object_file.as_ref().unwrap())
                        .input_file
                        .as_ref()
                        .unwrap()
                }
                .elf_sections
                .len()
        );
        unsafe {
            (self.object_file.as_ref().unwrap())
                .input_file
                .as_ref()
                .unwrap()
        }
        .elf_sections[self.shndx]
    }

    fn name(&self) -> &str {
        elf_get_name(
            unsafe {
                (self.object_file.as_ref().unwrap())
                    .input_file
                    .as_ref()
                    .unwrap()
            }
            .sh_strtab
            .unwrap(),
            self.shdr().name,
        )
    }
}
