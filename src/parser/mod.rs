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
        println!("Reducing using rule #{}", $rule_num);
        for _ in 0 .. $rule_len {
            $states.pop();
        }
        let curr = *$states.last().unwrap();
        $states.push(goto(curr, $rule_num));
    }}
}

// Matches the start of an expression
macro_rules! expression_start {
    ($tok:expr, $states:ident, $ids:ident, $exprs:ident, $expr_lists:ident) => {
        match *$tok {
            Token::Identifier(ref id) => {
                $ids.push(id.clone());
                $states.push(30);
            },
            Token::If => { $states.push(31); },
            Token::While => { $states.push(32); },
            Token::Let => { $states.push(33); },
            Token::Case => { $states.push(34); },
            Token::New => { $states.push(35); },
            Token::Isvoid => { $states.push(36); },
            Token::Not => { $states.push(37); },
            Token::LeftBrace => {
                $expr_lists.push(Vec::new());
                $states.push(38);
            },
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
    ($tok:expr, $states:ident, $expected:pat, $new_state:expr, $types:ident, $no_type:ident, $use_token:ident, $isStatic:ident) => {{
        match *$tok {
            $expected => { $states.push($new_state); },
            Token::Dot => {
                $isStatic.push(false);
                $states.push(101);
                $use_token = false;
            },
            Token::At => {
                $isStatic.push(true);
                $states.push(46);
            },
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

macro_rules! after_static_goto {
    ($state:ident, $rule:ident) => {{
        match $rule {
            45 => 45,
            _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", $state, $rule)
        }
    }}
}

pub fn parse_cool_program(tokens: &Vec<Token>) -> Option<ast::Program> {
    let mut class_lists: Vec<Box<ast::Class>> = Vec::new();
    let mut last_class: Option<Box<ast::Class>> = None;
    let mut types: Vec<ast::CoolType> = Vec::new();
    let mut features: Vec<Box<ast::Feature>> = Vec::new();
    let mut formals: Vec<ast::Formal> = Vec::new();
    let mut identifiers: Vec<ast::Symbol> = Vec::new();
    let mut expressions: Vec<Box<ast::Expression>> = Vec::new();
    let mut expression_lists: Vec<Vec<Box<ast::Expression>>> = Vec::new();
    let mut case_branches: Vec<ast::CaseBranch> = Vec::new();
    let mut states: Vec<i32> = vec![0];
    let mut is_statics: Vec<bool> = Vec::new();

    // Constants
    let object = Rc::new("Object".to_string());
    let no_type = Rc::new("_no_type".to_string());
    let self_obj = Rc::new("self".to_string());

    let mut tokens_iter = tokens.iter().peekable();
    loop {
        let mut should_consume = true;
        if let Some(&curr) = tokens_iter.peek() {
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
                    class_lists.push(last_class.unwrap());
                    last_class = None;
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
                    let inherits_from = types.pop().unwrap();
                    let class_name = types.pop().unwrap();
                    last_class = Some(Box::new(ast::Class {
                        name: class_name,
                        parent: inherits_from,
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
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                29 => {
                    match *curr {
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(45);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
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
                30 => {
                    match *curr {
                        Token::Arrow => {
                            states.push(54);
                        },
                        Token::LeftParen => {
                            states.push(55);
                            expression_lists.push(Vec::new());
                        },
                        _ => {
                            expressions.push(Box::new(ast::Expression::Identifier(
                                identifiers.pop().unwrap()
                            )));
                            // 36:     E -> <id>
                            reduce!(states, 36, 1);
                            should_consume = false;
                        }
                    }
                },
                31 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                32 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                33 => {
                    match_identifier!(curr, states, 57, identifiers);
                },
                35 => {
                    match_type!(curr, states, 94, types);
                },
                36 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                37 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                38 => {
                    states.push(110);
                    should_consume = false;
                },
                40 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                39 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                41 => {
                    match *curr {
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
                        _ => {
                            let expr = expressions.pop().unwrap();
                            expressions.push(Box::new(ast::Expression::Negation(
                                expr
                            )));
                            // 33:     E -> ~ E
                            reduce!(states, 33, 2);
                            should_consume = false;
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
                45 => {
                    match_single!(curr, Token::Dot, states, 101);
                },
                46 => {
                    match_type!(curr, states, 124, types);
                },
                54 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                55 => {
                    match *curr {
                        Token::RightParen => {
                            states.push(104);
                        },
                        _ => {
                            states.push(96);
                        }
                    }
                    should_consume = false;
                },
                56 => {
                    after_expression!(curr, states, Token::Then, 73,
                                      types, no_type, should_consume, is_statics);
                },
                57 => {
                    match_single!(curr, Token::Colon, states, 58);
                },
                58 => {
                    match_type!(curr, states, 59, types);
                },
                59 => {
                    match *curr {
                        Token::Arrow => {
                            states.push(28);
                        },
                        Token::In => {
                            states.push(60);
                            expressions.push(Box::new(ast::Expression::NoExpr));
                            should_consume = false;
                        },
                        Token::Comma => {
                            states.push(60);
                            expressions.push(Box::new(ast::Expression::NoExpr));
                            should_consume = false;
                        },
                        ref e @ _ => {
                            println!("Expected '<-', 'in' or ',' but found {:?}", e);
                            return None;
                        }
                    }
                },
                60 => {
                    match *curr {
                        Token::In => {
                            states.push(61);
                        },
                        Token::Comma => {
                            states.push(64);
                        },
                        ref e @ _ => {
                            println!("Expected 'in' or ',' but found {:?}", e);
                            return None;
                        }
                    }
                },
                61 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                62 => {
                    match *curr {
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
                        Token::Plus => { states.push(47); },
                        Token::Minus => { states.push(48); },
                        Token::Times => { states.push(49); },
                        Token::Divide => { states.push(50); },
                        Token::LessThan => { states.push(51); },
                        Token::LessThanEqual => { states.push(52); },
                        Token::Equal => { states.push(53); },
                        _ => {
                            let body = expressions.pop().unwrap();
                            let init = expressions.pop().unwrap();
                            let var_type = types.pop().unwrap();
                            let var_name = identifiers.pop().unwrap();

                            expressions.push(Box::new(ast::Expression::Let(
                                var_name,
                                var_type,
                                init,
                                body
                            )));

                            // 20:     E -> let id : TYPE W in E
                            reduce!(states, 20, 7);
                            should_consume = false;
                        }
                    }
                },
                63 => {
                    let body = expressions.pop().unwrap();
                    let init = expressions.pop().unwrap();
                    let var_type = types.pop().unwrap();
                    let var_name = identifiers.pop().unwrap();

                    expressions.push(Box::new(ast::Expression::Let(
                        var_name,
                        var_type,
                        init,
                        body
                    )));

                    // 21:     E -> let id : TYPE W N
                    reduce!(states, 21, 6);
                    should_consume = false;
                },
                64 => {
                    match_identifier!(curr, states, 65, identifiers);
                },
                65 => {
                    match_single!(curr, Token::Colon, states, 66);
                },
                66 => {
                    match_type!(curr, states, 67, types);
                },
                67 => {
                    match *curr {
                        Token::Arrow => {
                            states.push(28);
                        },
                        Token::In => {
                            states.push(68);
                            expressions.push(Box::new(ast::Expression::NoExpr));
                            should_consume = false;
                        },
                        Token::Comma => {
                            states.push(68);
                            expressions.push(Box::new(ast::Expression::NoExpr));
                            should_consume = false;
                        },
                        ref e @ _ => {
                            println!("Expected '<-', 'in' or ',' but found {:?}", e);
                            return None;
                        }
                    }
                },
                68 => {
                    match *curr {
                        Token::In => {
                            states.push(152);
                        },
                        Token::Comma => {
                            states.push(64);
                        },
                        ref e @ _ => {
                            println!("Expected 'in' or ',' but found {:?}", e);
                            return None;
                        }
                    }
                },
                69 => {
                    after_expression!(curr, states, Token::Loop, 70,
                                      types, no_type, should_consume, is_statics);
                },
                70 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                71 => {
                    after_expression!(curr, states, Token::Pool, 72,
                                      types, no_type, should_consume, is_statics);
                },
                72 => {
                    let body = expressions.pop().unwrap();
                    let condition = expressions.pop().unwrap();
                    expressions.push(Box::new(ast::Expression::While(
                        condition,
                        body
                    )));
                    // 19:     E -> while E loop E pool
                    reduce!(states, 19, 5);
                    should_consume = false;
                },
                73 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                74 => {
                    after_expression!(curr, states, Token::Else, 75,
                                      types, no_type, should_consume, is_statics);
                },
                75 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                76 => {
                    after_expression!(curr, states, Token::Fi, 77,
                                      types, no_type, should_consume, is_statics);
                },
                77 => {
                    let false_branch = expressions.pop().unwrap();
                    let true_branch = expressions.pop().unwrap();
                    let condition = expressions.pop().unwrap();
                    expressions.push(Box::new(ast::Expression::If(
                        condition,
                        true_branch,
                        false_branch
                    )));
                    // 18:     E -> if E then E else E fi
                    reduce!(states, 18, 7);
                    should_consume = false;
                },
                94 => {
                    expressions.push(Box::new(ast::Expression::New(
                        types.pop().unwrap()
                    )));
                    // 23:     E -> new TYPE
                    reduce!(states, 23, 2);
                    should_consume = false;
                },
                95 => {
                    match *curr {
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
                        Token::Plus => { states.push(47); },
                        Token::Minus => { states.push(48); },
                        Token::Times => { states.push(49); },
                        Token::Divide => { states.push(50); },
                        Token::LessThan => { states.push(51); },
                        Token::LessThanEqual => { states.push(52); },
                        Token::Equal => { states.push(53); },
                        _ => {
                            let expr = expressions.pop().unwrap();
                            let var_name = identifiers.pop().unwrap();
                            expressions.push(Box::new(ast::Expression::Assign(
                                var_name,
                                expr
                            )));
                            // 15:     E -> id <- E
                            reduce!(states, 15, 3);
                            should_consume = false;
                        }
                    }
                },
                96 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                97 => {
                    match *curr {
                        Token::RightParen => {
                            expression_lists.last_mut().unwrap().push(
                                expressions.pop().unwrap()
                            );
                            // 41:     L -> G E
                            reduce!(states, 41, 2);
                            should_consume = false;
                        },
                        Token::Comma => {
                            states.push(98);
                        },
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
                        Token::Plus => { states.push(47); },
                        Token::Minus => { states.push(48); },
                        Token::Times => { states.push(49); },
                        Token::Divide => { states.push(50); },
                        Token::LessThan => { states.push(51); },
                        Token::LessThanEqual => { states.push(52); },
                        Token::Equal => { states.push(53); },
                        ref e @ _ => {
                            println!("Errored in state {}, got {:?}", states.last().unwrap(), e);
                            return None;
                        }
                    }
                },
                98 => {
                    expression_lists.last_mut().unwrap().push(
                        expressions.pop().unwrap()
                    );
                    // 43:     G -> G E ,
                    reduce!(states, 43, 3);
                    should_consume = false;
                },
                101 => {
                    match_identifier!(curr, states, 102, identifiers);
                },
                102 => {
                    match *curr {
                        Token::LeftParen => {
                            expression_lists.push(Vec::new());
                            states.push(103);
                        },
                        ref e @ _ => {
                            println!("In state 102, expected '(' but found {:?}", e);
                            return None;
                        }
                    }
                },
                103 => {
                    match *curr {
                        Token::RightParen => {
                            states.push(105);
                        },
                        _ => {
                            states.push(96);
                        }
                    }
                    should_consume = false;
                },
                104 => {
                    match_single!(curr, Token::RightParen, states, 161);
                },
                105 => {
                    match_single!(curr, Token::RightParen, states, 106);
                },
                106 => {
                    let args = expression_lists.pop().unwrap();
                    let method_name = identifiers.pop().unwrap();
                    let obj = expressions.pop().unwrap();
                    if is_statics.pop().unwrap() {
                        let static_type = types.pop().unwrap();
                        expressions.push(Box::new(ast::Expression::StaticDispatch(
                            obj,
                            static_type,
                            method_name,
                            args
                        )));
                    } else {
                        expressions.push(Box::new(ast::Expression::Dispatch(
                            obj,
                            method_name,
                            args
                        )));
                    }
                    // 17:     E -> E T . id ( L )
                    reduce!(states, 17, 7);
                    should_consume = false;
                },
                107 => {
                    match *curr {
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
                        _ => {
                            let expr = expressions.pop().unwrap();
                            expressions.push(Box::new(ast::Expression::IsVoid(
                                expr
                            )));
                            // 24:     E -> isvoid E
                            reduce!(states, 24, 2);
                            should_consume = false;
                        }
                    }
                },
                108 => {
                    after_expression!(curr, states, Token::RightParen, 109,
                                      types, no_type, should_consume, is_statics);
                },
                109 => {
                    // Don't have to pop the expression, we're just going
                    // to put it right back
                    // 34:     E -> ( E )
                    reduce!(states, 34, 3);
                    should_consume = false;
                },
                110 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                111 => {
                    after_expression!(curr, states, Token::Semicolon, 112,
                                      types, no_type, should_consume, is_statics);
                },
                112 => {
                    match *curr {
                        Token::RightBrace => {
                            states.push(113);
                        },
                        _ => {
                            expression_lists.last_mut().unwrap().push(
                                expressions.pop().unwrap()
                            );
                            // 51:     B -> B E ;
                            reduce!(states, 51, 3);
                            should_consume = false;
                        }
                    }
                },
                113 => {
                    let mut body = expression_lists.pop().unwrap();
                    body.push(expressions.pop().unwrap());
                    expressions.push(Box::new(ast::Expression::Block(
                        body
                    )));

                    // 35:     E -> { B E ; }
                    reduce!(states, 35, 5);
                    should_consume = false;
                }
                116 => {
                    match *curr {
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
                        Token::Plus => { states.push(47); },
                        Token::Minus => { states.push(48); },
                        Token::Times => { states.push(49); },
                        Token::Divide => { states.push(50); },
                        Token::LessThan => { states.push(51); },
                        Token::LessThanEqual => { states.push(52); },
                        Token::Equal => { states.push(53); },
                        _ => {
                            let expr = expressions.pop().unwrap();
                            expressions.push(Box::new(ast::Expression::Not(
                                expr
                            )));
                            // 25:     E -> not E
                            reduce!(states, 25, 2);
                            should_consume = false;
                        }
                    }
                },
                124 => {
                    // 45:     T -> @ TYPE
                    reduce!(states, 45, 2);
                    should_consume = false;
                },
                125 => {
                    match_single!(curr, Token::RightParen, states, 23);
                },
                126 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                127 => {
                    after_expression!(curr, states, Token::RightBrace, 128,
                                      types, no_type, should_consume, is_statics);
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
                    class_lists.push(last_class.unwrap());
                    last_class = None;
                    // 1:      P -> C ;
                    reduce!(states, 0, 2);
                    should_consume = false;
                },
                152 => {
                    expression_start!(curr, states, identifiers, expressions, expression_lists);
                },
                153 => {
                    match *curr {
                        Token::Dot => {
                            is_statics.push(false);
                            states.push(101);
                            should_consume = false;
                        },
                        Token::At => {
                            is_statics.push(true);
                            states.push(46);
                        },
                        Token::Plus => { states.push(47); },
                        Token::Minus => { states.push(48); },
                        Token::Times => { states.push(49); },
                        Token::Divide => { states.push(50); },
                        Token::LessThan => { states.push(51); },
                        Token::LessThanEqual => { states.push(52); },
                        Token::Equal => { states.push(53); },
                        _ => {
                            let body = expressions.pop().unwrap();
                            let init = expressions.pop().unwrap();
                            let var_type = types.pop().unwrap();
                            let var_name = identifiers.pop().unwrap();

                            expressions.push(Box::new(ast::Expression::Let(
                                var_name,
                                var_type,
                                init,
                                body
                            )));

                            // 47:     N -> , id : TYPE W in E
                            reduce!(states, 47, 7);
                            should_consume = false;
                        }
                    }
                },
                154 => {
                    let body = expressions.pop().unwrap();
                    let init = expressions.pop().unwrap();
                    let var_type = types.pop().unwrap();
                    let var_name = identifiers.pop().unwrap();

                    expressions.push(Box::new(ast::Expression::Let(
                        var_name,
                        var_type,
                        init,
                        body
                    )));

                    // 48:     N -> , id : TYPE W N
                    reduce!(states, 48, 6);
                    should_consume = false;
                },
                160 => {
                    // 40:     E -> <int literal>
                    reduce!(states, 40, 1);
                    should_consume = false;
                },
                161 => {
                    expressions.push(Box::new(ast::Expression::Dispatch(
                        Box::new(ast::Expression::Identifier(self_obj.clone())),
                        identifiers.pop().unwrap(),
                        expression_lists.pop().unwrap()
                    )));
                    // 16:     E -> id ( L )
                    reduce!(states, 16, 4);
                    should_consume = false;
                },
                _ => { println!("Haven't implemented state {} yet", states.last().unwrap()); }
            }
        }
        if should_consume {
            let _ = tokens_iter.next();
        }
    }

    return Some(ast::Program {
        classes: class_lists
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
        29 => {
            after_static_goto!(state, rule)
        },
        31 => {
            on_expression_goto!(state, rule, 56)
        },
        32 => {
            on_expression_goto!(state, rule, 69)
        },
        36 => {
            on_expression_goto!(state, rule, 107)
        },
        37 => {
            on_expression_goto!(state, rule, 116)
        },
        38 => {
            match rule {
                51 => 110,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        39 => {
            on_expression_goto!(state, rule, 108)
        },
        40 => {
            on_expression_goto!(state, rule, 41)
        },
        41 => {
            after_static_goto!(state, rule)
        },
        54 => {
            on_expression_goto!(state, rule, 95)
        },
        55 => {
            match rule {
                41 => 104,
                43 => 96,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        56 => {
            after_static_goto!(state, rule)
        },
        59 => {
            match rule {
                13 => 60,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        60 => {
            match rule {
                47 => 63,
                48 => 63,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        61 => {
            on_expression_goto!(state, rule, 62)
        },
        62 => {
            after_static_goto!(state, rule)
        },
        67 => {
            match rule {
                13 => 68,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        68 => {
            match rule {
                47 => 154,
                48 => 154,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        69 => {
            after_static_goto!(state, rule)
        },
        70 => {
            on_expression_goto!(state, rule, 71)
        },
        71 => {
            after_static_goto!(state, rule)
        },
        73 => {
            on_expression_goto!(state, rule, 74)
        },
        74 => {
            after_static_goto!(state, rule)
        },
        75 => {
            on_expression_goto!(state, rule, 76)
        },
        76 => {
            after_static_goto!(state, rule)
        },
        95 => {
            after_static_goto!(state, rule)
        },
        96 => {
            on_expression_goto!(state, rule, 97)
        },
        97 => {
            after_static_goto!(state, rule)
        },
        103 => {
            match rule {
                41 => 105,
                43 => 96,
                _ => panic!("GOTO PANIC IN STATE {} AFTER REDUCING RULE {}", state, rule)
            }
        },
        107 => {
            after_static_goto!(state, rule)
        },
        108 => {
            after_static_goto!(state, rule)
        },
        110 => {
            on_expression_goto!(state, rule, 111)
        },
        111 => {
            after_static_goto!(state, rule)
        },
        116 => {
            after_static_goto!(state, rule)
        },
        126 => {
            on_expression_goto!(state, rule, 127)
        },
        127 => {
            after_static_goto!(state, rule)
        },
        152 => {
            on_expression_goto!(state, rule, 153)
        },
        153 => {
            after_static_goto!(state, rule)
        },
        _ => {
            panic!("Haven't implemented goto for state {} yet", state);
        }
    }
}
