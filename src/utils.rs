use std::{
    mem,
    process::exit,
    slice, usize,
};

pub fn fatal(v: &str) {
    println!("rvld: fatal: {:?}", v);
    exit(1);
}

#[allow(dead_code)]
fn must_no(err: &str) {
    if err == "nil" {
        fatal(err)
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
    return unsafe { *(data.as_ptr() as *const T) };
}

#[allow(dead_code)]
pub fn write<T>(data: &mut [u8], e: T) {
    let buf = (&e as *const T) as *const u8;
    let buf = unsafe { slice::from_raw_parts(buf, mem::size_of::<T>()) };
    data.copy_from_slice(buf);
}

#[allow(dead_code)]
pub fn remove_prefix(s: &str, prefix: &str) -> (String, bool) {
    if s.starts_with(prefix) {
        let s = String::from(s.strip_prefix(prefix).unwrap());
        return (s, true);
    }
    return (s.to_string(), false);
}

#[allow(dead_code)]
pub fn all_zeros(bs: &[u8]) -> bool {
    let mut b = 0u8;

    for i in bs {
        b |= i
    }
    return b == 0;
}

#[allow(dead_code)]
pub fn remove_if<T>(elems: &Vec<T>, mut func: impl FnMut(&T) -> bool) -> Vec<&T> {
    let mut new_elems = vec![];
    for elem in elems.into_iter() {
        if func(elem) {
            continue;
        }
        new_elems.push(elem);
    }

    new_elems
}

#[allow(dead_code)]
pub fn align_to(val: u64, align: u64) -> u64 {
    if align == 0 {
        return val;
    }
    return (val + align - 1) & !(align - 1);
}

#[allow(dead_code)]
pub fn read_slice<T>(mut data: &mut [u8], sz: usize) -> Vec<T>
where
    T: Copy,
{
    let nums = data.len() / sz;
    let mut res: Vec<T> = vec![];

    for _i in 0..nums {
        res.push(read::<T>(data));
        data = &mut data[sz..];
    }
    res
}

#[allow(dead_code)]
pub fn bit(val:u32, pos: i32) -> u32 {
    (val >> pos) & 1
}

#[allow(dead_code)]
pub fn bits(val: u32, hi: usize, lo: usize) -> u32
{
    (val >> lo) & ((1 << ((hi - lo) + 1)) - 1)
}

#[allow(dead_code)]
pub fn sign_extend(val:u64, size: i32) -> u64{
    val << (63-size) >> (63 -size)
}

#[test]
fn test_remove_prefix() {
    let s = "1234567890".to_string();
    let prefix = "123456";
    let r = remove_prefix(&s, prefix);
    println!("{:?}", r);
}
