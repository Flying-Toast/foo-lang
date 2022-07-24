mod lex;
mod ast;
mod interp;

fn main() {
    let tokens = lex::lex_tokens(include_str!("../example.foo"));
    for i in ast::parse_items(tokens) {
        println!("{i:#?}");
    }
}
