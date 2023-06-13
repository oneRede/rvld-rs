use std::{cell::UnsafeCell, env, fmt::format, process::exit};

use context::Context;
use elf::elf_get_name;
use file::must_new_file;
use machine_type::MACHINE_TYPE_RISCV64;
use object_file::new_object_file;
use utils::fatal;

mod context;
mod elf;
mod file;
mod file_type;
mod input_file;
mod machine_type;
mod magic;
mod object_file;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() < 2 {
        fatal("wrong args");
    }

    let elf_file = must_new_file(&args[1]);
    let mut object_file = new_object_file(elf_file);
    object_file.parse();
    // assert!(object_file.input_file.elf_sections.len() == 11);
    // assert!(object_file.input_file.first_global == Some(12));
    // assert!(object_file.input_file.elf_syms.len() == 12);
    for sym in object_file.input_file.elf_syms.into_iter() {
        println!(
            "{:?}",
            elf_get_name(object_file.input_file.symbol_strtab.unwrap(), sym.name)
        )
    }
}

#[allow(dead_code)]
fn parse_args<'a>(mut ctx: Context) -> Vec<&'a str> {
    let f_args: Vec<String> = env::args().collect();
    let s_args: &[String] = Box::leak(Box::new(f_args));
    let args: UnsafeCell<&[String]> = UnsafeCell::new(s_args);

    let _args = args.get();
    unsafe { *_args = &(*args.get())[1..] }

    let dashes = |name: &str| -> Vec<String> {
        if name.len() == 1 {
            return vec!["-".to_string() + name];
        }
        return vec!["-".to_string() + &name, "-".to_string() + &name];
    };

    let arg = UnsafeCell::new("");
    let read_arg = |name: &str| -> bool {
        let _arg = arg.get();
        for opt in dashes(name) {
            let _args = args.get();
            if unsafe { (*_args).get(0) }.unwrap() == &opt {
                if unsafe { (*args.get()).len() } == 1 {
                    fatal(&format!("option -{}: argument missing", name));
                }

                unsafe { *_arg = { (*_args).get(1) }.unwrap() };
                unsafe { *_args = &(*args.get())[2..] }
                return true;
            }
            let mut prefix = String::from(&opt);
            if name.len() > 1 {
                prefix += "=";
            }
            if unsafe { (*_args).get(0) }.unwrap().starts_with(&prefix) {
                unsafe { *_arg = &{ (*_args).get(1) }.unwrap()[prefix.len()..] };
                unsafe { *_args = &(*args.get())[1..] }
                return true;
            }
        }
        false
    };

    let read_flag = |name: &str| -> bool {
        for opt in dashes(name) {
            if unsafe { (*_args).get(0) }.unwrap() == &opt {
                unsafe { *_args = &(*args.get())[1..] }
                return true;
            }
        }
        false
    };

    let mut remaining: Vec<&str> = vec![];
    loop {
        if unsafe { (*_args).len() } > 0 {
            break;
        }
        if read_flag("help") {
            format!("usage: {} [options] file...\n", s_args[0]);
            exit(0);
        }
        if read_arg("o") || read_arg("output") {
            ctx.args.output = unsafe { *arg.get() };
        } else if read_arg("v") || read_arg("version") {
            format!("rvld {}\n", "");
            exit(0);
        } else if read_arg("m") {
            if unsafe { *arg.get() } == "elf64riscv" {
                ctx.args.emulation = MACHINE_TYPE_RISCV64;
            } else {
                fatal(&format!("unknown -m argument: {}", unsafe { *arg.get() }));
            }
        } else if read_arg("L") {
            ctx.args
                .library_paths
                .push("".to_string() + unsafe { *arg.get() });
        } else if read_arg("l") {
            ctx.args
                .library_paths
                .push("-l".to_string() + unsafe { *arg.get() });
        } else if read_arg("sysroot")
            || read_arg("static")
            || read_arg("plugin")
            || read_arg("as-needed")
            || read_arg("start-group")
            || read_arg("hash-style")
            || read_arg("build-id")
            || read_arg("s")
            || read_arg("no-relax")
        {
            todo!()
        } else {
            if unsafe { (*_args).get(0) }.unwrap() == "-" {
                fatal(&format!("unknown command line option: {}", unsafe {
                    *arg.get()
                }));
            }
            remaining.push(unsafe { *arg.get() });
            let _args = args.get();
            unsafe { *_args = &(*args.get())[1..] }
        }
    }
    return remaining;
}
