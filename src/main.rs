use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;

/*
enum CoolTokenType {
    ClassName (Rc<String>),
    VariableName (Rc<String>),
    IntegerLiteral (Rc<String>),
    StringLiteral (Rc<String>),
    Comment,
    Whitespace,
    KwCase,
    KwClass,
    KwElse,
    KwEsac,
    KwFalse,
    KwFi,
    KwIf,
    KwIn,
    KwInherits,
    KwIsvoid,
    KwLet,
    KwLoop,
    KwNew,
    KwNot,
    KwOf,
    KwPool,
    KwThen,
    KwTrue,
    KwWhile,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Colon,
    SemiColon,
    Period,
    Comma,
    AtSign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Tilde,
    LessThan,
    Equal,
    LessThanEqual,
    Arrow
}
*/

#[derive(PartialEq)]
enum LexerState {
    Start,
    Identifier,
    Number,
    LessThan,
    CommentOrMinus,
    CommentOrParens,
    SingleLineComment,
    MultiLineComment,
    MultiLineCommentEnd,
    String,
    StringEscape
}

fn main() {
    let mut f = File::open("cool.c").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s);
    let mut chars = s.chars().peekable();

    let mut state = LexerState::Start;
    let mut curToken = String::new();
    //let mut lineNo: i32 = 1;
    let mut useChar: bool;
    let mut commentDepth = 0;
    loop {
        useChar = true;
        match chars.peek() {
            Some(ch) => {
                match state {
                    LexerState::Start => {
                        match *ch {
                            'a' ... 'z' | 'A' ... 'Z' => {
                                state = LexerState::Identifier;
                                curToken.push(*ch);
                            },
                            '0' ... '9' => {
                                state = LexerState::Number;
                                curToken.push(*ch);
                            },
                            '\n' => {
                                //lineNo += 1;
                            },
                            ' ' | '\r' | '\t' => {
                                // Nothing
                            },
                            '<' => {
                                state = LexerState::LessThan;
                            },
                            '-' => {
                                state = LexerState::CommentOrMinus;
                            },
                            '(' => {
                                state = LexerState::CommentOrParens;
                            },
                            '"' => {
                                state = LexerState::String;
                            },
                            '{' => { println!("LBrace"); },
                            '}' => { println!("RBrace"); },
                            ')' => { println!("RParen"); },
                            ':' => { println!("Colon"); },
                            ';' => { println!("Semicolon"); },
                            '.' => { println!("Period"); },
                            ',' => { println!("Comma"); },
                            '@' => { println!("AtSign"); },
                            '+' => { println!("Plus"); },
                            '*' => { println!("Asterisk"); },
                            '/' => { println!("Slash"); },
                            '~' => { println!("Tilde"); },
                            '=' => { println!("Equals"); },
                            _ => {
                                println!("Unexpected character: {}", ch);
                            }
                        }
                    },
                    LexerState::Identifier => {
                        match *ch {
                            'a' ... 'z' | 'A' ... 'Z' | '0' ... '9' | '_' => {
                                curToken.push(*ch);
                            },
                            '\n' => {
                                //lineNo += 1;
                                state = LexerState::Start;
                            },
                            ' ' | '\r' | '\t' => {
                                state = LexerState::Start;
                            },
                            '<' => {
                                state = LexerState::Start;
                            },
                            '-' => {
                                state = LexerState::Start;
                            },
                            '(' => {
                                state = LexerState::Start;
                            },
                            '"' => {
                                state = LexerState::String;
                            },
                            '=' | '~' | '/' | '*' | '+' | '@' | ',' | '.' | ';' | ':' | ')' | '}' | '{' => {
                                state = LexerState::Start;
                            },
                            _ => {
                                println!("Unexpected character: {}", ch);
                            }
                        }
                        if state != LexerState::Identifier {
                            println!("Identifier: {}", curToken);
                            curToken = String::new();
                            useChar = false;
                        }
                    },
                    LexerState::Number => {
                        match *ch {
                            '0' ... '9' => {
                                curToken.push(*ch);
                            },
                            'a' ... 'z' | 'A' ... 'Z' | '_' => {
                                state = LexerState::Start;
                            }
                            '\n' => {
                                //lineNo += 1;
                                state = LexerState::Start;
                            },
                            ' ' | '\r' | '\t' => {
                                state = LexerState::Start;
                            },
                            '<' => {
                                state = LexerState::Start;
                            },
                            '-' => {
                                state = LexerState::Start;
                            },
                            '(' => {
                                state = LexerState::Start;
                            },
                            '"' => {
                                state = LexerState::Start;
                            },
                            '=' | '~' | '/' | '*' | '+' | '@' | ',' | '.' | ';' | ':' | ')' | '}' | '{' => {
                                state = LexerState::Start;
                            },
                            _ => {
                                println!("Unexpected character: {}", ch);
                            }
                        }
                        if state != LexerState::Number {
                            println!("Number: {}", curToken);
                            curToken = String::new();
                            useChar = false;
                        }
                    },
                    LexerState::LessThan => {
                        match *ch {
                            '-' => {
                                println!("Arrow");
                            },
                            '=' => {
                                println!("LessThanEqual");
                            },
                            '\n' => {
                                // lineNo += 1;
                                useChar = false;
                            },
                            _ => {
                                println!("LessThan");
                                useChar = false;
                            }
                        }
                        state = LexerState::Start;
                    },
                    LexerState::CommentOrMinus => {
                        match *ch {
                            '-' => {
                                state = LexerState::SingleLineComment;
                            },
                            _ => {
                                println!("Minus");
                                state = LexerState::Start;
                                useChar = false;
                            }
                        }
                    },
                    LexerState::SingleLineComment => {
                        match *ch {
                            '\n' => {
                                state = LexerState::Start;
                            },
                            _ => {
                            }
                        }
                    },
                    LexerState::CommentOrParens => {
                        match *ch {
                            '*' => {
                                commentDepth += 1;
                                state = LexerState::MultiLineComment;
                            },
                            _ => {
                                if commentDepth == 0 {
                                    println!("LParen");
                                    state = LexerState::Start;
                                    useChar = false;
                                } else {
                                    state = LexerState::MultiLineComment;
                                }
                            }
                        }
                    },
                    LexerState::MultiLineComment => {
                        match *ch {
                            '*' => {
                                state = LexerState::MultiLineCommentEnd;
                            },
                            _ => {
                            }
                        }
                    },
                    LexerState::MultiLineCommentEnd => {
                        match *ch {
                            ')' => {
                                commentDepth -= 1;
                                if commentDepth == 0 {
                                    println!("Comment");
                                    state = LexerState::Start;
                                } else {
                                    state = LexerState::MultiLineComment;
                                }
                            },
                            '*' => {
                                state = LexerState::MultiLineCommentEnd;
                            },
                            _ => {
                                state = LexerState::MultiLineComment;
                            }
                        }
                    },
                    LexerState::String => {
                        match *ch {
                            '\\' => {
                                state = LexerState::StringEscape;
                            },
                            '"' => {
                                println!("StringLiteral: {}", curToken);
                                curToken = String::new();
                                state = LexerState::Start;
                            },
                            '\n' => {
                                println!("Error: Newline in String Literal.");
                            },
                            _ => {
                                curToken.push(*ch);
                            }
                        }
                    },
                    LexerState::StringEscape => {
                        match *ch {
                            't' => {
                                curToken.push('\t');
                            },
                            'n' => {
                                curToken.push('\n');
                            },
                            '\n' => {
                            },
                            _ => {
                                curToken.push(*ch);
                            }
                        }
                        state = LexerState::String;
                    }
                }
            },
            None => {
                if state == LexerState::MultiLineComment {
                    println!("Error: EOF in comment.");
                } else if state == LexerState::String {
                    println!("Error: EOF in string literal.");
                }
                break;
            }
        }
        if useChar {
            let _ = chars.next();
        }
    }
}
