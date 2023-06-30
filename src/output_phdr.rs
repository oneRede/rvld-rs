// package linker

// import (
// 	"debug/elf"
// 	"github.com/ksco/rvld/pkg/utils"
// 	"math"
// )

// type OutputPhdr struct {
// 	Chunk

// 	Phdrs []Phdr
// }

// func NewOutputPhdr() *OutputPhdr {
// 	o := &OutputPhdr{Chunk: NewChunk()}
// 	o.Shdr.Flags = uint64(elf.SHF_ALLOC)
// 	o.Shdr.AddrAlign = 8
// 	return o
// }

// func toPhdrFlags(chunk Chunker) uint32 {
// 	ret := uint32(elf.PF_R)
// 	write := chunk.GetShdr().Flags&uint64(elf.SHF_WRITE) != 0
// 	if write {
// 		ret |= uint32(elf.PF_W)
// 	}
// 	if chunk.GetShdr().Flags&uint64(elf.SHF_EXECINSTR) != 0 {
// 		ret |= uint32(elf.PF_X)
// 	}
// 	return ret
// }

// func createPhdr(ctx *Context) []Phdr {
// 	vec := make([]Phdr, 0)
// 	define := func(typ, flags uint64, minAlign int64, chunk Chunker) {
// 		vec = append(vec, Phdr{})
// 		phdr := &vec[len(vec)-1]
// 		phdr.Type = uint32(typ)
// 		phdr.Flags = uint32(flags)
// 		phdr.Align = uint64(math.Max(
// 			float64(minAlign),
// 			float64(chunk.GetShdr().AddrAlign)))
// 		phdr.Offset = chunk.GetShdr().Offset
// 		if chunk.GetShdr().Type == uint32(elf.SHT_NOBITS) {
// 			phdr.FileSize = 0
// 		} else {
// 			phdr.FileSize = chunk.GetShdr().Size
// 		}
// 		phdr.VAddr = chunk.GetShdr().Addr
// 		phdr.PAddr = chunk.GetShdr().Addr
// 		phdr.MemSize = chunk.GetShdr().Size
// 	}

// 	push := func(chunk Chunker) {
// 		phdr := &vec[len(vec)-1]
// 		phdr.Align = uint64(math.Max(
// 			float64(phdr.Align),
// 			float64(chunk.GetShdr().AddrAlign)))
// 		if chunk.GetShdr().Type != uint32(elf.SHT_NOBITS) {
// 			phdr.FileSize = chunk.GetShdr().Addr +
// 				chunk.GetShdr().Size -
// 				phdr.VAddr
// 		}
// 		phdr.MemSize = chunk.GetShdr().Addr +
// 			chunk.GetShdr().Size -
// 			phdr.VAddr
// 	}

// 	define(uint64(elf.PT_PHDR), uint64(elf.PF_R), 8, ctx.Phdr)

// 	isTls := func(chunk Chunker) bool {
// 		return chunk.GetShdr().Flags&uint64(elf.SHF_TLS) != 0
// 	}

// 	isBss := func(chunk Chunker) bool {
// 		return chunk.GetShdr().Type == uint32(elf.SHT_NOBITS) && !isTls(chunk)
// 	}

// 	isNote := func(chunk Chunker) bool {
// 		shdr := chunk.GetShdr()
// 		return shdr.Type == uint32(elf.SHT_NOTE) &&
// 			shdr.Flags&uint64(elf.SHF_ALLOC) != 0
// 	}

// 	end := len(ctx.Chunks)
// 	for i := 0; i < end; {
// 		first := ctx.Chunks[i]
// 		i++
// 		if !isNote(first) {
// 			continue
// 		}

// 		flags := toPhdrFlags(first)
// 		alignment := first.GetShdr().AddrAlign
// 		define(uint64(elf.PT_NOTE), uint64(flags), int64(alignment), first)
// 		for i < end && isNote(ctx.Chunks[i]) &&
// 			toPhdrFlags(ctx.Chunks[i]) == flags {
// 			push(ctx.Chunks[i])
// 			i++
// 		}
// 	}

// 	{
// 		chunks := make([]Chunker, 0)
// 		for _, chunk := range ctx.Chunks {
// 			chunks = append(chunks, chunk)
// 		}

// 		chunks = utils.RemoveIf(chunks, func(chunk Chunker) bool {
// 			return isTbss(chunk)
// 		})

// 		end := len(chunks)
// 		for i := 0; i < end; {
// 			first := chunks[i]
// 			i++

// 			if first.GetShdr().Flags&uint64(elf.SHF_ALLOC) == 0 {
// 				break
// 			}

// 			flags := toPhdrFlags(first)
// 			define(uint64(elf.PT_LOAD), uint64(flags), PageSize, first)

// 			if !isBss(first) {
// 				for i < end && !isBss(chunks[i]) &&
// 					toPhdrFlags(chunks[i]) == flags {
// 					push(chunks[i])
// 					i++
// 				}
// 			}

// 			for i < end && isBss(chunks[i]) &&
// 				toPhdrFlags(chunks[i]) == flags {
// 				push(chunks[i])
// 				i++
// 			}
// 		}
// 	}

// 	for i := 0; i < len(ctx.Chunks); i++ {
// 		if !isTls(ctx.Chunks[i]) {
// 			continue
// 		}

// 		define(uint64(elf.PT_TLS), uint64(toPhdrFlags(ctx.Chunks[i])),
// 			1, ctx.Chunks[i])
// 		i++

// 		for i < len(ctx.Chunks) && isTls(ctx.Chunks[i]) {
// 			push(ctx.Chunks[i])
// 			i++
// 		}

// 		phdr := &vec[len(vec)-1]
// 		ctx.TpAddr = phdr.VAddr
// 	}

// 	return vec
// }

// func (o *OutputPhdr) UpdateShdr(ctx *Context) {
// 	o.Phdrs = createPhdr(ctx)
// 	o.Shdr.Size = uint64(len(o.Phdrs)) * uint64(PhdrSize)
// }

// func (o *OutputPhdr) CopyBuf(ctx *Context) {
// 	utils.Write(ctx.Buf[o.Shdr.Offset:], o.Phdrs)
// }

use std::cmp;

use crate::{
    chunk::{Chunk, Chunker},
    elf::{Phdr, Shdr, PF_W, PF_X, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE, SHT_NOBITS, PT_PHDR}, 
    context::Context,
};

#[allow(dead_code)]
pub struct OutputPhdr {
    pub chunk: Chunk,
    pub phdrs: Vec<Phdr>,
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
            chunk: chunk,
            phdrs: vec![],
        }
    }

    pub fn to_phdr_flags(&self, chunk: impl Chunker) -> u32 {
        let mut ret = PF_W;
        let write = chunk.get_shdr().flags & SHF_WRITE != 0;
        if write {
            ret |= PF_W;
        }
        if chunk.get_shdr().flags & SHF_EXECINSTR != 0 {
            ret |= PF_X;
        }
        ret
    }

    pub fn create_phdr(&self, ctx:Context) {
        let mut vec: Vec<Phdr> = vec![];
        let define = |ty: u64, flags: u64, min_align: i64, chunk: Box<dyn Chunker>| {
            let mut phdr = Phdr::new();
            phdr.p_type = ty  as u32;
            phdr.flags = flags as u32;
            phdr.align = cmp::max(min_align as u64, self.chunk.get_shdr().addr_align);
            phdr.offset = self.chunk.get_shdr().offset;

            if self.chunk.get_shdr().shdr_type == SHT_NOBITS{
                phdr.file_size = 0;
            } else{
                phdr.file_size = self.chunk.get_shdr().size
            }

            phdr.v_addr = self.chunk.get_shdr().addr;
            phdr.p_addr = self.chunk.get_shdr().addr;
            phdr.mem_size = self.chunk.get_shdr().size;
            vec.push(phdr)
        };

        let push = |chunk: Chunk| {
            let mut phdr = Phdr::new();
            phdr.align = cmp::max(phdr.align, chunk.get_shdr().addr_align);
            if chunk.get_shdr().shdr_type != SHT_NOBITS {
                phdr.file_size = chunk.get_shdr().addr + chunk.get_shdr().size + phdr.v_addr;
            }
            phdr.mem_size = chunk.get_shdr().addr + chunk.get_shdr().size - phdr.v_addr;
            vec.push(phdr)
        };

        // define(PT_PHDR, PF_W.into(), 8,)
    }
}
