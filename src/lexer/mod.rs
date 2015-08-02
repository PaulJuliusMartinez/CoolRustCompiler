use std::iter::Peekable;
use std::rc::Rc;
use std::ascii::AsciiExt;
use std::str::Chars;

#[derive(Debug)]
pub enum Token {
    Type (Rc<String>),
    Identifier (Rc<String>),
    IntegerLiteral (i32),
    StringLiteral (Rc<String>),
    Case,
    Class,
    Else,
    Esac,
    False,
    Fi,
    If,
    In,
    Inherits,
    Isvoid,
    Let,
    Loop,
    New,
    Not,
    Of,
    Pool,
    Then,
    True,
    While,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Colon,
    Semicolon,
    Dot,
    Comma,
    At,
    Plus,
    Minus,
    Times,
    Divide,
    Tilde,
    LessThan,
    Equal,
    LessThanEqual,
    Arrow,
    EOF
}

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

pub fn lex(mut chars: Peekable<Chars>) -> Vec<Token> {
    let mut state = LexerState::Start;
    let mut curToken = String::with_capacity(1024);
    let mut line_no = 0;
    let mut useChar;
    let mut commentDepth = 0;
    let mut tokens: Vec<Token> = Vec::new();

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
                                line_no += 1;
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
                            '{' => { tokens.push(Token::LeftBrace); },
                            '}' => { tokens.push(Token::RightBrace); },
                            ')' => { tokens.push(Token::RightParen); },
                            ':' => { tokens.push(Token::Colon); },
                            ';' => { tokens.push(Token::Semicolon); },
                            '.' => { tokens.push(Token::Dot); },
                            ',' => { tokens.push(Token::Comma); },
                            '@' => { tokens.push(Token::At); },
                            '+' => { tokens.push(Token::Plus); },
                            '*' => { tokens.push(Token::Times); },
                            '/' => { tokens.push(Token::Divide); },
                            '~' => { tokens.push(Token::Tilde); },
                            '=' => { tokens.push(Token::Equal); },
                            _ => {
                                println!("Unexpected character: {}", ch);
                                return tokens;
                            }
                        }
                    },
                    LexerState::Identifier => {
                        match *ch {
                            'a' ... 'z' | 'A' ... 'Z' | '0' ... '9' | '_' => {
                                curToken.push(*ch);
                            },
                            '\n' => {
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
                                return tokens;
                            }
                        }
                        if state != LexerState::Identifier {
                            tokens.push(stringToToken(curToken));
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
                            let value = curToken.parse::<i32>();
                            match value {
                                Ok(val) => {
                                    tokens.push(Token::IntegerLiteral(val));
                                },
                                Err(err) => {
                                    println!("{}", err);
                                    return tokens;
                                }
                            }
                            curToken = String::new();
                            useChar = false;
                        }
                    },
                    LexerState::LessThan => {
                        match *ch {
                            '-' => {
                                tokens.push(Token::Arrow);
                            },
                            '=' => {
                                tokens.push(Token::LessThanEqual);
                            },
                            '\n' => {
                                line_no += 1;
                            },
                            _ => {
                                tokens.push(Token::LessThan);
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
                                tokens.push(Token::Minus);
                                state = LexerState::Start;
                                useChar = false;
                            }
                        }
                    },
                    LexerState::SingleLineComment => {
                        match *ch {
                            '\n' => {
                                line_no += 1;
                                state = LexerState::Start;
                            },
                            _ => {
                                // Ignore
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
                                    tokens.push(Token::LeftParen);
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
                                tokens.push(Token::StringLiteral(Rc::new(curToken)));
                                curToken = String::new();
                                state = LexerState::Start;
                            },
                            '\n' => {
                                println!("Error: newline in string literal.");
                                return tokens;
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
    tokens.push(Token::EOF);
    return tokens;
}

fn stringToToken(chars: String) -> Token {
    let lowercase = chars.chars().nth(0).unwrap().is_lowercase();
    let copy = chars.clone();
    match chars.as_ref() {
        "case"     => Token::Case,
        "class"    => Token::Class,
        "else"     => Token::Else,
        "esac"     => Token::Esac,
        "fi"       => Token::Fi,
        "if"       => Token::If,
        "in"       => Token::In,
        "inherits" => Token::Inherits,
        "isvoid"   => Token::Isvoid,
        "let"      => Token::Let,
        "loop"     => Token::Loop,
        "new"      => Token::New,
        "not"      => Token::Not,
        "of"       => Token::Of,
        "pool"     => Token::Pool,
        "then"     => Token::Then,
        "while"    => Token::While,
        "true"     => {
            if lowercase {
                Token::True
            } else {
                Token::Type(Rc::new(copy))
            }
        },
        "false" => {
            if lowercase {
                Token::False
            } else {
                Token::Type(Rc::new(copy))
            }
        },
        other => {
            if lowercase {
                Token::Identifier(Rc::new(copy))
            } else {
                Token::Type(Rc::new(copy))
            }
        }
    }
}
