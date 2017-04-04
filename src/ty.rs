use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use class::{ClassId, TypeParamId};
use ctxt::{Context, StructId, TraitId};
use mem;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BuiltinType {
    // type with only one value: ()
    Unit,

    // value types
    Char,
    Byte,
    Int,
    Long,

    Float,
    Double,

    Bool,

    // type Nil, only used in typeck until final type is known
    Nil,

    // pointer to object, only used internally
    Ptr,

    // self type
    This,

    // String type
    Str,

    // some class
    Class(ClassId),

    // some struct
    Struct(StructId),

    // some trait
    Trait(TraitId),

    // some type variable
    TypeParam(TypeParamId),

    // generic types can have multiple params
    // use TypeId to store params
    Generic(TypeId),
}

impl BuiltinType {
    pub fn is_unit(&self) -> bool {
        match *self {
            BuiltinType::Unit => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match *self {
            BuiltinType::Nil => true,
            _ => false,
        }
    }

    pub fn is_cls(&self) -> bool {
        match *self {
            BuiltinType::Class(_) => true,
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        match *self {
            BuiltinType::Str => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            &BuiltinType::Float |
            &BuiltinType::Double => true,
            _ => false,
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            &BuiltinType::Generic(_) => true,
            _ => false,
        }
    }

    pub fn is_type_param(&self) -> bool {
        match self {
            &BuiltinType::TypeParam(_) => true,
            _ => false,
        }
    }

    pub fn type_id(&self) -> TypeId {
        match self {
            &BuiltinType::Generic(id) => id,
            _ => panic!(),
        }
    }

    pub fn cls_id(&self, ctxt: &Context) -> ClassId {
        match *self {
            BuiltinType::Class(cls_id) => cls_id,
            BuiltinType::Str => ctxt.primitive_classes.str_class,

            _ => panic!(),
        }
    }

    pub fn to_specialized(&self, ctxt: &Context) -> BuiltinType {
        match self {
            &BuiltinType::Generic(id) => {
                let cls_id = ctxt.types.borrow().get_cls_id(id);
                BuiltinType::Class(cls_id.expect("no specialized class exists"))
            }

            _ => *self,
        }
    }

    pub fn reference_type(&self) -> bool {
        !self.value_type()
    }

    pub fn value_type(&self) -> bool {
        match *self {
            BuiltinType::Unit | BuiltinType::Bool | BuiltinType::Byte | BuiltinType::Int |
            BuiltinType::Long | BuiltinType::Float | BuiltinType::Double => true,
            _ => false,
        }
    }

    pub fn subclass_from(&self, ctxt: &Context, ty: BuiltinType) -> bool {
        if !self.reference_type() {
            return false;
        }
        if !ty.reference_type() {
            return false;
        }

        let cls_id = self.cls_id(ctxt);
        let cls = ctxt.classes[cls_id].borrow();
        cls.subclass_from(ctxt, ty.cls_id(ctxt))
    }

    pub fn name(&self, ctxt: &Context) -> String {
        match *self {
            BuiltinType::Unit => "()".into(),
            BuiltinType::Byte => "byte".into(),
            BuiltinType::Char => "char".into(),
            BuiltinType::Int => "int".into(),
            BuiltinType::Long => "long".into(),
            BuiltinType::Float => "float".into(),
            BuiltinType::Double => "float".into(),
            BuiltinType::Bool => "bool".into(),
            BuiltinType::Nil => "nil".into(),
            BuiltinType::Ptr => panic!("type Ptr only for internal use."),
            BuiltinType::This => "Self".into(),
            BuiltinType::Str => "Str".into(),
            BuiltinType::Class(cid) => {
                let cls = ctxt.classes[cid].borrow();
                ctxt.interner.str(cls.name).to_string()
            }
            BuiltinType::Struct(sid) => {
                let name = ctxt.structs[sid].borrow().name;
                ctxt.interner.str(name).to_string()
            }
            BuiltinType::Trait(tid) => {
                let name = ctxt.traits[tid].borrow().name;
                ctxt.interner.str(name).to_string()
            }
            BuiltinType::TypeParam(id) => format!("#{}", id.idx()),
            BuiltinType::Generic(id) => {
                let generic = ctxt.types.borrow().get(id);
                let base = generic.base.name(ctxt);
                let params = generic
                    .params
                    .iter()
                    .map(|ty| ty.name(ctxt))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{}<{}>", base, params)
            }
        }
    }

    pub fn allows(&self, ctxt: &Context, other: BuiltinType) -> bool {
        match *self {
            BuiltinType::Unit |
            BuiltinType::Bool |
            BuiltinType::Byte |
            BuiltinType::Char |
            BuiltinType::Struct(_) => *self == other,
            BuiltinType::Int => *self == other,
            BuiltinType::Long => *self == other,
            BuiltinType::Float | BuiltinType::Double => *self == other,
            BuiltinType::Nil => panic!("nil does not allow any other types"),
            BuiltinType::Ptr => panic!("ptr does not allow any other types"),
            BuiltinType::This => unreachable!(),
            BuiltinType::Str |
            BuiltinType::Class(_) => {
                *self == other || other.is_nil() || other.subclass_from(ctxt, *self) ||
                (other.is_generic() && self.allows(ctxt, other.to_specialized(ctxt)))
            }
            BuiltinType::Trait(_) => unimplemented!(),
            BuiltinType::TypeParam(_) => *self == other,
            BuiltinType::Generic(_) => {
                *self == other || self.to_specialized(ctxt).allows(ctxt, other)
            }
        }
    }

    pub fn if_nil(&self, other: BuiltinType) -> BuiltinType {
        if self.is_nil() { other } else { *self }
    }

    pub fn size(&self, ctxt: &Context) -> i32 {
        match *self {
            BuiltinType::Unit => 0,
            BuiltinType::Bool => 1,
            BuiltinType::Byte => 1,
            BuiltinType::Char => 4,
            BuiltinType::Int => 4,
            BuiltinType::Long => 8,
            BuiltinType::Float => 4,
            BuiltinType::Double => 8,
            BuiltinType::Nil => panic!("no size for nil."),
            BuiltinType::This => panic!("no size for Self."),
            BuiltinType::Str |
            BuiltinType::Class(_) |
            BuiltinType::Ptr => mem::ptr_width(),
            BuiltinType::Struct(id) => ctxt.structs[id].borrow().size,
            BuiltinType::Trait(_) => 2 * mem::ptr_width(),
            BuiltinType::TypeParam(_) => panic!("no size for type variable."),
            BuiltinType::Generic(_) => self.to_specialized(ctxt).size(ctxt),
        }
    }

    pub fn align(&self, ctxt: &Context) -> i32 {
        match *self {
            BuiltinType::Unit => 0,
            BuiltinType::Bool => 1,
            BuiltinType::Byte => 1,
            BuiltinType::Char => 4,
            BuiltinType::Int => 4,
            BuiltinType::Long => 8,
            BuiltinType::Float => 4,
            BuiltinType::Double => 8,
            BuiltinType::Nil => panic!("no alignment for nil."),
            BuiltinType::This => panic!("no alignment for Self."),
            BuiltinType::Str |
            BuiltinType::Class(_) |
            BuiltinType::Ptr => mem::ptr_width(),
            BuiltinType::Struct(id) => ctxt.structs[id].borrow().align,
            BuiltinType::Trait(_) => mem::ptr_width(),
            BuiltinType::TypeParam(_) => panic!("no alignment for type variable."),
            BuiltinType::Generic(_) => self.to_specialized(ctxt).align(ctxt),
        }
    }

    pub fn mode(&self) -> MachineMode {
        match *self {
            BuiltinType::Unit => panic!("no machine mode for ()."),
            BuiltinType::Bool => MachineMode::Int8,
            BuiltinType::Byte => MachineMode::Int8,
            BuiltinType::Char => MachineMode::Int32,
            BuiltinType::Int => MachineMode::Int32,
            BuiltinType::Long => MachineMode::Int64,
            BuiltinType::Float => MachineMode::Float32,
            BuiltinType::Double => MachineMode::Float64,
            BuiltinType::Nil => panic!("no machine mode for nil."),
            BuiltinType::This => panic!("no machine mode for Self."),
            BuiltinType::Str |
            BuiltinType::Class(_) |
            BuiltinType::Ptr => MachineMode::Ptr,
            BuiltinType::Struct(_) => unimplemented!(),
            BuiltinType::Trait(_) => unimplemented!(),
            BuiltinType::TypeParam(_) => panic!("no machine mode for type variable."),
            BuiltinType::Generic(_) => MachineMode::Ptr,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MachineMode {
    Int8,
    Int32,
    Int64,
    Float32,
    Float64,
    Ptr,
}

impl MachineMode {
    pub fn size(self) -> i32 {
        match self {
            MachineMode::Int8 => 1,
            MachineMode::Int32 => 4,
            MachineMode::Int64 => 8,
            MachineMode::Ptr => mem::ptr_width(),
            MachineMode::Float32 => 4,
            MachineMode::Float64 => 8,
        }
    }

    pub fn is_float(self) -> bool {
        match self {
            MachineMode::Float32 |
            MachineMode::Float64 => true,
            _ => false,
        }
    }
}

pub struct Types {
    types: HashMap<Rc<TypeWithParams>, TypeId>,
    values: Vec<Rc<TypeWithParams>>,
    classes: RefCell<Vec<Option<ClassId>>>,
    next_type_id: usize,
}

impl Types {
    pub fn new() -> Types {
        Types {
            types: HashMap::new(),
            values: Vec::new(),
            classes: RefCell::new(Vec::new()),
            next_type_id: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn insert(&mut self, ty: BuiltinType, params: Vec<BuiltinType>) -> TypeId {
        assert!(ty.is_cls());

        let ty = TypeWithParams {
            base: ty,
            params: params,
        };

        if let Some(&val) = self.types.get(&ty) {
            return val;
        }

        let type_id = TypeId(self.next_type_id);
        let ty = Rc::new(ty);
        self.types.insert(ty.clone(), type_id);

        self.values.push(ty);
        self.classes.borrow_mut().push(None);

        self.next_type_id += 1;

        type_id
    }

    pub fn set_cls_id(&self, id: TypeId, cls_id: ClassId) {
        self.classes.borrow_mut()[id.0] = Some(cls_id);
    }

    pub fn get_cls_id(&self, id: TypeId) -> Option<ClassId> {
        self.classes.borrow()[id.0]
    }

    pub fn get(&self, id: TypeId) -> Rc<TypeWithParams> {
        self.values[id.0].clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeWithParams {
    pub base: BuiltinType,
    pub params: Vec<BuiltinType>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeId(usize);

impl From<usize> for TypeId {
    fn from(val: usize) -> TypeId {
        TypeId(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mem;

    #[test]
    fn mode_size() {
        assert_eq!(1, MachineMode::Int8.size());
        assert_eq!(4, MachineMode::Int32.size());
        assert_eq!(mem::ptr_width(), MachineMode::Ptr.size());
    }

    #[test]
    fn mode_for_types() {
        assert_eq!(MachineMode::Int8, BuiltinType::Bool.mode());
        assert_eq!(MachineMode::Int32, BuiltinType::Int.mode());
        assert_eq!(MachineMode::Ptr, BuiltinType::Ptr.mode());
        assert_eq!(MachineMode::Ptr, BuiltinType::Str.mode());
    }

    #[test]
    #[should_panic]
    fn mode_for_nil() {
        assert_eq!(MachineMode::Ptr, BuiltinType::Nil.mode());
    }

    #[test]
    #[should_panic]
    fn mode_for_unit() {
        assert_eq!(MachineMode::Ptr, BuiltinType::Unit.mode());
    }
}
