use std::sync::{Arc, Mutex};

use ast;
use ast::visit::{self, Visitor};
use ctxt::{Context, Fct, FctId, FctKind, FctParent, FctSrc, ImplId, NodeMap};
use error::msg::Msg;
use lexer::position::Position;
use sym::Sym;
use ty::BuiltinType;

pub fn check<'ast>(ctxt: &mut Context<'ast>, map_impl_defs: &mut NodeMap<ImplId>) {
    let mut clsck = ImplCheck {
        ctxt: ctxt,
        ast: ctxt.ast,
        impl_id: None,
        map_impl_defs: map_impl_defs,
    };

    clsck.check();
}

struct ImplCheck<'x, 'ast: 'x> {
    ctxt: &'x mut Context<'ast>,
    ast: &'ast ast::Ast,
    map_impl_defs: &'x mut NodeMap<ImplId>,

    impl_id: Option<ImplId>,
}

impl<'x, 'ast> ImplCheck<'x, 'ast> {
    fn check(&mut self) {
        self.visit_ast(self.ast);
    }
}

impl<'x, 'ast> Visitor<'ast> for ImplCheck<'x, 'ast> {
    fn visit_impl(&mut self, i: &'ast ast::Impl) {
        self.impl_id = Some(*self.map_impl_defs.get(i.id).unwrap());

        visit::walk_impl(self, i);

        let mut ximpl = self.ctxt.impls[self.impl_id.unwrap()].borrow_mut();

        if let Some(Sym::SymTrait(trait_id)) = self.ctxt.sym.borrow().get(i.trait_name) {
            ximpl.trait_id = Some(trait_id);
        } else {
            let name = self.ctxt.interner.str(i.trait_name).to_string();
            report(self.ctxt, i.pos, Msg::ExpectedTrait(name));
        }

        if let Some(Sym::SymClass(class_id)) = self.ctxt.sym.borrow().get(i.class_name) {
            ximpl.class_id = Some(class_id);
        } else {
            let name = self.ctxt.interner.str(i.class_name).to_string();
            report(self.ctxt, i.pos, Msg::ExpectedClass(name));
        }

        if ximpl.trait_id.is_some() && ximpl.class_id.is_some() {
            let mut cls = self.ctxt.classes[ximpl.class_id()].borrow_mut();
            cls.traits.push(ximpl.trait_id());
            cls.impls.push(ximpl.id);
        }

        self.impl_id = None;
    }

    fn visit_method(&mut self, f: &'ast ast::Function) {
        if self.impl_id.is_none() {
            return;
        }

        if f.block.is_none() {
            report(self.ctxt, f.pos, Msg::MissingFctBody);
        }

        let kind = FctKind::Source(Arc::new(Mutex::new(FctSrc::new())));

        let fct = Fct {
            id: FctId(0),
            ast: f,
            pos: f.pos,
            name: f.name,
            param_types: Vec::new(),
            return_type: BuiltinType::Unit,
            parent: FctParent::Impl(self.impl_id.unwrap()),
            has_override: f.has_override,
            has_open: f.has_open,
            has_final: f.has_final,
            is_pub: f.is_pub,
            is_static: f.is_static,
            internal: f.internal,
            internal_resolved: false,
            overrides: None,
            throws: f.throws,
            ctor: ast::CtorType::None,
            vtable_index: None,
            initialized: false,
            kind: kind,
        };

        let fctid = self.ctxt.add_fct(fct);

        let mut ximpl = self.ctxt.impls[self.impl_id.unwrap()].borrow_mut();
        ximpl.methods.push(fctid);
    }
}

fn report(ctxt: &Context, pos: Position, msg: Msg) {
    ctxt.diag.borrow_mut().report(pos, msg);
}

#[cfg(test)]
mod tests {
    use error::msg::Msg;
    use semck::tests::*;

    #[test]
    fn impl_method_without_body() {
        err("
            trait Foo {
                fun foo() -> int;
            }
            class Bar {}
            impl Foo for Bar { fun foo() -> int;}",
            pos(6, 32),
            Msg::MissingFctBody);
    }

    #[test]
    fn impl_method_defined_twice() {
        err("
            trait Foo {
                fun foo() -> int;
            }
            class Bar {}
            impl Foo for Bar {
                fun foo() -> int { return 0; }
                fun foo() -> int { return 1; }
            }",
            pos(8, 17),
            Msg::MethodExists("Foo".into(), "foo".into(), vec![], pos(7, 17)));
    }

    #[test]
    fn impl_for_unknown_trait() {
        err("class A {} impl Foo for A {}",
            pos(1, 12),
            Msg::ExpectedTrait("Foo".into()));
    }

    #[test]
    fn impl_for_unknown_class() {
        err("trait Foo {} impl Foo for A {}",
            pos(1, 14),
            Msg::ExpectedClass("A".into()));
    }

    #[test]
    fn impl_definitions() {
        ok("trait Foo {} class A {} impl Foo for A {}");
        ok("trait Foo { fun toBool() -> bool; }
            class A {}
            impl Foo for A { fun toBool() -> bool { return false; } }");
    }
}