use ast;
use lexer::Token;
use std::rc::Rc;

// Used when there's only one possible token to expect and
// we don't need to store its value anywhere
macro_rules! match_single {
    ($tok:expr, $expected:pat, $states:ident, $new_state:expr) => {{
        match *$tok {
            $expected => {
                $states.push($new_state);
            },
            ref e @ _ => {
                println!("Errored in state {}, got {:?}", $states.last().unwrap(), e);
                return None;
            }
        }
    }}
}

// Used when the only thing we expect is a type.
macro_rules! match_type {
    ($tok:expr, $states:ident, $new_state:expr, $types:expr) => {{
        match *$tok {
            Token::Type(ref type_name) => {
                $types.push(type_name.clone());
                $states.push($new_state);
            },
            _ => {
                println!("Errored in state {}, expected a type", $states.last().unwrap());
                return None;
            }
        }
    }}
}

// Used when the only thing we expect is an identifier.
macro_rules! match_identifier {
    ($tok:expr, $states:ident, $new_state:expr, $ids:expr) => {{
        match *$tok {
            Token::Identifier(ref id) => {
                $ids.push(id.clone());
                $states.push($new_state);
            },
            _ => {
                println!("Errored in state {}, expected an identifier", $states.last().unwrap());
                return None;
            }
        }
    }}
}

macro_rules! match_single_capture {
    ($tok:expr, $expected:pat, $states:ident, $new_state:expr, $body:stmt) => {{
        match *$tok {
            $expected => {
                $states.push($new_state);
                $body
            },
            _ => {
                println!("Errored in state {}", $states.last().unwrap());
                return None;
            }
        }
    }}
}

// Pops states and then looks up what state we should go to next
// in the goto table
macro_rules! reduce {
    ($states:expr, $rule_num:expr, $rule_len:expr) => {{
        for _ in 0 .. $rule_len {
            $states.pop();
        }
        let curr = *$states.last().unwrap();
        $states.push(goto(curr, $rule_num));
    }}
}

// Matches the start of an expression
macro_rules! expression_start {
    ($tok:expr, $states:ident, $ids:ident, $exprs:ident) => {
        match *$tok {
            Token::Identifier(ref id) => {
                $exprs.push(Box::new(ast::Expression::Identifier(id.clone())));
                $states.push(30);
            },
            Token::If => { $states.push(31); },
            Token::While => { $states.push(32); },
            Token::Let => { $states.push(33); },
            Token::Case => { $states.push(34); },
            Token::New => { $states.push(35); },
            Token::Isvoid => { $states.push(36); },
            Token::Not => { $states.push(37); },
            Token::LeftBrace => { $states.push(38); },
            Token::LeftParen => { $states.push(39); },
            Token::Tilde => { $states.push(40); },
            Token::StringLiteral(ref string) => {
                $exprs.push(Box::new(ast::Expression::StringLiteral(string.clone())));
                $states.push(42);
            },
            Token::True => {
                $exprs.push(Box::new(ast::Expression::True));
                $states.push(43);
            },
            Token::False => {
                $exprs.push(Box::new(ast::Expression::False));
                $states.push(44);
            },
            Token::IntegerLiteral(val) => {
                $exprs.push(Box::new(ast::Expression::IntLiteral(val)));
                $states.push(160);
            },
            ref e @ _ => {
                println!("Errored in state {}, got {:?}", $states.last().unwrap(), e);
                return None;
            }
        }
    }
}

// Matches after reducing an expression
macro_rules! after_expression {
    ($tok:expr, $states:ident, $expected:pat, $new_state:expr, $types:ident, $no_type:ident, $use_token:ident) => {{
        match *$tok {
            $expected => { $states.push($new_state); },
            Token::Dot => {
                $types.push($no_type.clone());
                $states.push(101);
                $use_token = false;
            },
            Token::At => { $states.push(46); },
            Token::Plus => { $states.push(47); },
            Token::Minus => { $states.push(48); },
            Token::Times => { $states.push(49); },
            Token::Divide => { $states.push(50); },
            Token::LessThan => { $states.push(51); },
            Token::LessThanEqual => { $states.push(52); },
            Token::Equal => { $states.push(53); },
            ref e @ _ => {
                println!("Errored in state {}, got {:?}", $states.last().unwrap(), e);
                return None;
            }
        }
    }}
}

macro_rules! on_expression_goto {
    ($state:ident, $rule:ident, $new_state:expr) => {{
        match $rule {
            15 ... 40 => {
                $new_state
            },
            _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", $state, $rule)
        }
    }}
}

pub fn parse_cool_program(tokens: &Vec<Token>) -> Option<ast::Program> {
    let mut classLists: Vec<Box<ast::Class>> = Vec::new();
    let mut lastClass: Option<Box<ast::Class>> = None;
    let mut types: Vec<ast::CoolType> = Vec::new();
    let mut features: Vec<Box<ast::Feature>> = Vec::new();
    let mut formals: Vec<ast::Formal> = Vec::new();
    let mut identifiers: Vec<ast::Symbol> = Vec::new();
    let mut expressions: Vec<Box<ast::Expression>> = Vec::new();
    let mut expressionLists: Vec<Vec<Box<ast::Expression>>> = Vec::new();
    let mut caseBranches: Vec<ast::CaseBranch> = Vec::new();
    let mut states: Vec<i32> = vec![0];

    // Constants
    let object = Rc::new("Object".to_string());
    let no_type = Rc::new("_no_type".to_string());

    let mut tokensIter = tokens.iter().peekable();
    loop {
        let mut should_consume = true;
        if let Some(&curr) = tokensIter.peek() {
            println!("{:?}, next: {:?}", states, curr);
            match *states.last().unwrap() {
                0 => {
                    match_single!(curr, Token::Class, states, 4);
                },
                1 => {
                    match *curr {
                        Token::Class => {
                            states.push(4);
                        },
                        Token::EOF => {
                            println!("Successfull parse!");
                            break;
                        },
                        _ => {
                            println!("Errored in state {}", states.last().unwrap());
                            return None;
                        }
                    }
                },
                2 => {
                    match_single!(curr, Token::Semicolon, states, 3);
                },
                3 => {
                    classLists.push(lastClass.unwrap());
                    lastClass = None;
                    // 0:      P -> P C ;
                    reduce!(states, 0, 3);
                    should_consume = false;
                },
                4 => {
                    match_type!(curr, states, 5, types);
                },
                5 => {
                    match *curr {
                        Token::Inherits => {
                            states.push(7);
                        },
                        Token::LeftBrace => {
                            types.push(object.clone());
                            states.push(6);
                            should_consume = false;
                        },
                        ref e @ _ => {
                            println!("Expected 'inherits' or '{{' but found {:?}", e);
                            return None;
                        }
                    }
                },
                6 => {
                    match_single!(curr, Token::LeftBrace, states, 8);
                },
                7 => {
                    match_type!(curr, states, 9, types);
                },
                8 => {
                    match *curr {
                        Token::RightBrace => {
                            states.push(10);
                            should_consume = false;
                        },
                        Token::Identifier(_) => {
                            states.push(10);
                            should_consume = false;
                        },
                        ref e @ _ => {
                            println!("Expected 'identifier' or '}}' but found {:?}", e);
                            return None;
                        }
                    }
                },
                9 => {
                    // 3:      I -> inherits TYPE
                    reduce!(states, 3, 2);
                    should_consume = false;
                },
                10 => {
                    match *curr {
                        Token::RightBrace => {
                            states.push(11);
                        },
                        Token::Identifier(ref id) => {
                            states.push(12);
                            identifiers.push(id.clone());
                        },
                        ref e @ _ => {
                            println!("Expected 'identifier' or '}}' but found {:?}", e);
                            return None;
                        }
                    }
                },
                11 => {
                    let inheritsFrom = types.pop().unwrap();
                    let className = types.pop().unwrap();
                    lastClass = Some(Box::new(ast::Class {
                        name: className,
                        parent: inheritsFrom,
                        features: features
                    }));
                    features = Vec::new();
                    // 2:      C -> class TYPE I { F }
                    reduce!(states, 2, 6);
                    should_consume = false;
                },
                12 => {
                    match *curr {
                        Token::LeftParen => {
                            states.push(13);
                        },
                        Token::Colon => {
                            states.push(14);
                        },
                        ref e @ _ => {
                            println!("Expected '(' or ':' but found {:?}", e);
                            return None;
                        }
                    }
                },
                13 => {
                    match *curr {
                        Token::RightParen => {
                            states.push(125);
                        },
                        Token::Identifier(_) => {
                            states.push(15);
                            should_consume = false;
                        },
                        ref e @ _ => {
                            println!("Expected ')' or 'identifier' but found {:?}", e);
                            return None;
                        }
                    }
                },
                14 => {
                    match_type!(curr, states, 17, types);
                },
                15 => {
                    match_identifier!(curr, states, 16, identifiers);
                },
                16 => {
                    match_single!(curr, Token::Colon, states, 19);
                },
                17 => {
                    match *curr {
                        Token::Arrow => {
                            states.push(28);
                        },
                        Token::Semicolon => {
                            states.push(21);
                            expressions.push(Box::new(ast::Expression::NoExpr));
                            should_consume = false;
                        },
                        ref e @ _ => {
                            println!("Expected '<-' or ';' but found {:?}", e);
                            return None;
                        }
                    }
                },
                18 => {
                    match *curr {
                        Token::RightParen => {
                            // 8:      R -> X A
                            reduce!(states, 8, 2);
                            should_consume = false;
                        },
                        Token::Comma => {
                            states.push(22);
                        },
                        ref e @ _ => {
                            println!("Expected ')' or ',' but found {:?}", e);
                            return None;
                        }
                    }
                },
                19 => {
                    match_type!(curr, states, 20, types);
                },
                20 => {
                    let formal_type = types.pop().unwrap();
                    let formal_name = identifiers.pop().unwrap();
                    formals.push(ast::Formal {
                        name: formal_name,
                        cool_type: formal_type
                    });
                    // 10:     A -> id : TYPE
                    reduce!(states, 10, 3);
                    should_consume = false;
                },
                21 => {
                    match_single!(curr, Token::Semicolon, states, 24);
                },
                22 => {
                    // 11:     X -> X A ,
                    reduce!(states, 11, 3);
                    should_consume = false;
                },
                23 => {
                    match_single!(curr, Token::Colon, states, 26);
                },
                24 => {
                    let init = expressions.pop().unwrap();
                    let var_type = types.pop().unwrap();
                    let var_name = identifiers.pop().unwrap();
                    features.push(Box::new(ast::Feature::Attribute{
                        name: var_name,
                        cool_type: var_type,
                        expr: init
                    }));
                    // 6:      F -> F id : TYPE W ;
                    reduce!(states, 6, 6);
                    should_consume = false;
                },
                26 => {
                    match_type!(curr, states, 27, types);
                },
                27 => {
                    match_single!(curr, Token::LeftBrace, states, 126);
                },
                28 => {
                    expression_start!(curr, states, identifiers, expressions);
                },
                29 => {
                    match *curr {
                        Token::Dot => {
                            types.push(no_type.clone());
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => { states.push(46); },
                        Token::Plus => { states.push(47); },
                        Token::Minus => { states.push(48); },
                        Token::Times => { states.push(49); },
                        Token::Divide => { states.push(50); },
                        Token::LessThan => { states.push(51); },
                        Token::LessThanEqual => { states.push(52); },
                        Token::Equal => { states.push(53); },
                        Token::Semicolon => {
                            // 13:     W -> <- E
                            reduce!(states, 13, 2);
                            should_consume = false;
                        },
                        ref e @ _ => {
                            println!("Errored in state {}, got {:?}", states.last().unwrap(), e);
                            return None;
                        }
                    }
                },
                42 => {
                    // 37:     E -> <string>
                    reduce!(states, 37, 1);
                    should_consume = false;
                },
                43 => {
                    // 38:     E -> true
                    reduce!(states, 38, 1);
                    should_consume = false;
                },
                44 => {
                    // 39:     E -> true
                    reduce!(states, 39, 1);
                    should_consume = false;
                },
                125 => {
                    match_single!(curr, Token::RightParen, states, 23);
                },
                126 => {
                    expression_start!(curr, states, identifiers, expressions);
                },
                127 => {
                    // TODO: FIX THIS
                    after_expression!(curr, states, Token::RightBrace, 128,
                                      types, no_type, should_consume);
                },
                128 => {
                    match_single!(curr, Token::Semicolon, states, 129);
                },
                129 => {
                    let fun_body = expressions.pop().unwrap();
                    let return_type = types.pop().unwrap();
                    let fun_name = identifiers.pop().unwrap();
                    features.push(Box::new(ast::Feature::Method{
                        name: fun_name,
                        params: formals,
                        return_type: return_type,
                        body: fun_body
                    }));
                    formals = Vec::new();
                    // 5:      F -> F id ( R ) : TYPE { E } ;
                    reduce!(states, 5, 11);
                    should_consume = false;
                },
                150 => {
                    match_single!(curr, Token::Semicolon, states, 151);
                },
                151 => {
                    classLists.push(lastClass.unwrap());
                    lastClass = None;
                    // 1:      P -> C ;
                    reduce!(states, 0, 2);
                    should_consume = false;
                },
                160 => {
                    // 40:     E -> <int literal>
                    reduce!(states, 40, 1);
                    should_consume = false;
                },
                _ => { println!("Haven't implemented state {} yet", states.last().unwrap()); }
            }
        }
        if should_consume {
            let _ = tokensIter.next();
        }
    }

    return Some(ast::Program {
        classes: classLists
    });
}

fn goto(state: i32, rule: i32) -> i32 {
    println!("goto {} {}", state, rule);
    match state {
        0 => {
            match rule {
                0 | 1 => 1,
                2 => 150,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        1 => {
            match rule {
                2 => 2,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        5 => {
            match rule {
                3 => 6,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        8 => {
            match rule {
                5 => 10,
                6 => 10,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        13 => {
            match rule {
                8 => 125,
                11 => 15,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        15 => {
            match rule {
                10 => 18,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        17 => {
            match rule {
                13 => 21,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        28 => {
            on_expression_goto!(state, rule, 29)
        },
        126 => {
            on_expression_goto!(state, rule, 127)
        },
        _ => {
            panic!("Haven't implemented goto for state {} yet", state);
        }
    }
}
