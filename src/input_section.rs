use crate::{
    context::Context,
    elf::{
        self, elf_get_name, RRicsv, Rela, Shdr, RELA_SIZE, R_RISCV_32, R_RISCV_64, R_RISCV_BRANCH,
        R_RISCV_CALL, R_RISCV_CALL_PLT, R_RISCV_HI20, R_RISCV_JAL, R_RISCV_LO12_I, R_RISCV_LO12_S,
        R_RISCV_NONE, R_RISCV_PCREL_HI20, R_RISCV_RELAX, R_RISCV_TLS_GOT_HI20,
        R_RISCV_TPREL_LO12_I, R_RISCV_TPREL_LO12_S, SHDR_SIZE, SHF_ALLOC, SHF_COMPRESSED,
        SHT_NOBITS,
    },
    object_file::ObjectFile,
    output_section::OutputSection,
    symbol::NEEDS_GOT_TP,
    utils::{bit, bits, read, read_slice, sign_extend, write},
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
    pub output_section: Option<*mut OutputSection<'a>>,

    pub relsec_idx: u32,
    pub rels: *mut Vec<Rela>,
}

#[allow(dead_code)]
impl<'a> InputSection<'a> {
    pub fn new(
        ctx: &Context<'a>,
        name: String,
        object_file: *mut ObjectFile<'a>,
        shndx: usize,
    ) -> Self {
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
        let output_section =
            OutputSection::get_output_section(&ctx, name, shdr.shdr_type as u64, shdr.flags);
        InputSection {
            object_file,
            contents,
            shndx,
            sh_size: sh_size as u32,
            is_alive: true,
            p2_align,

            offset: 0,
            output_section: Some(output_section),

            relsec_idx: 0,
            rels: Box::leak(Box::new(vec![])),
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

    pub fn write_to(&mut self, ctx: &Context, buf: &mut [u8]) {
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

    pub fn apply_reloc_alloc(&mut self, ctx: &Context, base: &mut [u8]) {
        let symbols = unsafe {
            &self
                .object_file
                .as_ref()
                .unwrap()
                .input_file
                .as_ref()
                .unwrap()
                .symbols
        };
        let rels = self.get_rels();

        for i in 0..unsafe { rels.unwrap().as_ref().unwrap().len() } {
            let rel = unsafe { rels.unwrap().as_ref().unwrap() }[i];
            if rel.ty == R_RISCV_NONE || rel.ty == R_RISCV_RELAX {
                continue;
            }

            let sym = symbols[rel.sym as usize];
            let loc = &mut base[rel.offset as usize..];

            if unsafe { sym.as_ref().unwrap().object_file.is_none() } {
                continue;
            }

            let s: u64 = unsafe { sym.as_ref().unwrap().get_addr() };
            let a = rel.addend as u64;
            let p = self.get_addr() + rel.offset;

            match rel.ty as RRicsv {
                R_RISCV_32 => write(loc, (s + a) as u32),
                R_RISCV_64 => write(loc, s + a),
                R_RISCV_BRANCH => write_b_type(loc, (s + a - p) as u32),
                R_RISCV_JAL => write_j_type(loc, (s + a - p) as u32),
                R_RISCV_CALL_PLT | R_RISCV_CALL => {
                    let val = s + a - p;
                    write_u_type(loc, val as u32);
                    write_i_type(&mut loc[4..], val as u32);
                }
                R_RISCV_TLS_GOT_HI20 => write(loc, unsafe {
                    sym.as_ref().unwrap().get_got_tp_addr(ctx) + a - p
                }),
                R_RISCV_PCREL_HI20 => write(loc, s + a - p),
                R_RISCV_HI20 => write(loc, s + a),
                R_RISCV_LO12_S | R_RISCV_LO12_I => {
                    let val = s + a;
                    if rel.ty == R_RISCV_LO12_I as u32 {
                        write_i_type(loc, val as u32)
                    } else {
                        write_s_type(loc, val as u32)
                    }
                    if sign_extend(val, 11) == val {
                        set_rs1(loc, 0)
                    }
                }
                R_RISCV_TPREL_LO12_I | R_RISCV_TPREL_LO12_S => {
                    let val = s + a - ctx.tp_addr;
                    if rel.ty == R_RISCV_TPREL_LO12_I as u32 {
                        write_i_type(loc, val as u32);
                    } else {
                        write_s_type(loc, val as u32);
                    }

                    if sign_extend(val, 11) == val {
                        set_rs1(loc, 4);
                    }
                }
                _ => {}
            }

            for i in 0..unsafe { rels.unwrap().as_ref().unwrap().len() } {
                match unsafe { rels.unwrap().as_ref().unwrap()[i].ty } as RRicsv {
                    R_RISCV_PCREL_HI20 | R_RISCV_TLS_GOT_HI20 => {
                        let loc = &mut base
                            [unsafe { rels.unwrap().as_ref().unwrap() }[i].offset as usize..];
                        let val = read::<u32>(&loc);
                        write(
                            loc,
                            read::<u32>(
                                &self.contents[unsafe { rels.unwrap().as_ref().unwrap() }[i].offset
                                    as usize..],
                            ),
                        );
                        write_u_type(loc, val)
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn get_rels(&mut self) -> Option<*mut Vec<Rela>> {
        if self.relsec_idx == u32::MAX || !unsafe { self.rels.as_ref().unwrap().is_empty() } {
            return Some(self.rels);
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
        let mut _rels = unsafe { self.rels.as_mut().unwrap() };
        _rels = &mut read_slice::<Rela>(bs.as_mut_slice(), RELA_SIZE);
        None
    }

    pub fn get_addr(&self) -> u64 {
        unsafe {
            self.output_section
                .unwrap()
                .as_ref()
                .unwrap()
                .chunk
                .as_ref()
                .unwrap()
                .shdr
                .addr
                + self.offset as u64
        }
    }

    pub fn scan_relocations(&mut self) {
        for rel in unsafe { self.get_rels().unwrap().as_ref().unwrap() } {
            let sym = unsafe {
                &self
                    .object_file
                    .as_ref()
                    .unwrap()
                    .input_file
                    .as_ref()
                    .unwrap()
                    .symbols[rel.sym as usize]
            };
            if unsafe { sym.as_ref().unwrap().object_file.is_none() } {
                continue;
            }
            if rel.ty == elf::R_RISCV_TLS_GOT_HI20 as u32 {
                unsafe { sym.as_mut().unwrap().flags |= NEEDS_GOT_TP }
            }
        }
    }
}

#[allow(dead_code)]
fn i_type(val: u32) -> u32 {
    return val << 20;
}

#[allow(dead_code)]
fn s_type(val: u32) -> u32 {
    (bits(val, 11, 5) << 25 | bits(val, 4, 0) << 7) as u32
}

#[allow(dead_code)]
pub fn b_type(val: u32) -> u32 {
    bit(val, 12) << 31 | bits(val, 10, 5) << 25 | bits(val, 4, 1) << 8 | bit(val, 11) << 7
}

#[allow(dead_code)]
fn u_type(val: u32) -> u32 {
    (val + 0x800) & 0xffff_f000
}

#[allow(dead_code)]
fn j_type(val: u32) -> u32 {
    bit(val, 20) << 31 | bits(val, 10, 1) << 21 | bit(val, 11) << 20 | bits(val, 19, 12) << 12
}

#[allow(dead_code)]
fn cb_type(_val: u16) -> u16 {
    0
}

#[allow(dead_code)]
fn cj_type(_al: u16) -> u16 {
    0
}

#[allow(dead_code)]
fn write_b_type(loc: &mut [u8], val: u32) {
    let mask = 0b000000_11111_11111_111_00000_1111111 as u32;
    write(loc, read::<u32>(&loc) & mask | val)
}

#[allow(dead_code)]
fn write_j_type(loc: &mut [u8], val: u32) {
    let mask = 0b000000_00000_00000_000_11111_1111111 as u32;
    write(loc, read::<u32>(&loc) & mask | val)
}

#[allow(dead_code)]
fn write_i_type(loc: &mut [u8], val: u32) {
    let mask = 0b000000_11111_11111_111_00000_1111111 as u32;
    write(loc, read::<u32>(&loc) & mask | val)
}

#[allow(dead_code)]
fn write_s_type(loc: &mut [u8], val: u32) {
    let mask = 0b000000_11111_11111_111_00000_1111111 as u32;
    write(loc, read::<u32>(&loc) & mask | val)
}

#[allow(dead_code)]
fn write_u_type(loc: &mut [u8], val: u32) {
    let mask = 0b000000_11111_11111_111_00000_1111111 as u32;
    write(loc, read::<u32>(&loc) & mask | val)
}

#[allow(dead_code)]
fn set_rs1(loc: &mut [u8], rs1: u32) {
    write(
        loc,
        read::<u32>(&loc) & 0b111111_11111_00000_111_11111_1111111,
    );
    write(loc, read::<u32>(&loc) | rs1 << 15);
}