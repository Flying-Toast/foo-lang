mod lexer;

fn main() {
    for t in lexer::lex_tokens(include_str!("../example.foo")) {
        println!("{:?}", t);
    }
}
