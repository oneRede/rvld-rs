use std::slice;

use crate::{
    context::Context,
    elf::{elf_get_name, Rela, Shdr, RELA_SIZE, SHDR_SIZE, SHF_ALLOC, SHF_COMPRESSED, SHT_NOBITS},
    object_file::ObjectFile,
    output_section::OutputSection,
    utils::read_slice,
};
#[allow(dead_code)]
pub struct InputSection<'a> {
    pub object_file: *mut ObjectFile<'a>,
    pub contents: &'static [u8],
    pub shndx: usize,
    pub sh_size: u32,
    pub is_alive: bool,
    pub p2_align: u8,

    pub offset: u32,
    pub output_section: Option<OutputSection<'a>>,

    pub relsec_idx: u32,
    pub rels: Vec<Rela>,
}

#[allow(dead_code)]
impl<'a> InputSection<'a> {
    pub fn new(object_file: *mut ObjectFile<'a>, shndx: usize) -> Self {
        let shdr = unsafe { (object_file.as_ref()).unwrap().input_file.as_ref().unwrap() }
            .elf_sections[shndx];
        let contents = &unsafe { (object_file.as_ref()).unwrap().input_file.as_ref().unwrap() }
            .file
            .contents[shdr.offset as usize..(shdr.offset + SHDR_SIZE as u64) as usize];
        assert!(shdr.flags & SHF_COMPRESSED == 0);
        let sh_size = shdr.size;
        let to_p2_align = |align: u64| -> u8 {
            if align == 0 {
                return 0;
            }
            return u64::trailing_zeros(align) as u8;
        };
        let p2_align = to_p2_align(shdr.addr_align);
        // todo!("ouput_section");
        InputSection {
            object_file,
            contents,
            shndx,
            sh_size: sh_size as u32,
            is_alive: true,
            p2_align,

            offset: 0,
            output_section: None,

            relsec_idx: 0,
            rels: vec![],
        }
    }

    pub fn shdr(&self) -> Shdr {
        assert!(
            self.shndx
                < unsafe {
                    (self.object_file.as_ref().unwrap())
                        .input_file
                        .as_ref()
                        .unwrap()
                }
                .elf_sections
                .len()
        );
        unsafe {
            (self.object_file.as_ref().unwrap())
                .input_file
                .as_ref()
                .unwrap()
        }
        .elf_sections[self.shndx]
    }

    pub fn name(&self) -> &str {
        elf_get_name(
            unsafe {
                (self.object_file.as_ref().unwrap())
                    .input_file
                    .as_ref()
                    .unwrap()
            }
            .sh_strtab
            .unwrap(),
            self.shdr().name,
        )
    }

    pub fn write_to(&mut self, ctx: Context, buf: &mut [u8]) {
        if self.shdr().shdr_type == SHT_NOBITS || self.sh_size == 0 {
            return;
        }
        let mut buf: Vec<u8> = buf.into_iter().map(|n| -> u8 { *n }).collect();
        let buf = buf.as_mut_slice();
        self.copy_contents(buf);

        if self.shdr().flags & SHF_ALLOC != 0 {
            self.apply_reloc_alloc(ctx, buf)
        }
    }

    pub fn copy_contents(&mut self, buf: &mut [u8]) {
        buf.copy_from_slice(self.contents)
    }

    pub fn apply_reloc_alloc(&self, ctx: Context, base: &[u8]) {}

    pub fn get_rels(&mut self) -> Option<&Vec<Rela>> {
        if self.relsec_idx == u32::MAX || !self.rels.is_empty() {
            return Some(&self.rels);
        }
        let bs = unsafe {
            self.object_file
                .as_ref()
                .unwrap()
                .input_file
                .as_ref()
                .unwrap()
                .get_bytes_from_shdr(
                    &self
                        .object_file
                        .as_ref()
                        .unwrap()
                        .input_file
                        .as_mut()
                        .unwrap()
                        .elf_sections[self.relsec_idx as usize],
                )
        };
        let mut bs: Vec<u8> = bs.into_iter().map(|n| -> u8 { *n }).collect();
        self.rels = read_slice::<Rela>(bs.as_mut_slice(), RELA_SIZE);
        None
    }
}

// package linker

// import (
// 	"debug/elf"
// 	"github.com/ksco/rvld/pkg/utils"
// 	"math"
// 	"math/bits"
// )

// type InputSection struct {
// 	File     *ObjectFile
// 	Contents []byte
// 	Shndx    uint32
// 	ShSize   uint32
// 	IsAlive  bool
// 	P2Align  uint8

// 	Offset        uint32
// 	OutputSection *OutputSection

// 	RelsecIdx uint32
// 	Rels      []Rela
// }

// func NewInputSection(ctx *Context, name string, file *ObjectFile, shndx uint32) *InputSection {
// 	s := &InputSection{
// 		File:      file,
// 		Shndx:     shndx,
// 		IsAlive:   true,
// 		Offset:    math.MaxUint32,
// 		RelsecIdx: math.MaxUint32,
// 		ShSize:    math.MaxUint32,
// 	}

// 	shdr := s.Shdr()
// 	s.Contents = file.File.Contents[shdr.Offset : shdr.Offset+shdr.Size]

// 	utils.Assert(shdr.Flags&uint64(elf.SHF_COMPRESSED) == 0)
// 	s.ShSize = uint32(shdr.Size)

// 	toP2Align := func(align uint64) uint8 {
// 		if align == 0 {
// 			return 0
// 		}
// 		return uint8(bits.TrailingZeros64(align))
// 	}
// 	s.P2Align = toP2Align(shdr.AddrAlign)

// 	s.OutputSection = GetOutputSection(
// 		ctx, name, uint64(shdr.Type), shdr.Flags)

// 	return s
// }

// func (i *InputSection) Shdr() *Shdr {
// 	utils.Assert(i.Shndx < uint32(len(i.File.ElfSections)))
// 	return &i.File.ElfSections[i.Shndx]
// }

// func (i *InputSection) Name() string {
// 	return ElfGetName(i.File.ShStrtab, i.Shdr().Name)
// }

// func (i *InputSection) WriteTo(ctx *Context, buf []byte) {
// 	if i.Shdr().Type == uint32(elf.SHT_NOBITS) || i.ShSize == 0 {
// 		return
// 	}

// 	i.CopyContents(buf)

// 	if i.Shdr().Flags&uint64(elf.SHF_ALLOC) != 0 {
// 		i.ApplyRelocAlloc(ctx, buf)
// 	}
// }

// func (i *InputSection) CopyContents(buf []byte) {
// 	copy(buf, i.Contents)
// }

// func (i *InputSection) GetRels() []Rela {
// 	if i.RelsecIdx == math.MaxUint32 || i.Rels != nil {
// 		return i.Rels
// 	}

// 	bs := i.File.GetBytesFromShdr(
// 		&i.File.InputFile.ElfSections[i.RelsecIdx])
// 	i.Rels = utils.ReadSlice[Rela](bs, RelaSize)
// 	return i.Rels
// }

// func (i *InputSection) GetAddr() uint64 {
// 	return i.OutputSection.Shdr.Addr + uint64(i.Offset)
// }

// func (i *InputSection) ScanRelocations() {
// 	for _, rel := range i.GetRels() {
// 		sym := i.File.Symbols[rel.Sym]
// 		if sym.File == nil {
// 			continue
// 		}

// 		if rel.Type == uint32(elf.R_RISCV_TLS_GOT_HI20) {
// 			sym.Flags |= NeedsGotTp
// 		}
// 	}
// }

// func (i *InputSection) ApplyRelocAlloc(ctx *Context, base []byte) {
// 	rels := i.GetRels()

// 	for a := 0; a < len(rels); a++ {
// 		rel := rels[a]
// 		if rel.Type == uint32(elf.R_RISCV_NONE) ||
// 			rel.Type == uint32(elf.R_RISCV_RELAX) {
// 			continue
// 		}

// 		sym := i.File.Symbols[rel.Sym]
// 		loc := base[rel.Offset:]

// 		if sym.File == nil {
// 			continue
// 		}

// 		S := sym.GetAddr()
// 		A := uint64(rel.Addend)
// 		P := i.GetAddr() + rel.Offset

// 		switch elf.R_RISCV(rel.Type) {
// 		case elf.R_RISCV_32:
// 			utils.Write[uint32](loc, uint32(S+A))
// 		case elf.R_RISCV_64:
// 			utils.Write[uint64](loc, S+A)
// 		case elf.R_RISCV_BRANCH:
// 			writeBtype(loc, uint32(S+A-P))
// 		case elf.R_RISCV_JAL:
// 			writeJtype(loc, uint32(S+A-P))
// 		case elf.R_RISCV_CALL, elf.R_RISCV_CALL_PLT:
// 			val := uint32(S + A - P)
// 			writeUtype(loc, val)
// 			writeItype(loc[4:], val)
// 		case elf.R_RISCV_TLS_GOT_HI20:
// 			utils.Write[uint32](loc, uint32(sym.GetGotTpAddr(ctx)+A-P))
// 		case elf.R_RISCV_PCREL_HI20:
// 			utils.Write[uint32](loc, uint32(S+A-P))
// 		case elf.R_RISCV_HI20:
// 			writeUtype(loc, uint32(S+A))
// 		case elf.R_RISCV_LO12_I, elf.R_RISCV_LO12_S:
// 			val := S + A
// 			if rel.Type == uint32(elf.R_RISCV_LO12_I) {
// 				writeItype(loc, uint32(val))
// 			} else {
// 				writeStype(loc, uint32(val))
// 			}

// 			if utils.SignExtend(val, 11) == val {
// 				setRs1(loc, 0)
// 			}
// 		case elf.R_RISCV_TPREL_LO12_I, elf.R_RISCV_TPREL_LO12_S:
// 			val := S + A - ctx.TpAddr
// 			if rel.Type == uint32(elf.R_RISCV_TPREL_LO12_I) {
// 				writeItype(loc, uint32(val))
// 			} else {
// 				writeStype(loc, uint32(val))
// 			}

// 			if utils.SignExtend(val, 11) == val {
// 				setRs1(loc, 4)
// 			}
// 		}
// 	}

// 	for a := 0; a < len(rels); a++ {
// 		switch elf.R_RISCV(rels[a].Type) {
// 		case elf.R_RISCV_PCREL_LO12_I, elf.R_RISCV_PCREL_LO12_S:
// 			sym := i.File.Symbols[rels[a].Sym]
// 			utils.Assert(sym.InputSection == i)
// 			loc := base[rels[a].Offset:]
// 			val := utils.Read[uint32](base[sym.Value:])

// 			if rels[a].Type == uint32(elf.R_RISCV_PCREL_LO12_I) {
// 				writeItype(loc, val)
// 			} else {
// 				writeStype(loc, val)
// 			}
// 		}
// 	}

// 	for a := 0; a < len(rels); a++ {
// 		switch elf.R_RISCV(rels[a].Type) {
// 		case elf.R_RISCV_PCREL_HI20, elf.R_RISCV_TLS_GOT_HI20:
// 			loc := base[rels[a].Offset:]
// 			val := utils.Read[uint32](loc)
// 			utils.Write[uint32](loc, utils.Read[uint32](i.Contents[rels[a].Offset:]))
// 			writeUtype(loc, val)
// 		}
// 	}
// }

// func itype(val uint32) uint32 {
// 	return val << 20
// }

// func stype(val uint32) uint32 {
// 	return utils.Bits(val, 11, 5)<<25 | utils.Bits(val, 4, 0)<<7
// }

// func btype(val uint32) uint32 {
// 	return utils.Bit(val, 12)<<31 | utils.Bits(val, 10, 5)<<25 |
// 		utils.Bits(val, 4, 1)<<8 | utils.Bit(val, 11)<<7
// }

// func utype(val uint32) uint32 {
// 	return (val + 0x800) & 0xffff_f000
// }

// func jtype(val uint32) uint32 {
// 	return utils.Bit(val, 20)<<31 | utils.Bits(val, 10, 1)<<21 |
// 		utils.Bit(val, 11)<<20 | utils.Bits(val, 19, 12)<<12
// }

// func cbtype(val uint16) uint16 {
// 	return utils.Bit(val, 8)<<12 | utils.Bit(val, 4)<<11 | utils.Bit(val, 3)<<10 |
// 		utils.Bit(val, 7)<<6 | utils.Bit(val, 6)<<5 | utils.Bit(val, 2)<<4 |
// 		utils.Bit(val, 1)<<3 | utils.Bit(val, 5)<<2
// }

// func cjtype(val uint16) uint16 {
// 	return utils.Bit(val, 11)<<12 | utils.Bit(val, 4)<<11 | utils.Bit(val, 9)<<10 |
// 		utils.Bit(val, 8)<<9 | utils.Bit(val, 10)<<8 | utils.Bit(val, 6)<<7 |
// 		utils.Bit(val, 7)<<6 | utils.Bit(val, 3)<<5 | utils.Bit(val, 2)<<4 |
// 		utils.Bit(val, 1)<<3 | utils.Bit(val, 5)<<2
// }

// func writeItype(loc []byte, val uint32) {
// 	mask := uint32(0b000000_00000_11111_111_11111_1111111)
// 	utils.Write[uint32](loc, (utils.Read[uint32](loc)&mask)|itype(val))
// }

// func writeStype(loc []byte, val uint32) {
// 	mask := uint32(0b000000_11111_11111_111_00000_1111111)
// 	utils.Write[uint32](loc, (utils.Read[uint32](loc)&mask)|stype(val))
// }

// func writeBtype(loc []byte, val uint32) {
// 	mask := uint32(0b000000_11111_11111_111_00000_1111111)
// 	utils.Write[uint32](loc, (utils.Read[uint32](loc)&mask)|btype(val))
// }

// func writeUtype(loc []byte, val uint32) {
// 	mask := uint32(0b000000_00000_00000_000_11111_1111111)
// 	utils.Write[uint32](loc, (utils.Read[uint32](loc)&mask)|utype(val))
// }

// func writeJtype(loc []byte, val uint32) {
// 	mask := uint32(0b000000_00000_00000_000_11111_1111111)
// 	utils.Write[uint32](loc, (utils.Read[uint32](loc)&mask)|jtype(val))
// }

// func setRs1(loc []byte, rs1 uint32) {
// 	utils.Write[uint32](loc, utils.Read[uint32](loc)&0b111111_11111_00000_111_11111_1111111)
// 	utils.Write[uint32](loc, utils.Read[uint32](loc)|(rs1<<15))
// }
