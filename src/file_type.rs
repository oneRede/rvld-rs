use crate::file::ElfFile;
use crate::machine_type::get_machine_type_from_contents;
use crate::magic::check_magic;
use crate::utils::{fatal, read};

pub type FileType = u8;
pub const FILE_TYPE_UNKNOWN: FileType = 0;
pub const FILE_TYPE_EMPTY: FileType = 1;
pub const FILE_TYPE_OBJECT: FileType = 2;
pub const FILE_TYPE_ARCHIVE: FileType = 3;

#[allow(dead_code)]
pub fn get_file_type(contents: &[u8]) -> FileType {
    if contents.len() == 0 {
        return FILE_TYPE_EMPTY;
    }

    if check_magic(contents) {
        let et = read(&contents[16..]);
        match et {
            1u16 => return FILE_TYPE_OBJECT,
            _ => {}
        }
    }

    if contents.starts_with("!<arch>\n".as_bytes()) {
        return FILE_TYPE_ARCHIVE;
    }
    return FILE_TYPE_UNKNOWN;
}

#[allow(dead_code)]
pub fn check_file_compatibility(emulation: u8, elf_file: &ElfFile) {
    let mt = get_machine_type_from_contents(elf_file.contents);
    if mt != emulation {
        fatal("incompatible file type")
    }
}
