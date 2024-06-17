use std::cmp;

use crate::{
    chunk::{Chunk, Chunker},
    context::Context,
    elf::{
        Phdr, Shdr, PF_W, PF_X, PT_LOAD, PT_NOTE, PT_PHDR, PT_TLS, SHF_ALLOC, SHF_EXECINSTR,
        SHF_TLS, SHF_WRITE, SHT_NOBITS, SHT_NOTE, PHDR_SIZE,
    },
    passes::is_tbss,
    utils::{remove_if, write},
};

const PAGE_SIZE: u64 = 4096;

#[allow(dead_code)]
pub struct OutputPhdr {
    pub chunk: *mut Chunk,
    pub phdrs: *mut Vec<Phdr>,
}

#[allow(dead_code)]
impl OutputPhdr {
    pub fn new() -> Self {
        let mut shdr = Shdr::new();
        shdr.flags = SHF_ALLOC;
        shdr.addr_align = 8;

        let mut chunk = Chunk::new();
        chunk.shdr = shdr;
        Self {
            chunk: Box::leak(Box::new(chunk)),
            phdrs: Box::leak(Box::new(vec![])),
        }
    }

    pub fn to_phdr_flags(&self, chunk: *mut Chunk) -> u32 {
        let mut ret = PF_W;
        let write = unsafe { chunk.as_ref().unwrap().get_shdr().flags } & SHF_WRITE != 0;
        if write {
            ret |= PF_W;
        }
        if unsafe { chunk.as_ref().unwrap().get_shdr().flags } & SHF_EXECINSTR != 0 {
            ret |= PF_X;
        }
        ret
    }

    pub fn create_phdr(&self, ctx: &mut Context) -> *mut Vec<Phdr> {
        let vec: *mut Vec<Phdr> = Box::leak(Box::new(vec![]));
        let define = |ty: u64, flags: u64, min_align: i64, chunk: *mut Chunk| {
            let mut phdr = Phdr::new();
            phdr.p_type = ty as u32;
            phdr.flags = flags as u32;
            phdr.align = cmp::max(min_align as u64, unsafe {
                chunk.as_ref().unwrap().get_shdr().addr_align
            });
            phdr.offset = unsafe { self.chunk.as_ref().unwrap().get_shdr().offset };

            if unsafe { chunk.as_ref().unwrap().get_shdr().shdr_type } == SHT_NOBITS {
                phdr.file_size = 0;
            } else {
                phdr.file_size = unsafe { chunk.as_ref().unwrap().get_shdr().size }
            }

            phdr.v_addr = unsafe { chunk.as_ref().unwrap().get_shdr().addr };
            phdr.p_addr = unsafe { chunk.as_ref().unwrap().get_shdr().addr };
            phdr.mem_size = unsafe { chunk.as_ref().unwrap().get_shdr().size };
            unsafe { vec.as_mut().unwrap().push(phdr) }
        };

        let push = |chunk: *mut Chunk| {
            let mut phdr = Phdr::new();
            phdr.align = cmp::max(phdr.align, unsafe {
                chunk.as_ref().unwrap().get_shdr().addr_align
            });
            if unsafe { chunk.as_ref().unwrap().get_shdr().shdr_type } != SHT_NOBITS {
                phdr.file_size = unsafe { chunk.as_ref().unwrap().get_shdr().addr }
                    + unsafe { chunk.as_ref().unwrap().get_shdr().size }
                    + phdr.v_addr;
            }
            phdr.mem_size = unsafe { chunk.as_ref().unwrap().get_shdr().addr }
                + unsafe { chunk.as_ref().unwrap().get_shdr().size }
                - phdr.v_addr;
            unsafe { vec.as_mut().unwrap().push(phdr) }
        };

        define(PT_PHDR, PF_W.into(), 8, ctx.phdr.chunk);

        let is_tls = |chunk: *mut Chunk| -> bool {
            unsafe { chunk.as_ref().unwrap().get_shdr().flags & SHF_TLS != 0 }
        };

        let is_bss = |chunk: *mut Chunk| -> bool {
            unsafe { chunk.as_ref().unwrap().get_shdr().shdr_type == SHT_NOBITS && is_tls(chunk) }
        };

        let is_note = |chunk: *mut Chunk| -> bool {
            let shdr = unsafe { chunk.as_ref().unwrap().get_shdr() };
            shdr.shdr_type == SHT_NOTE && shdr.flags & SHF_ALLOC != 0
        };

        let chunks = ctx.chunks.unwrap();
        let end = unsafe { chunks.as_ref().unwrap().len() };
        for i in 0..end {
            let first = unsafe { chunks.as_ref().unwrap() }[i];
            if !is_note(first) {
                continue;
            }
            let flags = self.to_phdr_flags(first);
            let alignment = unsafe { first.as_ref().unwrap().get_shdr().addr_align };
            define(PT_NOTE, flags as u64, alignment.try_into().unwrap(), first);

            for j in 0..end {
                if !is_note(unsafe { chunks.as_ref().unwrap() }[j]) {
                    continue;
                }
                if !self.to_phdr_flags(unsafe { chunks.as_ref().unwrap() }[j]) == flags {
                    continue;
                }
                push(unsafe { chunks.as_ref().unwrap() }[j]);
            }
        }

        {
            let mut chunks_c: Vec<*mut Chunk> = vec![];
            for i in 0..unsafe { chunks.as_ref().unwrap().len() } {
                chunks_c.push(unsafe { chunks.as_ref().unwrap() }[i])
            }

            chunks_c = remove_if(chunks_c, |chunk: &*mut Chunk| -> bool { is_tbss(*chunk) });

            let end = chunks_c.len();
            for i in 0..end {
                let first = unsafe { chunks.as_ref().unwrap() }[i];

                if unsafe { first.as_ref().unwrap().get_shdr().flags } & SHF_ALLOC == 0 {
                    break;
                }
                let flags = self.to_phdr_flags(first) as u64;
                define(PT_LOAD, flags, PAGE_SIZE as i64, first);
                if !is_bss(first) {
                    for i in 0..end {
                        if self.to_phdr_flags(chunks_c[i]) == flags as u32 {
                            continue;
                        }
                        push(chunks_c[i])
                    }
                }

                for i in 0..end {
                    if self.to_phdr_flags(chunks_c[i]) == flags as u32 {
                        continue;
                    }
                    push(chunks_c[i])
                }
            }
        }

        for i in 0..unsafe { chunks.as_ref().unwrap().len() } {
            if !is_tls(unsafe { chunks.as_ref().unwrap() }[i]) {
                continue;
            }
            define(PT_TLS, self.to_phdr_flags(unsafe { chunks.as_ref().unwrap() }[i]) as u64, 1, unsafe { chunks.as_ref().unwrap() }[i]);

            for i in 0..unsafe { chunks.as_ref().unwrap() }.len() {
                if is_tls(unsafe { chunks.as_ref().unwrap() }[i]) {
                    continue;
                }
                push(unsafe { chunks.as_ref().unwrap() }[i])
            }
            let phdr = unsafe { vec.as_ref().unwrap()[vec.as_ref().unwrap().len()] };
            ctx.tp_addr = phdr.v_addr;
        }
        vec
    }

    pub fn update_shdr(&mut self, ctx: &mut Context){
        self.phdrs = self.create_phdr(ctx);
        unsafe { self.chunk.as_mut().unwrap().shdr.size = (self.phdrs.as_ref().unwrap().len() * PHDR_SIZE) as u64};
    }

    pub fn copy_buf(&self, ctx:&mut Context){
        write(&mut ctx.buf[unsafe { self.chunk.as_ref().unwrap().shdr.offset } as usize..], self.phdrs)
    }
}
