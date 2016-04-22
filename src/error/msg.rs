use self::Msg::*;
use class::ClassId;
use ctxt::Context;
use interner::Name;
use lexer::position::Position;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Msg {
    Unimplemented,
    UnknownType(Name),
    UnknownIdentifier(Name),
    UnknownFunction(String),
    UnknownProp(String, String),
    UnknownMethod(String, Name, Vec<String>),
    UnknownCtor(Name, Vec<String>),
    MethodExists(String, Name, Vec<String>, Position),
    VarNotMutable(Name),
    IncompatibleWithNil(String),
    IdentifierExists(String),
    ShadowFunction(String),
    ShadowParam(String),
    ShadowClass(String),
    ShadowProp(String),
    VarNeedsTypeInfo(String),
    ParamTypesIncompatible(Name, Vec<String>, Vec<String>),
    WhileCondType(String),
    IfCondType(String),
    ReturnType(String, String),
    LvalueExpected,
    AssignType(Name, String, String),
    AssignProp(Name, ClassId, String, String),
    UnOpType(String, String),
    BinOpType(String, String, String),
    OutsideLoop,
    NoReturnValue,
    MainNotFound,
    WrongMainDefinition,
    SelfUnavailable,
    MultipleCandidates(String, Name, Vec<String>),
    SelfNeeded,
    InvalidUseOfSelf
}

impl Msg {
    pub fn message(&self, ctxt: &Context) -> String {
        match *self {
            Unimplemented => format!("feature not implemented yet."),
            UnknownType(name) => {
                let name = ctxt.interner.str(name).to_string();
                format!("type `{}` does not exist.", name)
            },
            UnknownIdentifier(name) => {
                let name = ctxt.interner.str(name).to_string();
                format!("unknown identifier `{}`.", name)
            },
            UnknownFunction(ref name) => format!("unknown function `{}`", name),
            UnknownMethod(ref cls, name, ref args) => {
                let name = ctxt.interner.str(name).to_string();
                let args = args.connect(", ");

                format!("no method with definition `{}({})` in class `{}`.", name, args, cls)
            },
            UnknownCtor(name, ref args) => {
                let name = ctxt.interner.str(name).to_string();
                let args = args.connect(", ");

                format!("no ctor with definition `{}({})`.", name, args)
            }
            MethodExists(ref cls, name, ref args, pos) => {
                let name = ctxt.interner.str(name).to_string();
                let args = args.connect(", ");

                format!(
                    "method with definition `{}({})` already exists in class `{}` at line {}.",
                    name, args, cls, pos)
            },
            VarNotMutable(name) => {
                let name = ctxt.interner.str(name).to_string();

                format!("var `{}` not mutable.", name)
            },
            IncompatibleWithNil(ref ty) => format!("cannot assign `nil` to type `{}`.", ty),
            UnknownProp(ref prop, ref ty) =>
                format!("unknown property `{}` for type `{}`", prop, ty),
            IdentifierExists(ref name) => format!("can not redefine identifier `{}`.", name),
            ShadowFunction(ref name) => format!("can not shadow function `{}`.", name),
            ShadowParam(ref name) => format!("can not shadow param `{}`.", name),
            ShadowClass(ref name) => format!("can not shadow class `{}`.", name),
            ShadowProp(ref name) => format!("property with name `{}` already exists.", name),
            VarNeedsTypeInfo(ref name) =>
                format!("variable `{}` needs either type declaration or expression.", name),
            ParamTypesIncompatible(name, ref def, ref expr) => {
                let name = ctxt.interner.str(name).to_string();
                let def = def.connect(", ");
                let expr = expr.connect(", ");

                format!("function `{}({})` cannot be called as `{}({})`",
                    name, def, name, expr)
            },
            WhileCondType(ref ty) =>
                format!("`while` expects condition of type `bool` but got `{}`.", ty),
            IfCondType(ref ty) =>
                format!("`if` expects condition of type `bool` but got `{}`.", ty),
            ReturnType(ref def, ref expr) =>
                format!("`return` expects value of type `{}` but got `{}`.",
                    def, expr),
            LvalueExpected => format!("lvalue expected for assignment"),
            AssignType(name, ref def, ref expr) => {
                let name = ctxt.interner.str(name).to_string();
                format!("cannot assign `{}` to variable `{}` of type `{}`.", expr, name, def)
            },
            AssignProp(name, clsid, ref def, ref expr) => {
                let cls = ctxt.cls_by_id(clsid);
                let cls_name = ctxt.interner.str(cls.name).to_string();
                let name = ctxt.interner.str(name).to_string();

                format!("cannot assign `{}` to property `{}`.`{}` of type `{}`.",
                        expr, cls_name, name, def)
            },
            UnOpType(ref op, ref expr) =>
                format!("unary operator `{}` can not handle value of type `{} {}`.", op, op,
                    expr),
            BinOpType(ref op, ref lhs, ref rhs) =>
                format!("binary operator `{}` can not handle expression of type `{} {} {}`",
                    op, lhs, op, rhs),
            OutsideLoop => "statement only allowed inside loops".into(),
            NoReturnValue => "function does not return a value in all code paths".into(),
            MainNotFound => "no `main` function found in the program".into(),
            WrongMainDefinition => "`main` function has wrong definition".into(),
            SelfUnavailable => "`self` can only be used in methods not functions".into(),
            MultipleCandidates(ref cls, name, ref call_types) => {
                let name = ctxt.interner.str(name).to_string();
                let call_types = call_types.connect(", ");

                format!("multiple candidates for invocation `{}({})` in class `{}`.",
                    name, call_types, cls)
            },
            SelfNeeded => "`self` parameter needed for methods.".into(),
            InvalidUseOfSelf => "`self` only allowed for first argument of methods.".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MsgWithPos {
    pub msg: Msg,
    pub pos: Position,
}

impl MsgWithPos {
    pub fn new(pos: Position, msg: Msg) -> MsgWithPos {
        MsgWithPos {
            pos: pos,
            msg: msg
        }
    }

    pub fn message(&self, ctxt: &Context) -> String {
        format!("error at {}: {}", self.pos, self.msg.message(ctxt))
    }
}
