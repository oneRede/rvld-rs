use std::{process::exit};

use crate::elf::{Ehdr, Shdr, Sym};

pub fn fatal(v: &str) {
    println!("rvld: fatal: {:?}", v);
    exit(1);
}

#[allow(dead_code)]
fn must_no(err: &str) {
    if err == "nil" {
        fatal(err
        )
    }
}

#[allow(dead_code)]
pub fn assert(con: bool) {
    if !con {
        fatal("assert failed")
    }
}

#[allow(dead_code)]
pub fn read_ehdr(data: &[u8]) -> Ehdr {
    let size = std::mem::size_of::<Ehdr>();
    return unsafe{*(data[..size].as_ptr() as *mut Ehdr)}
}

#[allow(dead_code)]
pub fn read_shdr(data: &[u8]) -> Shdr {
    return unsafe{*(data.as_ptr() as *const Shdr)}
}

#[allow(dead_code)]
pub fn read_sym(data: &[u8]) -> Sym {
    return unsafe{*(data.as_ptr() as *const Sym)}
}

#[allow(dead_code)]
pub fn read_u16(data: &[u8]) -> u16 {
    return unsafe{*(data.as_ptr() as *const u16)}
}

#[allow(dead_code)]
pub fn read<T: Copy>(data: &mut [u8]) -> T {
    return unsafe{*(data.as_ptr() as *const T)}

}