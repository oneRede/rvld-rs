use crate::machine_type::{MachineType, MACHINE_TYPE_NONE};

#[allow(dead_code)]
pub struct ContextArgs {
    pub output: &'static str,
    pub emulation: MachineType,
    pub library_paths: Vec<String>,
}

#[allow(dead_code)]
pub struct Context {
    pub args: ContextArgs,
}

#[allow(dead_code)]
fn new_context() -> Context {
    Context {
        args: ContextArgs {
            output: "a.out",
            emulation: MACHINE_TYPE_NONE,
            library_paths: vec![],
        },
    }
}
