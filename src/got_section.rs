use crate::{
    chunk::Chunk,
    context::Context,
    elf::{SHF_ALLOC, SHF_WRITE, SHT_PROGBITS},
    symbol::Symbol,
    utils::write,
};

#[allow(dead_code)]
pub struct GotSection<'a> {
    pub chunk: Chunk,
    pub got_tp_syms: Vec<*mut Symbol<'a>>,
}

#[allow(dead_code)]
impl<'a> GotSection<'a> {
    pub fn new() -> Self {
        let mut chunk = Chunk::new();
        chunk.name = ".got".to_string();
        chunk.shdr.shdr_type = SHT_PROGBITS;
        chunk.shdr.flags = SHF_ALLOC | SHF_WRITE;
        chunk.shdr.addr_align = 0;

        Self {
            chunk: chunk,
            got_tp_syms: vec![],
        }
    }

    pub fn add_got_tp_symbol(&mut self, sym: *mut Symbol<'a>) {
        unsafe { sym.as_mut().unwrap().got_tp_id = (self.chunk.shdr.size / 8).try_into().unwrap() };
        self.chunk.shdr.size += 8;
        self.got_tp_syms.push(sym);
    }

    pub fn get_entries(&self, ctx: *mut Context) -> Vec<GotEntry> {
        let mut entries: Vec<GotEntry> = vec![];
        for sym in &self.got_tp_syms {
            let idx = unsafe { sym.as_ref().unwrap().got_tp_id };
            let val = unsafe { sym.as_ref().unwrap().get_addr() }
                - unsafe { ctx.as_ref().unwrap().tp_addr };
            let entry = GotEntry::new(idx as i64, val);
            entries.push(entry)
        }
        entries
    }

    pub fn copy_buf(&self, ctx: *mut Context) {
        let base =
            &mut unsafe { &mut ctx.as_mut().unwrap().buf }[self.chunk.shdr.offset as usize..];
        for ent in self.get_entries(ctx) {
            write(base, ent.val);
        }
    }
}

#[allow(dead_code)]
pub struct GotEntry {
    pub idx: i64,
    pub val: u64,
}

impl GotEntry {
    pub fn new(idx: i64, val: u64) -> Self {
        Self { idx, val }
    }
}
