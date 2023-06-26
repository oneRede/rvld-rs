use std::process::exit;

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
pub fn remove_if<T>(elems: &Vec<T>, mut func: impl FnMut(&T) -> bool) -> Vec<&T>{
    let mut new_elems = vec![];
    for elem in elems.into_iter() {
        if func(elem){
            continue;
        }
        new_elems.push(elem);
     }

    new_elems
}

#[allow(dead_code)]
pub fn align_to(val: u64, align: u64) -> u64 {
    if align == 0{
        return val
    }
    return (val + align-1) &! (align -1)
}

#[test]
fn test_remove_prefix() {
    let s = "1234567890".to_string();
    let prefix = "123456";
    let r = remove_prefix(&s, prefix);
    println!("{:?}", r);
}
