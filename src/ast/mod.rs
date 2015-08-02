use std::rc::Rc;
/*
 * Structs and enums for the various parts of the Cool Language.
 */
pub type Symbol = Rc<String>;
pub type CoolType = Rc<String>;

pub struct Program {
    pub classes: Vec<Box<Class>>
}

pub struct Class {
    pub name: Symbol,
    pub parent: Symbol,
    pub features: Vec<Box<Feature>>
}

pub enum Feature {
    Method {
        name: Symbol,
        params: Vec<Formal>,
        return_type: CoolType,
        body: Box<Expression>
    },
    Attribute {
        name: Symbol,
        cool_type: CoolType,
        expr: Box<Expression>
    }
}

pub struct Formal {
    pub name: Symbol,
    pub cool_type: CoolType
}

pub enum Expression {
    Assign(Symbol, Box<Expression>),
    Dispatch(Box<Expression>, Symbol, Vec<Box<Expression>>),
    StaticDispatch(Box<Expression>, Symbol, Symbol, Vec<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    While(Box<Expression>, Box<Expression>),
    Let(Symbol, CoolType, Box<Expression>, Box<Expression>),
    Case(Box<Expression>, Vec<Box<CaseBranch>>),
    Block(Vec<Expression>),
    New(CoolType),
    IsVoid(Box<Expression>),
    BinaryOperation(BinOp, Box<Expression>, Box<Expression>),
    Negation(Box<Expression>),
    Not(Box<Expression>),
    Identifier(Symbol),
    IntLiteral(i32),
    StringLiteral(Symbol),
    True,
    False,
    NoExpr
}

pub struct CaseBranch {
    pub name: Symbol,
    pub cool_type: CoolType,
    pub expr: Box<Expression>
}

pub enum BinOp {
    Plus,
    Minus,
    Mult,
    Divide,
    LessThan,
    LessThanEqual,
    Equal
}
