use std::vec;

use crate::{
    context::Context,
    elf::{
        elf_get_name, Shdr, Sym, SHF_ALLOC, SHF_MERGE, SHF_STRINGS, SHN_XINDEX, SHT_GROUP,
        SHT_NULL, SHT_REL, SHT_RELA, SHT_STRTAB, SHT_SYMTAB, SHT_SYMTAB_SHNDX,
    },
    file::ElfFile,
    input_file::{new_input_file, InputFile},
    input_section::InputSection,
    mergeablesection::MergeableSection,
    merged_section::get_merged_section_instance,
    symbol::Symbol,
    utils::{all_zeros, fatal},
};

#[allow(dead_code)]
pub struct ObjectFile<'a> {
    pub input_file: *mut InputFile<'a>,
    pub symtab_sec: Option<Shdr>,
    pub symbol_shndx_sec: Vec<u32>,
    pub input_sections: Vec<Option<*mut InputSection<'a>>>,
    pub mergeable_sections: Vec<Option<*mut MergeableSection>>,
}

#[allow(dead_code)]
pub fn new_object_file(elf_file: ElfFile, is_alive: bool) -> ObjectFile {
    let input_file = new_input_file(elf_file);
    unsafe { input_file.as_mut().unwrap().is_alive = is_alive };
    let object_file = ObjectFile {
        input_file: input_file,
        symtab_sec: None,
        symbol_shndx_sec: vec![],
        input_sections: vec![],
        mergeable_sections: vec![],
    };
    object_file
}

#[allow(dead_code)]
impl<'a> ObjectFile<'a> {
    pub fn parse(&mut self) {
        self.symtab_sec =
            unsafe { self.input_file.as_mut().unwrap() }.find_section(SHT_SYMTAB as u32);
        match self.symtab_sec {
            None => {}
            Some(shdr) => {
                unsafe { self.input_file.as_mut().unwrap() }.first_global = Some(shdr.info as i64);
                unsafe { self.input_file.as_mut().unwrap() }.fillup_elf_syms(shdr);
                unsafe { self.input_file.as_mut().unwrap() }.symbol_strtab = Some(
                    unsafe { self.input_file.as_ref().unwrap() }
                        .get_bytes_from_idx(shdr.link as i64),
                );
            }
        }
    }

    pub fn initialize_sections(&'a mut self, ctx: Context<'a>) {
        for i in 0..unsafe { self.input_file.as_ref().unwrap() }
            .elf_sections
            .len()
        {
            let shdr = unsafe { self.input_file.as_ref().unwrap() }.elf_sections[i];
            match shdr.shdr_type {
                SHT_GROUP | SHT_SYMTAB | SHT_STRTAB | SHT_RELA | SHT_NULL | SHT_REL => {
                    break;
                }
                SHT_SYMTAB_SHNDX => {
                    self.fillup_symtab_shndx_sec(shdr);
                }
                _ => {
                    let name = elf_get_name(
                        unsafe { self.input_file.as_ref().unwrap().sh_strtab.unwrap() },
                        shdr.name,
                    );
                    self.input_sections[i] = Some(Box::leak(Box::new(InputSection::new(
                        &ctx,
                        name.to_owned(),
                        self,
                        i,
                    ))));
                }
            }
        }
    }

    pub fn initialize_symbols(&'a mut self, ctx: Context<'a>) {
        if self.symtab_sec.is_none() {
            return ();
        }

        for _i in 0..unsafe { self.input_file.as_ref().unwrap().local_symbols.len() } {
            unsafe {
                self.input_file
                    .as_mut()
                    .unwrap()
                    .local_symbols
                    .push(Box::leak(Box::new(Symbol::new(""))))
            }
        }

        for i in 0..unsafe { self.input_file.as_ref().unwrap().local_symbols.len() } {
            let esym = unsafe { &self.input_file.as_ref().unwrap().elf_syms }[i];
            let sym = unsafe { &self.input_file.as_mut().unwrap().local_symbols }[i];
            let str_tab = unsafe { self.input_file.as_ref().unwrap().symbol_strtab.unwrap() };
            unsafe { sym.as_mut().unwrap() }.name = elf_get_name(str_tab, esym.name);
            unsafe { sym.as_mut().unwrap() }.object_file = Some(self as *mut ObjectFile);
            unsafe { sym.as_mut().unwrap() }.value = esym.val;
            unsafe { sym.as_mut().unwrap() }.symidx = i as i32;
        }

        for i in 0..unsafe { self.input_file.as_ref().unwrap().local_symbols.len() } {
            unsafe {
                self.input_file
                    .as_mut()
                    .unwrap()
                    .symbols
                    .push(self.input_file.as_ref().unwrap().local_symbols[i])
            };
        }

        let len_ls = unsafe { self.input_file.as_ref().unwrap().local_symbols.len() };
        let len_es = unsafe { self.input_file.as_ref().unwrap().elf_syms.len() };
        for i in len_ls..len_es {
            let esym = unsafe { &self.input_file.as_ref().unwrap().elf_syms }[i];
            let str_tab = unsafe { self.input_file.as_ref().unwrap().symbol_strtab.unwrap() };
            let name = elf_get_name(str_tab, esym.name);
            unsafe { self.input_file.as_mut().unwrap() }.symbols[i] =
                Symbol::get_symbol_by_name(&ctx, name)
        }
    }

    pub fn fillup_symtab_shndx_sec(&mut self, shdr: Shdr) {
        let bs = unsafe { self.input_file.as_ref().unwrap() }.get_bytes_from_shdr(&shdr);
        self.symbol_shndx_sec = bs.into_iter().map(|n| *n as u32).collect();
    }

    pub fn get_shndx(&self, esym: Sym, idx: i32) -> usize {
        assert!(
            idx >= 0
                && idx
                    < (unsafe { self.input_file.as_ref() }.unwrap().elf_syms.len() as usize)
                        .try_into()
                        .unwrap()
        );
        if esym.shndx == SHN_XINDEX {
            return self.symbol_shndx_sec[idx as usize] as usize;
        }
        return esym.shndx as usize;
    }

    pub fn resolve_symbols(&'a mut self) {
        for i in 0..unsafe { self.input_file.as_ref().unwrap().first_global.unwrap() } {
            let sym = unsafe { &self.input_file.as_ref().unwrap().symbols }[i as usize];
            let esym = unsafe { self.input_file.as_ref().unwrap() }.elf_syms[i as usize];

            if esym.is_undef() {
                continue;
            }

            let mut isec: Option<*mut InputSection> = None;
            if !esym.is_abs() {
                isec = Some(self.get_section(esym, i.try_into().unwrap()))
            }

            if unsafe { sym.as_ref().unwrap().object_file.is_none() } {
                let sym = unsafe { sym.as_mut().unwrap() };
                sym.object_file = Some((self as *const ObjectFile).cast_mut());
                sym.value = esym.val;
                sym.symidx = i as i32;
                sym.set_input_section(isec.unwrap());
            }
        }
    }

    pub fn get_section(&'a self, esym: Sym, idx: usize) -> *mut InputSection {
        self.input_sections[self.get_shndx(esym, idx.try_into().unwrap())].unwrap()
    }

    pub fn mark_live_objects(&self, _ctx: &Context, mut feeder: impl FnMut()) {
        assert!(unsafe { self.input_file.as_ref().unwrap().is_alive });

        let fg = unsafe { self.input_file.as_ref().unwrap().first_global.unwrap() };
        let len_es = unsafe { self.input_file.as_ref().unwrap().elf_syms.len() };

        for i in fg..(len_es as u64) as i64 {
            let sym = unsafe { &self.input_file.as_ref().unwrap().symbols }[i as usize];
            let esym = unsafe { self.input_file.as_ref().unwrap().elf_syms[i as usize] };

            if unsafe { sym.as_ref().unwrap().object_file.is_none() } {
                continue;
            }

            if esym.is_undef()
                && !unsafe {
                    sym.as_ref()
                        .unwrap()
                        .input_section
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .is_alive
                }
            {
                unsafe {
                    sym.as_ref()
                        .unwrap()
                        .input_section
                        .unwrap()
                        .as_mut()
                        .unwrap()
                }
                .is_alive = true;
                let _ = &feeder();
            }
        }
    }

    pub fn clear_symbols(&self) {
        for sym in unsafe { &self.input_file.as_ref().unwrap().symbols } {
            if unsafe { sym.as_ref().unwrap().object_file.is_none() } {
                unsafe { sym.as_ref().unwrap().clear() }
            }
        }
    }

    pub fn initilize_mergeable_sections(&mut self, ctx: &mut Context) {
        for i in 0..unsafe { self.input_file.as_ref().unwrap().symbols.len() } {
            let isec = self.input_sections[i];
            if isec.is_none()
                && unsafe { isec.unwrap().as_ref().unwrap().is_alive }
                && unsafe { isec.unwrap().as_ref().unwrap().shdr().flags } & SHF_MERGE != 0
            {
                self.mergeable_sections[i] = Some(self.split_section(ctx, isec.unwrap()));
                unsafe { isec.unwrap().as_mut().unwrap().is_alive = false };
            }
        }
    }

    pub fn find_null(data: &[u8], ent_size: usize) -> isize {
        if ent_size == 1 {
            return data.binary_search(&0u8).unwrap() as isize;
        }

        for i in 0..((data.len() - ent_size) / ent_size) {
            let bs = &data[i..(i + ent_size)];
            if all_zeros(bs) {
                return i.try_into().unwrap();
            }
        }

        return -1;
    }

    pub fn split_section(
        &self,
        ctx: &mut Context,
        isec: *mut InputSection,
    ) -> *mut MergeableSection {
        let mut m = MergeableSection::new();
        let shdr = unsafe { isec.as_ref().unwrap() }.shdr();

        m.parent = get_merged_section_instance(
            ctx,
            unsafe { isec.as_ref().unwrap() }.name(),
            shdr.shdr_type,
            shdr.flags,
        );
        m.p2_align = unsafe { isec.as_ref().unwrap() }.p2_align;

        let mut data = unsafe { isec.as_ref().unwrap() }.contents;
        let mut offset: usize = 0usize;

        if shdr.flags & SHF_STRINGS != 0 {
            for _i in 0..data.len() {
                let end = ObjectFile::find_null(data, shdr.ent_size.try_into().unwrap());
                if end == -1 {
                    fatal("string is not null terminated")
                }

                let sz = end + shdr.ent_size as isize;
                let sub_str = &data[..(sz as usize)];
                m.strs.push(String::from_utf8(sub_str.to_vec()).unwrap());
                m.frag_offsets.push(offset);
                offset += sz as usize;
            }
        } else {
            if data.len() % shdr.ent_size as usize != 0 {
                let sub_str = &data[..shdr.ent_size as usize];
                data = &data[shdr.ent_size as usize..];
                m.strs.push(String::from_utf8(sub_str.to_vec()).unwrap());
                m.frag_offsets.push(offset);
                offset += shdr.ent_size as usize;
                println!("{:?}{:?}", offset, data);
            }
        }
        return Box::leak(Box::new(m));
    }

    pub fn register_section_pieces(&self) {
        for m in &self.mergeable_sections {
            if m.is_none() {
                continue;
            }

            for i in 0..unsafe { m.unwrap().as_ref().unwrap().strs.len() } {
                let fs = unsafe {
                    m.unwrap()
                        .as_ref()
                        .unwrap()
                        .parent
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .insert(
                            &m.unwrap().as_ref().unwrap().strs[i],
                            m.unwrap().as_ref().unwrap().p2_align.into(),
                        )
                };
                unsafe { m.unwrap().as_mut().unwrap().fragments.push(fs.unwrap()) };
            }

            for i in 0..unsafe { self.input_file.as_ref().unwrap().elf_syms.len() } {
                let sym = unsafe { self.input_file.as_ref().unwrap().symbols[i] };
                let esym = unsafe { &self.input_file.as_ref().unwrap().elf_syms }[i];

                if esym.is_abs() || esym.is_undef() || esym.is_common() {
                    continue;
                }

                let m = self.mergeable_sections[self.get_shndx(esym, i.try_into().unwrap())];
                if m.is_none() {
                    continue;
                }

                let (frag, frag_offset) = unsafe {
                    m.unwrap()
                        .as_ref()
                        .unwrap()
                        .get_fragment(esym.val.try_into().unwrap())
                };
                if frag.is_none() {
                    fatal("bad symbol value")
                }
                unsafe { sym.as_mut().unwrap().set_section_fragment(frag.unwrap()) };
                unsafe { sym.as_mut().unwrap().value = frag_offset as u64 };
            }
        }
    }

    pub fn skip_eh_fragment_sections(&self) {
        for isec in &self.input_sections {
            if !isec.unwrap().is_null()
                && unsafe { isec.unwrap().as_ref().unwrap().is_alive }
                && unsafe { isec.unwrap().as_ref().unwrap().name() } == ".eh_frame"
            {
                unsafe { isec.unwrap().as_mut().unwrap().is_alive = false }
            }
        }
    }

    pub fn scan_relocations(&self) {
        for isec in &self.input_sections {
            if !isec.unwrap().is_null()
                && unsafe { isec.unwrap().as_ref().unwrap().is_alive }
                && unsafe { isec.unwrap().as_ref().unwrap().shdr().flags } & SHF_ALLOC != 0
            {
                unsafe { isec.unwrap().as_mut().unwrap().scan_relocations() }
            }
        }
    }
}
