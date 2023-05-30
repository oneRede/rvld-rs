use std::{process::exit};

use crate::elf::{Ehdr, Shdr};

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
fn assert(con: bool) {
    if !con {
        fatal("assert failed")
    }
}

// func Read[T any](data []byte) (val T) {
// 	reader := bytes.NewReader(data)
// 	err := binary.Read(reader, binary.LittleEndian, &val)
// 	MustNo(err)
// 	return
// }

#[allow(dead_code)]
pub fn read_ehdr(data: &[u8]) -> Ehdr {
    return unsafe{*(data.as_ptr() as *mut Ehdr)}
}

#[allow(dead_code)]
pub fn read_shdr(data: &[u8]) -> Shdr {
    return unsafe{*(data.as_ptr() as *const Shdr)}
}

#[allow(dead_code)]
pub fn read<T: Copy>(data: &mut [u8]) -> T {
    return unsafe{*(data.as_ptr() as *const T)}

}