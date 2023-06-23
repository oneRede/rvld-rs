use crate::{merged_section::MergedSection, section_fragment::SectionFragment};

#[allow(dead_code)]
pub struct MergeableSection {
    pub parent: MergedSection,
    pub p2_align: u8,
    pub strs: Vec<String>,
    pub frag_offsets: Vec<usize>,
    pub fragments: Vec<SectionFragment>,
}

#[allow(dead_code)]
impl MergeableSection {
    fn get_fragment(&self, offset: usize) -> (Option<&SectionFragment>, usize){
        let pos = self.frag_offsets.binary_search_by(|fo| {fo.cmp(&offset)}).unwrap();

        if pos == 0 {
            return (None,0)        
        }
        let idx = pos -1;
        return (Some(&self.fragments[idx]), offset-self.frag_offsets[idx])
    }
}
