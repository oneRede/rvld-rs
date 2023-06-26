use std::vec;

use crate::{
    context::Context, object_file::ObjectFile, output_ehdr::OutputEhdr,
    utils::align_to,
};

#[allow(dead_code)]
pub fn resolve_symbols(ctx: &mut Context) {
    let mut marks: Vec<usize> = vec![];
    for file in &ctx.objs {
        unsafe { file.as_mut().unwrap().resolve_symbols() }
        if unsafe { file.as_ref().unwrap().input_file.as_ref().unwrap().is_alive } {
            marks.push(1);
        } else {
            marks.push(0);
        }
    }

    mark_live_objects(ctx);
    for file in &ctx.objs {
        if unsafe { file.as_ref().unwrap().input_file.as_ref().unwrap().is_alive } {
            unsafe { file.as_ref().unwrap().clear_symbols() };
        }
    }

    let _func = |file: &*mut ObjectFile| -> bool {
        unsafe { file.as_ref().unwrap().input_file.as_ref().unwrap().is_alive }
    };
    for i in 0..ctx.objs.len() {
        if marks.get(i).unwrap() == &0 {
            ctx.objs.remove(i);
        }
    }
}

#[allow(dead_code)]
pub fn mark_live_objects(ctx: &Context) {
    let mut roots = vec![];
    for file in &ctx.objs {
        if unsafe { file.as_ref().unwrap().input_file.as_ref().unwrap().is_alive } {
            roots.push(file.cast())
        }
    }

    assert!(roots.len() > 0);

    for _i in 0..roots.len() {
        let file: *mut ObjectFile = roots[0];
        if unsafe { file.as_ref().unwrap().input_file.as_ref().unwrap().is_alive } {
            continue;
        }

        let func = || roots.push(file);

        unsafe { file.as_ref().unwrap().mark_live_objects(ctx, func) }

        roots.remove(0);
    }
}

#[allow(dead_code)]
pub fn register_section_pieces(ctx: &mut Context) {
    for file in &ctx.objs {
        unsafe { file.as_mut().unwrap().register_section_pieces() }
    }
}

#[allow(dead_code)]
pub fn create_synthetic_sections(ctx: &mut Context) {
    ctx.ehdr = OutputEhdr::new();
    ctx.chunks.as_mut().unwrap().push(ctx.ehdr.chunk);
}

#[allow(dead_code)]
fn get_file_size(ctx: Context) -> u64 {
    let mut file_off = 0u64;

    for c in ctx.chunks.unwrap() {
        file_off = align_to(file_off, unsafe { c.as_ref().unwrap().shdr.addr_align });
        file_off += unsafe { c.as_ref().unwrap().shdr.size };
    }

    file_off
}
