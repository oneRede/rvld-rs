use crate::machine_type::{MachineType, MACHINE_TYPE_NONE};

#[allow(dead_code)]
struct ContextArgs {
    output: &'static str,
    emulation: MachineType,
    library_paths: Option<Vec<&'static str>>,
}

#[allow(dead_code)]
struct Context {
    args: ContextArgs,
}

#[allow(dead_code)]
fn new_context() -> Context {
    Context {
        args: ContextArgs {
            output: "a.out",
            emulation: MACHINE_TYPE_NONE,
            library_paths: None,
        },
    }
}
