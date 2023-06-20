// package linker

// import "github.com/ksco/rvld/pkg/utils"

// type Symbol struct {
// 	File         *ObjectFile
// 	InputSection *InputSection
// 	Name         string
// 	Value        uint64
// 	SymIdx       int
// }

// func NewSymbol(name string) *Symbol {
// 	s := &Symbol{Name: name}
// 	return s
// }

// func (s *Symbol) SetInputSection(isec *InputSection) {
// 	s.InputSection = isec
// }

// func GetSymbolByName(ctx *Context, name string) *Symbol {
// 	if sym, ok := ctx.SymbolMap[name]; ok {
// 		return sym
// 	}
// 	ctx.SymbolMap[name] = NewSymbol(name)
// 	return ctx.SymbolMap[name]
// }

// func (s *Symbol) ElfSym() *Sym {
// 	utils.Assert(s.SymIdx < len(s.File.ElfSyms))
// 	return &s.File.ElfSyms[s.SymIdx]
// }

// func (s *Symbol) Clear() {
// 	s.File = nil
// 	s.InputSection = nil
// 	s.SymIdx = -1
// }

use crate::elf::Sym;
use crate::{object_file::ObjectFile, input_section::InputSection};
use crate::context::Context;
pub struct Symbol<'a> {
    object_file: ObjectFile<'a>,
    input_section: InputSection<'a>,
    name: &'static str,
    value: u64,
    symidx: i32,
}

impl<'a> Symbol<'a>{
    #[allow(dead_code)]
    fn new(object_file: ObjectFile<'a>, input_section: InputSection<'a>, name: &'static str, value: u64, symidx: i32) -> Self{
        Symbol { object_file: object_file, input_section: input_section, name: name, value: value, symidx: symidx }
    }
    
    #[allow(dead_code)]
    fn get_symbol_by_name(&self, ctx: Context<'a>, name:&str) -> *const Symbol{
       &ctx.symbol_map[name] as *const Symbol
    }

    #[allow(dead_code)]
    fn elf_sym(&self) -> Sym{
        assert!(self.symidx < self.object_file.input_file.elf_syms.len() as i32);
        self.object_file.input_file.elf_syms[self.symidx as usize]
    }
    
    #[allow(dead_code)]
    fn clear(&self) {
        // nothing
    }
}
