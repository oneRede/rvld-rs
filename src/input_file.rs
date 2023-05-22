use crate::elf::Shdr;
use crate::file::ElfFile;
// use crate::elf::EHDR_SIZE;

#[allow(dead_code)]
struct InputFile {
    file: ElfFile,
    elf_sections: Vec<Shdr>,
}

// func NewInputFile(file *File) InputFile {
// 	f := InputFile{File: file}
// 	if len(file.Contents) < EhdrSize {
// 		utils.Fatal("file too small")
// 	}
// 	if !CheckMagic(file.Contents) {
// 		utils.Fatal("not an ELF file")
// 	}
// 	ehdr := utils.Read[Ehdr](file.Contents)
// 	contents := file.Contents[ehdr.ShOff:]
// 	shdr := utils.Read[Shdr](contents)
// 	numSections := int64(ehdr.ShNum)
// 	if numSections == 0 {
// 		numSections = int64(shdr.Size)
// 	}
// 	f.ElfSections = []Shdr{shdr}
// 	for numSections > 1 {
// 		contents = contents[ShdrSize:]
// 		f.ElfSections = append(f.ElfSections, utils.Read[Shdr](contents))
// 		numSections--
// 	}
// 	return f
// }

// fn new_input_file(elf_file: ElfFile) {
//     let input_file = InputFile {
//         file: elf_file,
//         elf_sections: Vec::new(),
//     };

//     if (&elf_file.contents).len() < EHDR_SIZE {
//         println!("error!!");
//     }
// }
