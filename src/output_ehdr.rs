use crate::{
    chunk::Chunk,
    context::Context,
    elf::{
        Ehdr, EHDR_SIZE, EI_ABIVERSION, EI_CLASS, EI_DATA, EI_OSABI, EI_VERSION, ELFCLASS64,
        ELFDATA2LSB, EM_RISCV, ET_EXEC, EV_CURRENT, PHDR_SIZE, SHDR_SIZE,
    },
    magic::write_magic,
};

#[allow(dead_code)]
pub struct OutputEhdr {
    pub chunk: *mut Chunk,
}

#[allow(dead_code)]
impl OutputEhdr {
    pub fn new() -> Self {
        OutputEhdr {
            chunk: &mut Chunk::new(),
        }
    }

    pub fn copy_buf(&self, ctx: &mut Context) {
        let mut ehdr = Ehdr::new();
        write_magic(&mut ehdr.ident[..]);

        ehdr.ident[EI_CLASS as usize] = ELFCLASS64;
        ehdr.ident[EI_DATA as usize] = ELFDATA2LSB;
        ehdr.ident[EI_VERSION as usize] = EI_VERSION;
        ehdr.ident[EI_OSABI as usize] = 0;
        ehdr.ident[EI_ABIVERSION as usize] = 0;

        ehdr.hdr_type = ET_EXEC;
        ehdr.machine = EM_RISCV;
        ehdr.version = EV_CURRENT;

        ehdr.eh_size = EHDR_SIZE as u16;
        ehdr.ph_ent_size = PHDR_SIZE as u16;
        ehdr.sh_ent_size = SHDR_SIZE as u16;

        let buf =
            unsafe { std::slice::from_raw_parts((&ehdr as *const Ehdr) as *const u8, EHDR_SIZE) };
        
        let _ = (ctx.buf[(unsafe { self.chunk.as_mut().unwrap().shdr.offset } as usize)..]).copy_from_slice(buf);

    }
}
