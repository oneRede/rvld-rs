use std::{cmp, vec};

use crate::{
    chunk::{Chunk, Chunker},
    context::Context,
    elf::{self, IMAGE_BASE, SHF_ALLOC, SHF_TLS, SHT_NOBITS},
    input_section::InputSection,
    object_file::ObjectFile,
    output_ehdr::OutputEhdr,
    symbol::{Symbol, NEEDS_GOT_TP},
    utils::{align_to, remove_if},
};

#[allow(dead_code)]
pub fn resolve_symbols(ctx: &mut Context) {
    for file in &ctx.objs {
        unsafe { file.as_mut().unwrap().resolve_symbols() }
    }

    mark_live_objects(ctx);
    for file in &ctx.objs {
        if unsafe { file.as_ref().unwrap().input_file.as_ref().unwrap().is_alive } {
            unsafe { file.as_ref().unwrap().clear_symbols() };
        }
    }

    let func = |file: &*mut ObjectFile| -> bool {
        unsafe { file.as_ref().unwrap().input_file.as_ref().unwrap().is_alive }
    };
    ctx.objs = remove_if(ctx.objs.clone(), func);
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
    unsafe { ctx.chunks.unwrap().as_mut().unwrap().push(ctx.ehdr.chunk) };
}

#[allow(dead_code)]
fn get_file_size(ctx: Context) -> u64 {
    let mut file_off = 0u64;

    for c in unsafe { ctx.chunks.unwrap().as_ref().unwrap() } {
        file_off = align_to(file_off, unsafe { c.as_ref().unwrap().shdr.addr_align });
        file_off += unsafe { c.as_ref().unwrap().shdr.size };
    }

    file_off
}

#[allow(dead_code)]
pub fn is_tbss(chunk: *mut Chunk) -> bool {
    let shdr = unsafe { chunk.as_ref().unwrap().get_shdr() };
    shdr.shdr_type == SHT_NOBITS && shdr.flags & SHF_TLS != 0
}

#[allow(dead_code)]
fn set_output_section_offsets(ctx: *mut Context) -> u64 {
    let mut addr = IMAGE_BASE;
    let chunks = unsafe { ctx.as_ref().unwrap().chunks.unwrap() };
    for chunk in unsafe { chunks.as_ref().unwrap() } {
        if unsafe { chunk.as_ref().unwrap().get_shdr().flags } & SHF_ALLOC == 0 {
            continue;
        }
        addr = align_to(addr, unsafe {
            chunk.as_ref().unwrap().get_shdr().addr_align
        });
        unsafe { chunk.as_mut().unwrap().get_shdr().addr = addr };

        if !is_tbss(*chunk) {
            addr += unsafe { chunk.as_ref().unwrap().get_shdr().size };
        }
    }

    let mut i = 0;
    let first = unsafe { chunks.as_ref().unwrap() }[0];

    loop {
        let shdr = unsafe { &mut chunks.as_mut().unwrap()[i].as_ref().unwrap().get_shdr() };
        shdr.offset = shdr.addr - unsafe { first.as_ref().unwrap().shdr.addr };
        i += 1;
        if i >= unsafe { chunks.as_ref().unwrap().len() }
            || unsafe {
                &chunks.as_mut().unwrap()[i]
                    .as_ref()
                    .unwrap()
                    .get_shdr()
                    .flags
            } & SHF_ALLOC
                == 0
        {
            break;
        }
    }

    let last_shdr = unsafe { chunks.as_ref().unwrap()[i - 1].as_ref().unwrap().get_shdr() };
    let mut file_off = last_shdr.offset + last_shdr.size;

    for _j in i..unsafe { chunks.as_ref().unwrap().len() } {
        let mut shdr = unsafe { chunks.as_ref().unwrap()[i].as_ref().unwrap().get_shdr() };
        file_off = align_to(file_off, shdr.addr_align);
        shdr.offset = file_off;
        file_off += shdr.size;
    }

    unsafe {
        ctx.as_mut()
            .unwrap()
            .phdr
            .update_shdr(ctx.as_mut().unwrap())
    };
    file_off
}

#[allow(dead_code)]
pub fn bin_sections(ctx: &Context) {
    let mut group: Vec<*mut Vec<*mut InputSection>> = vec![];
    for _i in 0..unsafe { ctx.output_sections.as_ref().unwrap() }.len() {
        group.push(Box::leak(Box::new(vec![])))
    }
    for file in &ctx.objs {
        for isec in unsafe { &file.as_ref().unwrap().input_sections } {
            let isec_op = isec.is_none();
            let isec_ref = unsafe { isec.unwrap().as_ref().unwrap() };
            if isec_op || isec_ref.is_alive {
                continue;
            }
            let idx = unsafe { isec_ref.output_section.unwrap().as_ref().unwrap().idx as usize };
            unsafe { group[idx].as_mut().unwrap().push(isec.unwrap()) };
        }
    }
    let idx = 0;
    for osec in unsafe { ctx.output_sections.as_ref().unwrap() } {
        unsafe { osec.as_mut().unwrap().members = group[idx] };
    }
}

#[allow(dead_code)]
pub fn collect_output_sections(ctx: Context) -> Vec<*mut Chunk> {
    let mut osecs: Vec<*mut Chunk> = vec![];
    for osec in unsafe { ctx.output_sections.as_ref().unwrap() } {
        if unsafe { osec.as_ref().unwrap().members.as_ref().unwrap().len() } > 0 {
            osecs.push(unsafe { osec.as_ref().unwrap().chunk })
        }
    }

    for osec in ctx.merged_sections {
        if unsafe { osec.as_ref().unwrap().chunk.as_ref().unwrap().shdr.size } > 0 {
            osecs.push(unsafe { osec.as_ref().unwrap().chunk })
        }
    }

    osecs
}

#[allow(dead_code)]
pub fn compute_section_sizes(ctx: &Context) {
    for osec in unsafe { ctx.output_sections.as_ref().unwrap() } {
        let mut offset = 0u64;
        let mut p2_align = 0u64;

        for isec in unsafe { osec.as_ref().unwrap().members.as_ref().unwrap() } {
            offset = align_to(offset, 1 << p2_align);
            unsafe { isec.as_mut().unwrap().offset = offset as u32 };
            offset += unsafe { isec.as_ref().unwrap().sh_size as u64 };
            p2_align = cmp::max(p2_align, unsafe { isec.as_ref().unwrap().p2_align as u64 })
        }

        unsafe { osec.as_mut().unwrap().chunk.as_mut().unwrap().shdr.size = offset };
        unsafe {
            osec.as_mut()
                .unwrap()
                .chunk
                .as_mut()
                .unwrap()
                .shdr
                .addr_align = 1 << p2_align
        };
    }
}

#[allow(dead_code)]
pub fn sort_output_sections(ctx: &Context) {
    let rank = |chunk: &Chunk| -> i32 {
        let ty = chunk.get_shdr().shdr_type;
        let flags = chunk.get_shdr().flags;

        if flags & SHF_ALLOC == 0 {
            return (u32::MAX - 1) as i32;
        }
        if chunk == &ctx.shdr.chunk {
            return i32::MAX;
        }
        if chunk == &*unsafe { ctx.ehdr.chunk.as_ref().unwrap() } {
            return 0;
        }
        if chunk == &*unsafe { ctx.phdr.chunk.as_ref().unwrap() } {
            return 1;
        }
        if ty == elf::SHT_NOTE {
            return 2;
        }

        let b2i = |b: bool| -> i32 {
            if b {
                return 1;
            }
            return 0;
        };

        let writeable = b2i(flags & elf::SHF_WRITE != 0);
        let not_exec = b2i(flags & elf::SHF_EXECINSTR == 0);
        let not_tls = b2i(flags & elf::SHF_TLS == 0);
        let is_bss = b2i(ty == elf::SHT_NOBITS);

        return writeable << 7 | not_exec << 6 | not_tls << 5 | is_bss << 4 as i32;
    };

    unsafe {
        ctx.chunks.unwrap().as_mut().unwrap().sort_by(|a, b| {
            rank(a.as_ref().unwrap())
                .partial_cmp(&rank(b.as_ref().unwrap()))
                .unwrap()
        })
    };
}

#[allow(dead_code)]
pub fn compute_merged_sections_size(ctx: &Context) {
    for osec in ctx.merged_sections.clone() {
        unsafe { osec.as_mut().unwrap().assign_offsets() }
    }
}

#[allow(dead_code)]
pub fn scan_relocations(ctx: &mut Context) {
    let mut syms: Vec<*mut Symbol> = vec![];
    for file in &ctx.objs {
        for sym in unsafe { &file.as_ref().unwrap().input_file.as_ref().unwrap().symbols } {
            if unsafe { sym.as_ref().unwrap().object_file == Some(file.clone()) }
                && unsafe { sym.as_ref().unwrap().flags } != 0
            {
                syms.push(*sym)
            }
        }
    }

    for sym in syms {
        if unsafe { sym.as_ref().unwrap().flags } & NEEDS_GOT_TP != 0 {
            ctx.got.add_got_tp_symbol(sym)
        }
        unsafe { sym.as_mut().unwrap().flags = 0 }
    }
}
