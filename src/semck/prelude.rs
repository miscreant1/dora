use parser::ast::ctxt::*;

use sym::Sym::*;
use sym::BuiltinType;

pub fn init(ctxt: &Context) {
    add_builtin_types(ctxt);
    add_builtin_functions(ctxt);
}

fn add_builtin_types(ctxt: &Context) {
    builtin_type("int", BuiltinType::Int, ctxt);
    builtin_type("bool", BuiltinType::Bool, ctxt);
    builtin_type("str", BuiltinType::Str, ctxt);
}

fn builtin_type(name: &str, ty: BuiltinType, ctxt: &Context) {
    let name = ctxt.interner.intern(name.into());
    assert!(ctxt.sym.borrow_mut().insert(name, SymType(ty)).is_none());
}

fn add_builtin_functions(ctxt: &Context) {
    // TODO: add println(str) and assert(bool)
    let name = ctxt.interner.intern("assert");

    let fct_info = FctInfo {
        name: name,
        params_types: vec![BuiltinType::Bool],
        return_type: BuiltinType::Unit,
        ast: None,
    };

    assert!(ctxt.add_function(fct_info).is_ok());
}
