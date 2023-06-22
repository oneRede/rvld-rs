#[allow(dead_code)]
const ELF_MERGE: u64 = 0;
#[allow(dead_code)]
const ELF_STRING: u64 = 0;
#[allow(dead_code)]
const PREFIXES: [&str; 13] = [
    ".text.",
    ".data.rel.ro.",
    ".data.",
    ".rodata.",
    ".bss.rel.ro.",
    ".bss.",
    ".init_array.",
    ".fini_array.",
    ".tbss.",
    ".tdata.",
    ".gcc_except_table.",
    ".ctors.",
    ".dtors.",
];

#[allow(dead_code)]
pub fn get_output_name(name: &str, flags: u64) -> &str {
    if name == ".rodata" || name.starts_with(".rodata") && flags & ELF_MERGE != 0 {
        if flags & ELF_STRING != 0 {
            return ".rodata";
        } else {
            return ".rodata.cst";
        }
    }

    for prefix in PREFIXES {
        let stem = &prefix[..(prefix.len() - 1)];
        if name == stem || name.starts_with(prefix) {
            return stem;
        }
    }

    return name;
}
