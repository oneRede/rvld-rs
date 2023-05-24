use crate::elf::{EHDR_SIZE, SHDR_SIZE};
use crate::elf::{Ehdr, Shdr};
use crate::file::ElfFile;
use crate::magic::check_magic;
use crate::utils::{fatal, read, read_ehdr};

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

fn new_input_file(file: ElfFile) -> InputFile {
    let mut f = InputFile {
        file: file,
        elf_sections: Vec::new(),
    };

    if f.file.contents.len() < EHDR_SIZE {
        fatal("file too small");
    }
    if !check_magic(&f.file.contents) {
        fatal("not an ELF file")
    }
    let ehdr: Ehdr = read(&mut f.file.contents);
    let contents = &file.contents[ehdr.sh_off as usize..];
    let shdr: Shdr = read(&mut f.file.contents[(ehdr.sh_off as usize)..]);

    let mut num_sections = ehdr.sh_num as i64;
    if num_sections == 0 {
        num_sections = shdr.size as i64;
    }

    f.elf_sections.push(shdr);
    for i in 0..num_sections {
        // let contents = &contents[SHDR_SIZE..];
    }
    return f;
}
