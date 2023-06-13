use std::{cell::UnsafeCell, env, process::exit};

mod context;
mod elf;
mod file;
mod file_type;
mod input_file;
mod machine_type;
mod magic;
mod object_file;
mod utils;

use context::Context;
use file::must_new_file;
use machine_type::MACHINE_TYPE_RISCV64;
use utils::fatal;
use crate::machine_type::{get_machine_type_from_contents, MACHINE_TYPE_NONE};

fn main() {
    let mut ctx = Context::new();
    let remaining = parse_args(&mut ctx);

    if ctx.args.emulation == MACHINE_TYPE_NONE {
        for file_name in &remaining {
            if file_name.starts_with("-") {
                continue;
            }

            let file = must_new_file(file_name);
            ctx.args.emulation = get_machine_type_from_contents(file.contents);

            if ctx.args.emulation != MACHINE_TYPE_NONE {
                break;
            }
        }
    }

    if ctx.args.emulation != MACHINE_TYPE_RISCV64 {
        fatal("unknown emulation type");
    }
    println!("{:?}", remaining);
}

#[allow(dead_code)]
fn parse_args<'a>(ctx: &mut Context) -> Vec<&'a str> {
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
