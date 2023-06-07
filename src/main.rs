use std::env;

use elf::elf_get_name;
use file::must_new_file;
use object_file::new_object_file;
use utils::fatal;

mod elf;
mod file;
mod input_file;
mod magic;
mod object_file;
mod utils;

// objFile := linker.NewObjectFile(file)
// 	objFile.Parse()
// 	utils.Assert(len(objFile.ElfSections) == 11)
// 	utils.Assert(objFile.FirstGlobal == 10)
// 	utils.Assert(len(objFile.ElfSyms) == 12)

// 	for _, sym := range objFile.ElfSyms {
// 		println(linker.ElfGetName(objFile.SymbolStrtab, sym.Name))
// 	}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() < 2 {
        fatal("wrong args");
    }

    let elf_file = must_new_file(&args[1]);
    let mut object_file = new_object_file(elf_file);
    object_file.parse();
    assert!(object_file.input_file.elf_sections.len() == 11);
    assert!(object_file.input_file.first_global == Some(10));
    assert!(object_file.input_file.elf_syms.len() == 12);
    for sym in object_file.input_file.elf_syms {
        println!("{:?}", elf_get_name(object_file.input_file.symbol_strtab.unwrap(), sym.name))
    }
}
