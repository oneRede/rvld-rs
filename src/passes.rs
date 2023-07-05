use std::{process::id, vec, cmp};

use crate::{
    chunk::{Chunk, Chunker},
    context::Context,
    elf::{IMAGE_BASE, SHF_ALLOC, SHF_TLS, SHT_NOBITS},
    file,
    input_section::InputSection,
    object_file::ObjectFile,
    output_ehdr::OutputEhdr,
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

    for j in 0..unsafe { chunks.as_ref().unwrap().len() } {
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
pub fn bin_sections(ctx: Context) {
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
pub fn compute_section_sizes(ctx: Context) {
    for osec in unsafe { ctx.output_sections.as_ref().unwrap() }{
        let mut offset = 0u64;
        let mut p2_align = 0u64;

        for isec in unsafe { osec.as_ref().unwrap().members.as_ref().unwrap() } {
            offset = align_to(offset, 1<<p2_align);
            unsafe { isec.as_mut().unwrap().offset = offset as u32 };
            offset += unsafe { isec.as_ref().unwrap().sh_size as u64};
            p2_align = cmp::max(p2_align, unsafe { isec.as_ref().unwrap().p2_align as u64})
        }

        unsafe { osec.as_mut().unwrap().chunk.as_mut().unwrap().shdr.size = offset };
        unsafe { osec.as_mut().unwrap().chunk.as_mut().unwrap().shdr.addr_align = 1 << p2_align };
    }
}

#[allow(dead_code)]
pub fn sort_output_sections(ctx: Context){
    let rank = |chunk: Chunk| -> u32{
        let ty = chunk.get_shdr().shdr_type;
        let flags = chunk.get_shdr().flags;

        if flags&SHF_ALLOC ==0{
            return u32::MAX - 1
        }
        // if chunk == ctx.shdr {
        //     return i32::MAX
        // }
        0
    };
}

#[allow(dead_code)]
pub fn compute_merged_sections_size(ctx: Context){
    for osec in ctx.merged_sections {
        unsafe { osec.as_mut().unwrap().assign_offsets() }
    }
}

#[allow(dead_code)]
pub fn scan_relocations(ctx: Context){
    for file in ctx.objs{
        
    }
}
// func SetOutputSectionOffsets(ctx *Context) uint64 {
// 	addr := IMAGE_BASE
// 	for _, chunk := range ctx.Chunks {
// 		if chunk.GetShdr().Flags&uint64(elf.SHF_ALLOC) == 0 {
// 			continue
// 		}

// 		addr = utils.AlignTo(addr, chunk.GetShdr().AddrAlign)
// 		chunk.GetShdr().Addr = addr

// 		if !isTbss(chunk) {
// 			addr += chunk.GetShdr().Size
// 		}
// 	}

// 	i := 0
// 	first := ctx.Chunks[0]
// 	for {
// 		shdr := ctx.Chunks[i].GetShdr()
// 		shdr.Offset = shdr.Addr - first.GetShdr().Addr
// 		i++

// 		if i >= len(ctx.Chunks) ||
// 			ctx.Chunks[i].GetShdr().Flags&uint64(elf.SHF_ALLOC) == 0 {
// 			break
// 		}
// 	}

// 	lastShdr := ctx.Chunks[i-1].GetShdr()
// 	fileoff := lastShdr.Offset + lastShdr.Size

// 	for ; i < len(ctx.Chunks); i++ {
// 		shdr := ctx.Chunks[i].GetShdr()
// 		fileoff = utils.AlignTo(fileoff, shdr.AddrAlign)
// 		shdr.Offset = fileoff
// 		fileoff += shdr.Size
// 	}

// 	ctx.Phdr.UpdateShdr(ctx)
// 	return fileoff
// }

// func BinSections(ctx *Context) {
// 	group := make([][]*InputSection, len(ctx.OutputSections))
// 	for _, file := range ctx.Objs {
// 		for _, isec := range file.Sections {
// 			if isec == nil || !isec.IsAlive {
// 				continue
// 			}

// 			idx := isec.OutputSection.Idx
// 			group[idx] = append(group[idx], isec)
// 		}
// 	}

// 	for idx, osec := range ctx.OutputSections {
// 		osec.Members = group[idx]
// 	}
// }

// func CollectOutputSections(ctx *Context) []Chunker {
// 	osecs := make([]Chunker, 0)
// 	for _, osec := range ctx.OutputSections {
// 		if len(osec.Members) > 0 {
// 			osecs = append(osecs, osec)
// 		}
// 	}

// 	for _, osec := range ctx.MergedSections {
// 		if osec.Shdr.Size > 0 {
// 			osecs = append(osecs, osec)
// 		}
// 	}

// 	return osecs
// }

// func ComputeSectionSizes(ctx *Context) {
// 	for _, osec := range ctx.OutputSections {
// 		offset := uint64(0)
// 		p2align := int64(0)

// 		for _, isec := range osec.Members {
// 			offset = utils.AlignTo(offset, 1<<isec.P2Align)
// 			isec.Offset = uint32(offset)
// 			offset += uint64(isec.ShSize)
// 			p2align = int64(math.Max(float64(p2align), float64(isec.P2Align)))
// 		}

// 		osec.Shdr.Size = offset
// 		osec.Shdr.AddrAlign = 1 << p2align
// 	}
// }

// func SortOutputSections(ctx *Context) {
// 	rank := func(chunk Chunker) int32 {
// 		typ := chunk.GetShdr().Type
// 		flags := chunk.GetShdr().Flags

// 		if flags&uint64(elf.SHF_ALLOC) == 0 {
// 			return math.MaxInt32 - 1
// 		}
// 		if chunk == ctx.Shdr {
// 			return math.MaxInt32
// 		}
// 		if chunk == ctx.Ehdr {
// 			return 0
// 		}
// 		if chunk == ctx.Phdr {
// 			return 1
// 		}
// 		if typ == uint32(elf.SHT_NOTE) {
// 			return 2
// 		}

// 		b2i := func(b bool) int {
// 			if b {
// 				return 1
// 			}
// 			return 0
// 		}

// 		writeable := b2i(flags&uint64(elf.SHF_WRITE) != 0)
// 		notExec := b2i(flags&uint64(elf.SHF_EXECINSTR) == 0)
// 		notTls := b2i(flags&uint64(elf.SHF_TLS) == 0)
// 		isBss := b2i(typ == uint32(elf.SHT_NOBITS))

// 		return int32(writeable<<7 | notExec<<6 | notTls<<5 | isBss<<4)
// 	}

// 	sort.SliceStable(ctx.Chunks, func(i, j int) bool {
// 		return rank(ctx.Chunks[i]) < rank(ctx.Chunks[j])
// 	})
// }

// func ComputeMergedSectionSizes(ctx *Context) {
// 	for _, osec := range ctx.MergedSections {
// 		osec.AssignOffsets()
// 	}
// }

// func ScanRelocations(ctx *Context) {
// 	for _, file := range ctx.Objs {
// 		file.ScanRelocations()
// 	}

// 	syms := make([]*Symbol, 0)
// 	for _, file := range ctx.Objs {
// 		for _, sym := range file.Symbols {
// 			if sym.File == file && sym.Flags != 0 {
// 				syms = append(syms, sym)
// 			}
// 		}
// 	}

// 	for _, sym := range syms {
// 		if sym.Flags&NeedsGotTp != 0 {
// 			ctx.Got.AddGotTpSymbol(sym)
// 		}

// 		sym.Flags = 0
// 	}
// }
