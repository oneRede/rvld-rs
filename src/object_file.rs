use std::vec;

use crate::{
    context::Context,
    elf::{
        elf_get_name, Shdr, Sym, SHN_XINDEX, SHT_GROUP, SHT_NULL, SHT_REL, SHT_RELA, SHT_STRTAB,
        SHT_SYMTAB, SHT_SYMTAB_SHNDX,
    },
    file::ElfFile,
    input_file::{new_input_file, InputFile},
    input_section::InputSection,
    mergeablesection::MergeableSection,
    symbol::Symbol,
};

#[allow(dead_code)]
pub struct ObjectFile<'a> {
    pub input_file: *mut InputFile<'a>,
    pub symtab_sec: Option<Shdr>,
    pub symbol_shndx_sec: Vec<u32>,
    pub input_sections: Vec<*mut InputSection<'a>>,
    pub mergeable_sections: Vec<MergeableSection>,
}

#[allow(dead_code)]
pub fn new_object_file(elf_file: ElfFile, _is_alive: bool) -> ObjectFile {
    let input_file = new_input_file(elf_file);
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

    pub fn initialize_sections(&'a mut self) {
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
                    self.input_sections[i] = Box::leak(Box::new(InputSection::new(self, i)));
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
                sym.object_file = Some(self as *const ObjectFile);
                sym.value = esym.val;
                sym.symidx = i as i32;
                sym.set_input_section(isec.unwrap());
            }
        }
    }

    pub fn get_section(&'a self, esym: Sym, idx: usize) -> *mut InputSection {
        self.input_sections[self.get_shndx(esym, idx.try_into().unwrap())]
    }

    pub fn mark_live_objects(&self, _ctx: Context, feeder: fn(*const ObjectFile)) {
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
                feeder(unsafe { sym.as_ref().unwrap().object_file.unwrap() })
            }
        }
    }
}

// func (o *ObjectFile) ClearSymbols() {
// 	for _, sym := range o.Symbols[o.FirstGlobal:] {
// 		if sym.File == o {
// 			sym.Clear()
// 		}
// 	}
// }

// func (o *ObjectFile) InitializeMergeableSections(ctx *Context) {
// 	o.MergeableSections = make([]*MergeableSection, len(o.Sections))
// 	for i := 0; i < len(o.Sections); i++ {
// 		isec := o.Sections[i]
// 		if isec != nil && isec.IsAlive &&
// 			isec.Shdr().Flags&uint64(elf.SHF_MERGE) != 0 {
// 			o.MergeableSections[i] = splitSection(ctx, isec)
// 			isec.IsAlive = false
// 		}
// 	}
// }

// func findNull(data []byte, entSize int) int {
// 	if entSize == 1 {
// 		return bytes.Index(data, []byte{0})
// 	}

// 	for i := 0; i <= len(data)-entSize; i += entSize {
// 		bs := data[i : i+entSize]
// 		if utils.AllZeros(bs) {
// 			return i
// 		}
// 	}

// 	return -1
// }

// func splitSection(ctx *Context, isec *InputSection) *MergeableSection {
// 	m := &MergeableSection{}
// 	shdr := isec.Shdr()

// 	m.Parent = GetMergedSectionInstance(ctx, isec.Name(), shdr.Type,
// 		shdr.Flags)
// 	m.P2Align = isec.P2Align

// 	data := isec.Contents
// 	offset := uint64(0)
// 	if shdr.Flags&uint64(elf.SHF_STRINGS) != 0 {
// 		for len(data) > 0 {
// 			end := findNull(data, int(shdr.EntSize))
// 			if end == -1 {
// 				utils.Fatal("string is not null terminated")
// 			}

// 			sz := uint64(end) + shdr.EntSize
// 			substr := data[:sz]
// 			data = data[sz:]
// 			m.Strs = append(m.Strs, string(substr))
// 			m.FragOffsets = append(m.FragOffsets, uint32(offset))
// 			offset += sz
// 		}
// 	} else {
// 		if uint64(len(data))%shdr.EntSize != 0 {
// 			utils.Fatal("section size is not multiple of entsize")
// 		}

// 		for len(data) > 0 {
// 			substr := data[:shdr.EntSize]
// 			data = data[shdr.EntSize:]
// 			m.Strs = append(m.Strs, string(substr))
// 			m.FragOffsets = append(m.FragOffsets, uint32(offset))
// 			offset += shdr.EntSize
// 		}
// 	}

// 	return m
// }

// func (o *ObjectFile) RegisterSectionPieces() {
// 	for _, m := range o.MergeableSections {
// 		if m == nil {
// 			continue
// 		}

// 		m.Fragments = make([]*SectionFragment, 0, len(m.Strs))
// 		for i := 0; i < len(m.Strs); i++ {
// 			m.Fragments = append(m.Fragments,
// 				m.Parent.Insert(m.Strs[i], uint32(m.P2Align)))
// 		}
// 	}

// 	for i := 1; i < len(o.ElfSyms); i++ {
// 		sym := o.Symbols[i]
// 		esym := &o.ElfSyms[i]

// 		if esym.IsAbs() || esym.IsUndef() || esym.IsCommon() {
// 			continue
// 		}

// 		m := o.MergeableSections[o.GetShndx(esym, i)]
// 		if m == nil {
// 			continue
// 		}

// 		frag, fragOffset := m.GetFragment(uint32(esym.Val))
// 		if frag == nil {
// 			utils.Fatal("bad symbol value")
// 		}
// 		sym.SetSectionFragment(frag)
// 		sym.Value = uint64(fragOffset)
// 	}
// }
