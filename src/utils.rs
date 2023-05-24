use std::{process::exit};

use crate::elf::{Ehdr, Shdr};

pub fn fatal(v: &str) {
    println!("rvld: fatal: {:?}", v);
    exit(1);
}

fn must_no(err: &str) {
    if err == "nil" {
        fatal(err
        )
    }
}

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

pub fn read_ehdr(data: &mut Vec<u8>) -> Ehdr {
    return unsafe{*(data.as_mut_ptr() as *mut Ehdr)}
}

pub fn read_shdr(data: &mut Vec<u8>) -> Shdr {
    return unsafe{*(data.as_mut_ptr() as *mut Shdr)}
}

pub fn read<T: Copy>(data: &mut [u8]) -> T {
    return unsafe{*(data.as_mut_ptr() as *mut T)}

}