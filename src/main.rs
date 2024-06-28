use std::process::exit;

mod archive;
mod chunk;
mod context;
mod elf;
mod file;
mod file_type;
mod got_section;
mod input;
mod input_file;
mod input_section;
mod machine_type;
mod magic;
mod mergeablesection;
mod merged_section;
mod object_file;
mod output;
mod output_ehdr;
mod output_phdr;
mod output_section;
mod output_shdr;
mod passes;
mod section_fragment;
mod symbol;
mod utils;

use crate::{
    input::read_input_files,
    machine_type::{get_machine_type_from_contents, MACHINE_TYPE_NONE},
    passes::{register_section_pieces, resolve_symbols},
};
use context::Context;
use file::must_new_file;
use machine_type::MACHINE_TYPE_RISCV64;
use utils::fatal;

#[allow(dead_code)]
struct Args {
    raw_args: &'static [&'static str],
    args: &'static [&'static str],
    idx: usize,
    arg: &'static str,
}

#[allow(dead_code)]
impl Args {
    fn new() -> Self {
        let args: Vec<&str> = vec!["./ld", "-plugin", "/usr/lib/gcc-cross/riscv64-linux-gnu/10/liblto_plugin.so", "-plugin-opt=/usr/lib/gcc-cross/riscv64-linux-gnu/10/lto-wrapper", "-plugin-opt=-fresolution=/tmp/ccfKlx09.res", "-plugin-opt=-pass-through=-lgcc", "-plugin-opt=-pass-through=-lgcc_eh", "-plugin-opt=-pass-through=-lc", "--sysroot=/", "--build-id", "-hash-style=gnu", "-as-needed", "-melf64lriscv", "-static", "-o", "out/tests/hello/out", "/usr/lib/gcc-cross/riscv64-linux-gnu/10/../../../../riscv64-linux-gnu/lib/crt1.o", "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crti.o", "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crtbeginT.o", "-L.", "-L/usr/lib/gcc-cross/riscv64-linux-gnu/10", "-L/usr/lib/gcc-cross/riscv64-linux-gnu/10/../../../../riscv64-linux-gnu/lib", "-L/usr/lib/riscv64-linux-gnu", "out/tests/hello/a.o", "--start-group", "-lgcc", "-lgcc_eh", "-lc", "--end-group", "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crtend.o", "/usr/lib/gcc-cross/riscv64-linux-gnu/10/crtn.o"];
        let args = Box::leak(Box::new(args));

        Self {
            raw_args: args,
            args: &args[1..],
            idx: 0,
            arg: "",
        }
    }
}

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

    read_input_files(&mut ctx, &remaining);
    resolve_symbols(&mut ctx);
    register_section_pieces(&mut ctx);

    for obj in ctx.objs {
        if unsafe { obj.as_ref().unwrap().input_file.as_ref().unwrap().file.name }
            == "out/tests/hello/a.o"
        {
            for sym in unsafe { &obj.as_ref().unwrap().input_file.as_ref().unwrap().symbols } {
                if unsafe { sym.as_ref().unwrap().name } == "puts" {
                    println!("{:?}", unsafe {
                        sym.as_ref()
                            .unwrap()
                            .object_file
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .input_file
                            .as_ref()
                            .unwrap()
                            .file
                            .files
                            .get(0)
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .name
                    });
                }
            }
        }
    }
}

#[allow(dead_code)]
fn parse_args(ctx: &mut Context) -> Vec<String> {
    let mut args = Args::new();

    let dashes = |name: &str| -> Vec<String> {
        if name.len() == 1 {
            return vec!["-".to_string() + name];
        }

        return vec!["-".to_string() + &name, "--".to_string() + &name];
    };

    let read_arg = |name: &str, args: &mut Args| -> bool {
        for opt in dashes(name) {
            if args.args[0] == &opt {
                if args.args.len() == 1 {
                    fatal(&format!("option -{}: argument missing", name));
                }
                args.arg = args.args[1];
                args.args = &args.args[2..];
                return true;
            }
            let mut prefix = String::from(&opt);
            if name.len() > 1 {
                prefix += "=";
            }
            if args.args[0].starts_with(&prefix) {
                args.arg = &args.args[0][prefix.len()..];
                args.args = &args.args[1..];
                return true;
            }
        }
        return false;
    };

    let read_flag = |name: &str, args: &mut Args| -> bool {
        for opt in dashes(name) {
            if args.args[0] == &opt {
                args.args = &args.args[1..];
                return true;
            }
        }
        return false;
    };

    let mut remaining: Vec<String> = vec![];
    while args.args.len() > 0 {
        if read_flag("help", &mut args) {
            format!("usage: {} [options] file...\n", args.raw_args[0]);
            exit(0);
        }
        if read_arg("o", &mut args) || read_arg("output", &mut args) {
            ctx.args.output = String::from(args.arg);
        } else if read_flag("v", &mut args) || read_flag("version", &mut args) {
            format!("rvld {}\n", "");
            exit(0);
        } else if read_arg("m", &mut args) {
            if args.arg == "elf64lriscv" {
                ctx.args.emulation = MACHINE_TYPE_RISCV64;
            } else {
                fatal(&format!("unknown -m argument: {}", args.arg));
            }
        } else if read_arg("L", &mut args) {
            ctx.args.library_paths.push("".to_string() + args.arg);
        } else if read_arg("l", &mut args) {
            remaining.push("-l".to_string() + args.arg);
        } else if read_arg("sysroot", &mut args)
            || read_flag("static", &mut args)
            || read_arg("plugin", &mut args)
            || read_arg("plugin-opt", &mut args)
            || read_flag("as-needed", &mut args)
            || read_flag("start-group", &mut args)
            || read_flag("end-group", &mut args)
            || read_arg("hash-style", &mut args)
            || read_arg("build-id", &mut args)
            || read_flag("s", &mut args)
            || read_flag("no-relax", &mut args)
        {
            // ignore
        } else {
            if args.args[0].starts_with("-") {
                fatal(&format!("unknown command line option: {}", args.arg));
            }
            remaining.push(String::from(args.args[0]));
            args.args = &args.args[1..];
        }
    }
    return remaining;
}
