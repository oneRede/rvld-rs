use crate::machine_type::{MachineType, MACHINE_TYPE_NONE};

#[allow(dead_code)]
pub struct ContextArgs {
    pub output: String,
    pub emulation: MachineType,
    pub library_paths: Vec<String>,
}

#[allow(dead_code)]
pub struct Context {
    pub args: ContextArgs,
}

impl Context {
    #[allow(dead_code)]
    pub fn new() -> Self{
        Context {
            args: ContextArgs {
                output: "a.out".to_string(),
                emulation: MACHINE_TYPE_NONE,
                library_paths: vec![],
            },
        }
    }
}
