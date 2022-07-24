mod lex;
mod ast;
mod interp;

fn main() {
    let tokens = lex::lex_tokens(include_str!("../example.foo"));
    let items = ast::parse_items(tokens);
    interp::Program::from_items(items).execute();
}
