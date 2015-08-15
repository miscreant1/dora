use std::collections::HashMap;
use std::collections::hash_map::IterMut;
use std::collections::hash_map::Entry::{Vacant, Occupied};

use ast::BuiltinType;
use ast::NodeId;
use ast::Type;

use interner::Name;

#[derive(Debug)]
pub struct SymTable {
    map: HashMap<Name, Sym>
}

impl SymTable {
    // creates a new table
    pub fn new() -> SymTable {
        SymTable {
            map: HashMap::new()
        }
    }

    // finds symbol in table
    pub fn find(&self, name: Name) -> Option<&Sym> {
        self.map.get(&name)
    }

    // inserts symbol into the table
    pub fn insert(& mut self, name: Name, sym: Sym) -> Result<(), &Sym> {
        match self.map.entry(name) {
            Vacant(entry) => {
                entry.insert(sym);

                Ok(())
            }

            Occupied(old) => Err(old.into_mut())
        }
    }

    pub fn functions_mut(&mut self) -> SymFunctionIterMut {
        SymFunctionIterMut {
            iter: self.map.iter_mut()
        }
    }
}

#[derive(Debug)]
pub enum Sym {
    SymLocalVar(SymLocalVarType),
    SymFunction(SymFunctionType),

    // only for testing purposes
    SymDummy(u8),
}

#[derive(Debug)]
pub struct SymFunctionType {
    pub name: Name,
    pub return_type: BuiltinType,
    pub params: Vec<Param>,
    pub body: NodeId,
}

#[derive(Debug)]
pub struct Param {
    pub name: Name,
    pub data_type: BuiltinType
}

#[derive(Debug)]
pub struct SymLocalVarType {
    pub name: Name,
    pub data_type: Type,
    pub expr: Option<NodeId>,
}

struct SymFunctionIterMut<'a> {
    iter: IterMut<'a, Name, Sym>
}

impl<'a> Iterator for SymFunctionIterMut<'a> {
    type Item = &'a SymFunctionType;

    fn next(&mut self) -> Option<&'a SymFunctionType> {
        loop {
            let next = self.iter.next();

            match next {
                None => return None,

                Some(val) => {
                    if let Sym::SymFunction(ref fct) = *val.1 {
                        return Some(fct);
                    }
                }
            }
        }
    }
}

#[test]
fn test_insert_and_find_again() {
    let mut table = SymTable::new();

    assert!(table.insert(Name(1), Sym::SymDummy(1)).is_ok());
    assert!(table.find(Name(1)).is_some());
    assert!(table.find(Name(2)).is_none());
}

#[test]
fn test_insert_twice_into_same_level() {
    let mut table = SymTable::new();

    assert!(table.insert(Name(1), Sym::SymDummy(1)).is_ok());
    assert!(table.insert(Name(1), Sym::SymDummy(2)).is_err());
}

