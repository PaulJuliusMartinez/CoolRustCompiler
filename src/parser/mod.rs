use lexer::Token;
use ast;

macro_rules! match_single {
    ($tok:expr, $expected:pat, $state:ident, $new_state:expr) => {{
        match $tok {
            $expected => {
                $state = $new_state;
            },
            _ => {
                println!("Errored in state {}", $state);
            }
        }
    }}
}

pub fn parse_cool_program(tokens: &Vec<Token>) -> Option<ast::Program> {
    let mut classLists: Vec<ast::Class> = Vec::new();
    let mut lastClass: Option<ast::Class> = None;
    let mut types: Vec<ast::CoolType> = Vec::new();
    let mut features: Vec<ast::Feature> = Vec::new();
    let mut formals: Vec<ast::Formal> = Vec::new();
    let mut identifiers: Vec<ast::Symbol> = Vec::new();
    let mut expressions: Vec<ast::Expression> = Vec::new();
    let mut expressionLists: Vec<Vec<ast::Expression>> = Vec::new();
    let mut caseBranches: Vec<ast::CaseBranch> = Vec::new();
    let mut state = 0;

    let x = Token::If;
    let y = Token::While;

    for tok in tokens {
        match state {
            _ => { println!("Haven't implemented state {} yet", state); }
        }
    }

    return None;
}
