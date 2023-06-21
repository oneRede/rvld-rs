use std::{cell::UnsafeCell, env, process::exit};

mod archive;
mod context;
mod elf;
mod file;
mod file_type;
mod input;
mod input_file;
mod input_section;
mod machine_type;
mod magic;
mod object_file;
mod symbol;
mod chunk;
mod merged_section;
mod section_fragment;
mod utils;

use crate::{
    input::read_input_files,
    machine_type::{get_machine_type_from_contents, MACHINE_TYPE_NONE},
};
use context::Context;
use file::must_new_file;
use machine_type::MACHINE_TYPE_RISCV64;
use utils::fatal;

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

    read_input_files(&mut ctx, remaining);
    println!("{:?}", ctx.objs.len());

    for obj in ctx.objs {
        println!("{:?}", obj.input_file.file.name);
    }
}

#[allow(dead_code)]
fn parse_args<'a>(ctx: &mut Context) -> Vec<String> {
    let _f_args: Vec<String> = env::args().collect();
    let f_args: Vec<String> = vec![
        "./ld".to_string(),
        "-plugin".to_string(),
        "/usr/lib/gcc-cross/riscv64-linux-gnu/10/liblto_plugin.so".to_string(),
        "-plugin-opt=/usr/lib/gcc-cross/riscv64-linux-gnu/10/lto-wrapper".to_string(),
        "-plugin-opt=-fresolution=/tmp/ccnH96wF.res".to_string(),
        "-plugin-opt=-pass-through=-lgcc".to_string(),
        "-plugin-opt=-pass-through=-lgcc_eh".to_string(),
        "-plugin-opt=-pass-through=-lc".to_string(),
        "--sysroot=/".to_string(),
        "--build-id".to_string(),
        "-hash-style=gnu".to_string(),
        "--as-needed".to_string(),
        "-melf64lriscv".to_string(),
        "-static".to_string(),
        "-z".to_string(),
        "relro".to_string(),
        "-o".to_string(),
        "out/tests/hello/out".to_string(),
        "/usr/lib/gcc-cross/riscv64-linux-gnu/10/../../../../riscv64-linux-gnu/lib/crt1.o"
            .to_string(),
        "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crti.o".to_string(),
        "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crtbeginT.o".to_string(),
        "-L.".to_string(),
        "-L/usr/lib/gcc-cross/riscv64-linux-gnu/10".to_string(),
        "-L/usr/lib/gcc-cross/riscv64-linux-gnu/10/../../../../riscv64-linux-gnu/lib".to_string(),
        "-L/lib/riscv64-linux-gnu".to_string(),
        "-L/usr/lib/riscv64-linux-gnu".to_string(),
        "out/tests/hello/a.o".to_string(),
        "--start-group".to_string(),
        "-lgcc".to_string(),
        "-lgcc_eh".to_string(),
        "-lc".to_string(),
        "--end-group".to_string(),
        "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crtend.o".to_string(),
        "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crtn.o".to_string(),
    ];

    let s_args: &[String] = Box::leak(Box::new(f_args));
    let args: UnsafeCell<&[String]> = UnsafeCell::new(s_args);

    let _args = args.get();
    unsafe { *_args = &(*args.get())[1..] }

    let dashes = |name: &str| -> Vec<String> {
        if name.len() == 1 {
            return vec!["-".to_string() + name];
        }
        return vec!["-".to_string() + &name, "--".to_string() + &name];
    };

    let arg = UnsafeCell::new("");
    let _arg = arg.get();

    let read_arg = |name: &str| -> bool {
        for opt in dashes(name) {
            if unsafe { (*_args).get(0) }.unwrap() == &opt {
                if unsafe { (*_args).len() } == 1 {
                    fatal(&format!("option -{}: argument missing", name));
                }
                unsafe { *_arg = { (*_args).get(1) }.unwrap() };
                unsafe { *_args = &(*_args)[2..] }
                return true;
            }
            let mut prefix = String::from(&opt);
            if name.len() > 1 {
                prefix += "=";
            }
            if unsafe { (*_args).get(0) }.unwrap().starts_with(&prefix) {
                unsafe { *_arg = &{ (*_args).get(0) }.unwrap()[prefix.len()..] };
                unsafe { *_args = &(*_args)[1..] }
                return true;
            }
        }
        return false;
    };

    let read_flag = |name: &str| -> bool {
        for opt in dashes(name) {
            if unsafe { (*_args).get(0) }.unwrap() == &opt {
                unsafe { *_args = &(*_args)[1..] }
                return true;
            }
        }
        return false;
    };

    let mut remaining: Vec<String> = vec![];
    loop {
        if unsafe { (*args.get()).len() } < 1 {
            break;
        }
        if read_flag("help") {
            format!("usage: {} [options] file...\n", s_args[0]);
            exit(0);
        }
        if read_arg("o") || read_arg("output") {
            ctx.args.output = String::from(unsafe { *arg.get() });
        } else if read_flag("v") || read_flag("version") {
            format!("rvld {}\n", "");
            exit(0);
        } else if read_arg("m") {
            if unsafe { *_arg } == "elf64lriscv" {
                ctx.args.emulation = MACHINE_TYPE_RISCV64;
            } else {
                fatal(&format!("unknown -m argument: {}", unsafe { *arg.get() }));
            }
        } else if read_arg("L") {
            ctx.args
                .library_paths
                .push("".to_string() + &unsafe { *arg.get() });
        } else if read_arg("l") {
            remaining.push("-l".to_string() + &unsafe { *arg.get() });
        } else if read_arg("sysroot")
            || read_flag("static")
            || read_arg("plugin")
            || read_arg("plugin-opt")
            || read_flag("as-needed")
            || read_flag("start-group")
            || read_flag("end-group")
            || read_arg("hash-style")
            || read_arg("build-id")
            || read_flag("s")
            || read_flag("no-relax")
            || read_flag("z")
        {
            // ignore
        } else {
            if unsafe { (*_args).get(0) }.unwrap().starts_with("-") {
                fatal(&format!("unknown command line option: {}", unsafe {
                    *_arg
                }));
            }
            let _args = args.get();
            remaining.push(String::from(unsafe { *_args }.get(0).unwrap()));
            unsafe { *_args = &(*_args)[1..] }
        }
    }
    return remaining;
}
