// package linker

// import "github.com/ksco/rvld/pkg/utils"

// type InputSection struct {
// 	File     *ObjectFile
// 	Contents []byte
// 	Shndx    uint32
// }

// func NewInputSection(file *ObjectFile, shndx uint32) *InputSection {
// 	s := &InputSection{File: file, Shndx: shndx}

// 	shdr := s.Shdr()
// 	s.Contents = file.File.Contents[shdr.Offset : shdr.Offset+shdr.Size]

// 	return s
// }

// func (i *InputSection) Shdr() *Shdr {
// 	utils.Assert(i.Shndx < uint32(len(i.File.ElfSections)))
// 	return &i.File.ElfSections[i.Shndx]
// }

// func (i *InputSection) Name() string {
// 	return ElfGetName(i.File.ShStrtab, i.Shdr().Name)
// }

use crate::{object_file::ObjectFile, elf::{Shdr, elf_get_name, SHDR_SIZE}};

pub struct InputSection<'a>{
    object_file: ObjectFile<'a>,
    contents: &'static [u8],
    shndx: usize
}

impl<'a> InputSection<'a>{
    #[allow(dead_code)]
    fn new(object_file: ObjectFile<'a>, shndx: usize) -> Self{
        let shdr = object_file.input_file.elf_sections[shndx];
        let contents = &object_file.input_file.file.contents[shdr.offset as usize..(shdr.offset+SHDR_SIZE as u64) as usize];
        InputSection { object_file, contents: contents, shndx }

    }

    #[allow(dead_code)]
    fn shdr(&self) -> Shdr{
        assert!(self.shndx < self.object_file.input_file.elf_sections.len() );
        self.object_file.input_file.elf_sections[self.shndx]
    }

    #[allow(dead_code)]
    fn name(&self) -> &str{
        elf_get_name(self.object_file.input_file.sh_strtab.unwrap(), self.shdr().name)
    }
}