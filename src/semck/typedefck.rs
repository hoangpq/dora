use ast::*;
use ast::Stmt::*;
use ast::Type::*;
use ast::visit::*;
use ctxt::{Context, FctContext};
use error::msg::Msg;
use ty::BuiltinType;

pub fn check<'a, 'ast>(ctxt: &Context<'a, 'ast>) {
    for fct in ctxt.fcts.iter() {
        let mut fct = fct.lock().unwrap();

        if let Some(ast) = fct.ast {
            let mut defck = TypeDefCheck {
                ctxt: ctxt,
                fct: &mut fct,
                ast: ast,
                current_type: BuiltinType::Unit,
            };

            defck.check();
        }
    }
}

struct TypeDefCheck<'a, 'ast: 'a> {
    ctxt: &'a Context<'a, 'ast>,
    fct: &'a mut FctContext<'ast>,
    ast: &'ast Function,
    current_type: BuiltinType,
}

impl<'a, 'ast> TypeDefCheck<'a, 'ast> {
    fn check(&mut self) {
        self.visit_fct(self.ast);
    }
}

impl<'a, 'ast> Visitor<'ast> for TypeDefCheck<'a, 'ast> {
    fn visit_fct(&mut self, f: &'ast Function) {
        for p in &f.params {
            self.visit_param(p);
        }

        if let Some(ref ty) = f.return_type {
            self.visit_type(ty);
            self.fct.return_type = self.current_type;
        }

        self.visit_stmt(&f.block);
    }

    fn visit_param(&mut self, p: &'ast Param) {
        self.visit_type(&p.data_type);

        self.fct.params_types.push(self.current_type);

        let var = self.fct.var_by_node_id_mut(p.id);
        var.data_type = self.current_type;
    }

    fn visit_stmt(&mut self, s: &'ast Stmt) {
        if let StmtVar(ref var) = *s {
            if let Some(ref data_type) = var.data_type {
                self.visit_type(data_type);

                let var = self.fct.var_by_node_id_mut(var.id);
                var.data_type = self.current_type;
            }

            if let Some(ref expr) = var.expr {
                visit::walk_expr(self, expr);
            }

        } else {
            visit::walk_stmt(self, s);
        }
    }

    fn visit_type(&mut self, t: &'ast Type) {
        match *t {
            TypeBasic(ref basic) => {
                if let Some(builtin) = self.ctxt.sym.borrow().get_type(basic.name) {
                    self.current_type = builtin;
                } else {
                    let tyname = self.ctxt.interner.str(basic.name).to_string();
                    let msg = Msg::UnknownType(tyname);
                    self.ctxt.diag.borrow_mut().report(basic.pos, msg);
                }
            }

            _ => self.ctxt.diag.borrow_mut().report_unimplemented(t.pos())
        }
    }
}