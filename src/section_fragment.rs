use crate::merged_section::MergedSection;

#[allow(dead_code)]
pub struct SectionFragment {
    pub output: *mut MergedSection,
    pub offset: u32,
    pub p2_align: u32,
    pub is_alive: bool,
}

#[allow(dead_code)]
impl SectionFragment {
    pub fn new(m: *mut MergedSection) -> Self {
        SectionFragment {
            output: m,
            offset: std::u32::MAX,
            p2_align: 0,
            is_alive: false,
        }
    }

    pub fn get_addr(&self) -> u64{
        unsafe { self.output.as_ref().unwrap().chunk.shdr.addr + self.offset as u64 } 
    }
}
