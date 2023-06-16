use crate::{
    file_type::{get_file_type, FileType, FILE_TYPE_OBJECT},
    utils::read,
};
pub type MachineType = u8;

pub const MACHINE_TYPE_NONE: MachineType = 0;
pub const MACHINE_TYPE_RISCV64: MachineType = 1;

#[allow(dead_code)]
pub fn get_machine_type_from_contents(contents: &[u8]) -> MachineType {
    let ft: FileType = get_file_type(contents);

    match ft {
        FILE_TYPE_OBJECT => {
            let machine: u16 = read(&contents[18..]);
            if machine == 243u16 {
                let class = &contents[4];
                match class {
                    &2u8 => {
                        return MACHINE_TYPE_RISCV64;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    return MACHINE_TYPE_NONE;
}

pub struct MachineTypeStringer {
    machine_type: MachineType,
}

impl MachineTypeStringer {
    #[allow(dead_code)]
    fn string(&self) -> &str {
        match self.machine_type {
            MACHINE_TYPE_RISCV64 => "risc64",
            _ => "None",
        }
    }
}
