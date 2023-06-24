use crate::{merged_section::MergedSection, section_fragment::SectionFragment};

#[allow(dead_code)]
pub struct MergeableSection {
    pub parent: Option<*mut MergedSection>,
    pub p2_align: u8,
    pub strs: Vec<String>,
    pub frag_offsets: Vec<usize>,
    pub fragments: Vec<*mut SectionFragment>,
}

#[allow(dead_code)]
impl MergeableSection {
    pub fn new() -> Self {
        Self {
            parent: None,
            p2_align: 0,
            strs: vec![],
            frag_offsets: vec![],
            fragments: vec![],
        }
    }

    pub fn get_fragment(&self, offset: usize) -> (Option<*mut SectionFragment>, usize) {
        let pos = self
            .frag_offsets
            .binary_search_by(|fo| fo.cmp(&offset))
            .unwrap();

        if pos == 0 {
            return (None, 0);
        }
        let idx = pos - 1;
        return (Some(self.fragments[idx]), offset - self.frag_offsets[idx]);
    }
}
