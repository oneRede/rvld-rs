use crate::{merged_section::MergedSection, section_fragment::SectionFragment};

#[allow(dead_code)]
struct MergeableSection {
    parent: MergedSection,
    p2_align: u8,
    strs: Vec<String>,
    frag_offsets: Vec<usize>,
    fragments: Vec<SectionFragment>,
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
