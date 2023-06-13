use std::{process::exit};

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
pub fn read<T: Copy>(data: &[u8]) -> T {
    return unsafe{*(data.as_ptr() as *const T)}
}