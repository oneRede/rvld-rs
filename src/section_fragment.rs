use crate::merged_section::MergedSection;

#[allow(dead_code)]
pub struct SectionFragment<'a> {
    pub output: &'a MergedSection<'a>,
    pub offset: u32,
    pub p2_align: u32,
    pub is_alive: bool,
}

impl<'a> SectionFragment<'a> {
    #[allow(dead_code)]
    pub fn new(m: &'a MergedSection) -> Self {
        SectionFragment {
            output: m,
            offset: std::u32::MAX,
            p2_align: 0,
            is_alive: false,
        }
    }
}
