use ast::BinOp;
use ast::CaseBranch;
use ast::Class;
use ast::Expression;
use ast::Feature;
use ast::Formal;
use ast::Program;
use lexer::Token;

pub trait Printable {
    fn pretty_print(&self, tabs: u32);
}

impl Printable for Program {
    fn pretty_print(&self, tabs: u32) {
        println(tabs, "_program");
        for class in &self.classes {
            class.pretty_print(tabs + 1);
        }
    }
}

impl Printable for Class {
    fn pretty_print(&self, tabs: u32) {
        println(tabs, "_class");
        println(tabs + 1, &self.name);
        println(tabs + 1, &self.parent);
        for feature in &self.features {
            feature.pretty_print(tabs + 1);
        }
    }
}

impl Printable for Feature {
    fn pretty_print(&self, tabs: u32) {
        match *self {
            Feature::Method { ref name, ref params, ref return_type, ref body } => {
                println(tabs, "_method");
                println(tabs + 1, name);
                for param in params {
                    println(tabs + 1, "_formal");
                    param.pretty_print(tabs + 2);
                }
                println(tabs + 1, return_type);
                body.pretty_print(tabs + 1);
            },
            Feature::Attribute { ref name, ref cool_type, ref expr } => {
                println(tabs, "_attr");
                println(tabs + 1, name);
                println(tabs + 1, cool_type);
                expr.pretty_print(tabs + 1);
            }
        }
    }
}

impl Printable for Formal {
    fn pretty_print(&self, tabs: u32) {
        println(tabs, &self.name);
        println(tabs, &self.cool_type);
    }
}

impl Printable for Expression {
    fn pretty_print(&self, tabs: u32) {
        match *self {
            Expression::Assign(ref var, ref expr) => {
                println(tabs, "_assign");
                println(tabs + 1, var);
                expr.pretty_print(tabs + 1);
            },
            Expression::Dispatch(ref obj, ref name, ref args) => {
                println(tabs, "_dispatch");
                obj.pretty_print(tabs + 1);
                println(tabs + 1, name);
                println(tabs + 1, "(");
                for arg in args {
                    arg.pretty_print(tabs + 1);
                }
                println(tabs + 1, ")");
            },
            Expression::StaticDispatch(ref obj, ref cool_type, ref name, ref args) => {
                println(tabs, "_dispatch");
                obj.pretty_print(tabs + 1);
                println(tabs + 1, name);
                println(tabs + 1, cool_type);
                println(tabs + 1, "(");
                for arg in args {
                    arg.pretty_print(tabs + 1);
                }
                println(tabs + 1, ")");
            },
            Expression::If(ref cond, ref true_branch, ref false_branch) => {
                println(tabs, "_cond");
                cond.pretty_print(tabs + 1);
                true_branch.pretty_print(tabs + 1);
                false_branch.pretty_print(tabs + 1);
            },
            Expression::While(ref cond, ref body) => {
                println(tabs, "_loop");
                cond.pretty_print(tabs + 1);
                body.pretty_print(tabs + 1);
            },
            Expression::Let(ref var, ref cool_type, ref init, ref body) => {
                println(tabs, "_let");
                println(tabs + 1, cool_type);
                init.pretty_print(tabs + 1);
                body.pretty_print(tabs + 1);
            },
            Expression::Case(ref expr, ref branches) => {
                println(tabs, "_typcase");
                expr.pretty_print(tabs + 1);
                for branch in branches {
                    branch.pretty_print(tabs + 1);
                }
            },
            Expression::Block(ref exprs) => {
                println(tabs, "_block");
                for expr in exprs {
                    expr.pretty_print(tabs + 1);
                }
            },
            Expression::New(ref cool_type) => {
                println(tabs, "_new");
                println(tabs + 1, cool_type);
            },
            Expression::IsVoid(ref expr) => {
                println(tabs, "_isvoid");
                expr.pretty_print(tabs + 1);
            },
            Expression::BinaryOperation(ref op, ref left, ref right) => {
                println(tabs, match *op {
                    BinOp::Plus => "_plus",
                    BinOp::Minus => "_sub",
                    BinOp::Mult => "_mul",
                    BinOp::Divide => "_divide",
                    BinOp::LessThan => "_lt",
                    BinOp::LessThanEqual => "_leq",
                    BinOp::Equal => "_eq"
                });
                left.pretty_print(tabs + 1);
                right.pretty_print(tabs + 1);
            },
            Expression::Negation(ref expr) => {
                println(tabs, "_neg");
                expr.pretty_print(tabs + 1);
            },
            Expression::Not(ref expr) => {
                println(tabs, "_comp");
                expr.pretty_print(tabs + 1);
            },
            Expression::Identifier(ref variable) => {
                println(tabs, "_object");
                println(tabs + 1, variable);
            },
            Expression::IntLiteral(ref value) => {
                println(tabs, "_int");
                println(tabs + 1, &format!("{}", value));
            },
            Expression::StringLiteral(ref value) => {
                println(tabs, "_string");
                println(tabs + 1, value);
            },
            Expression::True => {
                println(tabs, "_bool");
                println(tabs + 1, "1");
            },
            Expression::False => {
                println(tabs, "_bool");
                println(tabs + 1, "0");
            },
            Expression::NoExpr => {
                println(tabs, "_no_expr");
            }
        }
    }
}

impl Printable for CaseBranch {
    fn pretty_print(&self, tabs: u32) {
        println(tabs, "_branch");
        println(tabs + 1, &self.name);
        println(tabs + 1, &self.cool_type);
        self.expr.pretty_print(tabs + 1);
    }
}

pub fn println(tabs: u32, string: &str) {
    for _ in 0 .. tabs {
        print!("  ");
    }
    println!("{}", string);
}

impl Printable for Token {
    fn pretty_print(&self, tabs: u32) {
        print!("Token: ");
        match *self {
            Token::Type(ref class) => println!("Type: {}", class),
            Token::Identifier(ref var) => println!("Identifier: {}", var),
            Token::IntegerLiteral(ref val) => println!("Integer: {}", val),
            Token::StringLiteral(ref val) => println!("String: {}", val),
            Token::Case => println!("case"),
            Token::Class => println!("class"),
            Token::Else => println!("else"),
            Token::Esac => println!("esac"),
            Token::False => println!("false"),
            Token::Fi => println!("fi"),
            Token::If => println!("if"),
            Token::In => println!("in"),
            Token::Inherits => println!("inherits"),
            Token::Isvoid => println!("isvoid"),
            Token::Let => println!("let"),
            Token::Loop => println!("loop"),
            Token::New => println!("new"),
            Token::Not => println!("not"),
            Token::Of => println!("of"),
            Token::Pool => println!("pool"),
            Token::Then => println!("then"),
            Token::True => println!("true"),
            Token::While => println!("while"),
            Token::LeftBrace => println!("leftBrace"),
            Token::RightBrace => println!("rightBrace"),
            Token::LeftParen => println!("leftParen"),
            Token::RightParen => println!("rightParen"),
            Token::Colon => println!("colon"),
            Token::Semicolon => println!("semicolon"),
            Token::Dot => println!("dot"),
            Token::Comma => println!("comma"),
            Token::At => println!("at"),
            Token::Plus => println!("plus"),
            Token::Minus => println!("minus"),
            Token::Times => println!("times"),
            Token::Divide => println!("divide"),
            Token::Tilde => println!("tilde"),
            Token::LessThan => println!("lessThan"),
            Token::Equal => println!("equal"),
            Token::LessThanEqual => println!("lessThanEqual"),
            Token::Arrow => println!("arrow"),
            Token::EOF => println!("<EOF>")
        }
    }
}
