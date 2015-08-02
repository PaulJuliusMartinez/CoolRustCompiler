use ast;
use lexer::Token;
use std::rc::Rc;

macro_rules! match_single {
    ($tok:expr, $expected:pat, $states:ident, $new_state:expr) => {{
        match *$tok {
            $expected => {
                $states.push($new_state);
            },
            _ => {
                println!("Errored in state {}", $states.last().unwrap());
                return None;
            }
        }
    }}
}

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

macro_rules! reduce {
    ($states:expr, $rule_num:expr, $rule_len:expr) => {{
        for _ in 0 .. $rule_len {
            $states.pop();
        }
        let curr = *$states.last().unwrap();
        $states.push(goto(curr, $rule_num));
    }}
}

pub fn parse_cool_program(tokens: &Vec<Token>) -> Option<ast::Program> {
    let mut classLists: Vec<Box<ast::Class>> = Vec::new();
    let mut lastClass: Option<Box<ast::Class>> = None;
    let mut types: Vec<ast::CoolType> = Vec::new();
    let mut features: Vec<Box<ast::Feature>> = Vec::new();
    let mut formals: Vec<Box<ast::Formal>> = Vec::new();
    let mut identifiers: Vec<ast::Symbol> = Vec::new();
    let mut expressions: Vec<Box<ast::Expression>> = Vec::new();
    let mut expressionLists: Vec<Vec<Box<ast::Expression>>> = Vec::new();
    let mut caseBranches: Vec<ast::CaseBranch> = Vec::new();
    let mut states: Vec<i32> = vec![0];

    // Constants
    let object = Rc::new("Object".to_string());

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
                14 => {
                    match_type!(curr, states, 17, types);
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
                21 => {
                    match_single!(curr, Token::Semicolon, states, 24);
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
        17 => {
            match rule {
                14 => 21,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        _ => {
            panic!("Haven't implemented goto for state {} yet", state);
        }
    }
}
