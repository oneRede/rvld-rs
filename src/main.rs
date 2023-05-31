use std::env;

use file::must_new_file;
use input_file::new_input_file;
use utils::{fatal, assert};

mod elf;
mod file;
mod magic;
mod input_file;
mod object_file;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        fatal("wrong args");
    }

    let file = must_new_file(&args[1]);
    let input_file = new_input_file(file);
    assert(input_file.elf_sections.len() == 11)
}
