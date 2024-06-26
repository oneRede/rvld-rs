use crate::{
    elf::{ArHdr, AR_HDR_SIZE},
    file::ElfFile,
    file_type::{get_file_type, FILE_TYPE_ARCHIVE},
    utils::read,
};

#[allow(dead_code)]
pub fn read_archive_members(file: ElfFile) -> Vec<ElfFile> {
    assert!(get_file_type(file.contents) == FILE_TYPE_ARCHIVE);

    let mut pos: usize = 8;
    let mut str_tab: &str = "";
    let mut elf_files: Vec<ElfFile> = vec![];
    while  file.contents.len() - pos > 1 {
        if pos % 2 == 1 {
            pos += 1;
        }
        let hdr: ArHdr = read::<ArHdr>(&file.contents[pos..]);
        let data_start = pos + AR_HDR_SIZE;
        pos = data_start + hdr.get_size();
        let data_end = pos;
        let contents = &file.contents[data_start..data_end];

        if hdr.is_symtab() {
            continue;
        } else if hdr.is_str_tab() {
            str_tab = std::str::from_utf8(contents).unwrap();
            continue;
        }

        elf_files.push(ElfFile {
            name: Box::leak(Box::new(hdr.read_name(str_tab))),
            contents: contents,
            files: vec![&file as *const ElfFile],
        });
    }

    elf_files
}
