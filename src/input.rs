use std::cell::UnsafeCell;

use crate::archive::read_archive_members;
use crate::context::Context;
use crate::file::{find_library, must_new_file, ElfFile};
use crate::file_type::{get_file_type, FILE_TYPE_ARCHIVE, FILE_TYPE_OBJECT, check_file_compatibility};
use crate::object_file::{new_object_file, ObjectFile};
use crate::utils::{fatal, remove_prefix};

#[allow(dead_code)]
pub fn read_input_files(ctx: &mut Context, remaining: Vec<String>) {
    let ctx = UnsafeCell::new(ctx);
    let _ctx = ctx.get();
    for arg in remaining {
        let (arg, ok) = remove_prefix(&arg, "-l");
        let arg = Box::leak(Box::new(arg));
        if ok {
            read_file(
                unsafe { _ctx.as_mut().unwrap() },
                find_library(unsafe { &*ctx.get() }, arg).unwrap(),
            )
        } else {
            read_file(unsafe { _ctx.as_mut().unwrap() }, must_new_file(arg))
        }
    }
}

#[allow(dead_code)]
pub fn read_file<'a>(ctx: &mut Context<'a>, elf_file: ElfFile<'a>) {
    let ft = get_file_type(elf_file.contents);
    let emulation: u8 = ctx.args.emulation;
    match ft {
        FILE_TYPE_OBJECT => {
            ctx.objs.push(create_object_file(emulation, elf_file));
        }
        FILE_TYPE_ARCHIVE => {
            for child in read_archive_members(elf_file) {
                assert_eq!(get_file_type(child.contents), FILE_TYPE_OBJECT);
                ctx.objs.push(create_object_file(emulation, child))
            }
        }
        _ => fatal("unkown file type!"),
    }
}

#[allow(dead_code)]
fn create_object_file<'a>(emulation :u8,elf_file: ElfFile<'a>) -> ObjectFile<'a> {
    check_file_compatibility(emulation, &elf_file);
    let mut obj = new_object_file(elf_file);
    obj.parse();
    obj
}
