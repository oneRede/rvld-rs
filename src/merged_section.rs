// package linker

// import "debug/elf"

// type MergedSection struct {
// 	Chunk
// 	Map map[string]*SectionFragment
// }

// func NewMergedSection(
// 	name string, flags uint64, typ uint32) *MergedSection {
// 	m := &MergedSection{
// 		Chunk: NewChunk(),
// 		Map:   make(map[string]*SectionFragment),
// 	}

// 	m.Name = name
// 	m.Shdr.Flags = flags
// 	m.Shdr.Type = typ
// 	return m
// }

// func GetMergedSectionInstance(
// 	ctx *Context, name string, typ uint32, flags uint64) *MergedSection {
// 	name = GetOutputName(name, flags)
// 	flags = flags & ^uint64(elf.SHF_GROUP) & ^uint64(elf.SHF_MERGE) &
// 		^uint64(elf.SHF_STRINGS) & ^uint64(elf.SHF_COMPRESSED)

// 	find := func() *MergedSection {
// 		for _, osec := range ctx.MergedSections {
// 			if name == osec.Name && flags == osec.Shdr.Flags &&
// 				typ == osec.Shdr.Type {
// 				return osec
// 			}
// 		}

// 		return nil
// 	}

// 	if osec := find(); osec != nil {
// 		return osec
// 	}

// 	osec := NewMergedSection(name, flags, typ)
// 	ctx.MergedSections = append(ctx.MergedSections, osec)
// 	return osec
// }

// func (m *MergedSection) Insert(
// 	key string, p2align uint32) *SectionFragment {
// 	frag, ok := m.Map[key]
// 	if !ok {
// 		frag = NewSectionFragment(m)
// 		m.Map[key] = frag
// 	}

// 	if frag.P2Align < p2align {
// 		frag.P2Align = p2align
// 	}

// 	return frag
// }

use std::{collections::HashMap, cell::UnsafeCell};

use crate::{chunk::Chunk, section_fragment::SectionFragment};

#[allow(dead_code)]
pub struct MergedSection<'a>{
    chunk: Chunk,
    map: HashMap<String, &'a SectionFragment<'a>>
}

impl<'a> MergedSection<'a> {
    #[allow(dead_code)]
    fn new(name: String, flags: u64, ty: u32) -> Self {
        let mut chunk = Chunk::new();
        chunk.name = name;
        chunk.shdr.flags =flags;
        chunk.shdr.shdr_type =ty;

        MergedSection { chunk: chunk, map: HashMap::new() }
    }
    
    #[allow(dead_code)]
    fn insert(&'a mut self, key: String, p2_align: u32) {
        let frag = self.map.get(&key);
        match frag {
            Some(v) => {
                if v.p2_align < p2_align{
                    self.map.insert(key, v);
                }
            },
            None => {
                let frag = SectionFragment::new(self);
                // self.map.insert(key, &frag);
            }
        }
    }

    #[allow(dead_code)]
    fn insert_v2(self, key: String, p2_align: u32) {
        let m = UnsafeCell::new(self);
        let _m = m.get();
    //     match frag {
    //         Some(v) => {
    //             if v.p2_align < p2_align{
    //                 self.map.insert(key, v);
    //             }
    //         },
    //         None => {
    //             let frag = SectionFragment::new(&self);
    //             self.map.insert(key, &frag);
    //         }
    //     }
    }
}


