use crate::magic::check_magic;
use crate::utils::read;

pub type FileType = u8;
pub const FILE_TYPE_UNKNOWN: FileType = 0;
pub const FILE_TYPE_EMPTY: FileType = 0;
pub const FILE_TYPE_OBJECT: FileType = 0;

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
    return FILE_TYPE_UNKNOWN;
}
