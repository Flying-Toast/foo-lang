#[derive(Debug)]
pub enum Token<'a> {
    Begin,
    LeftBrace,
    RightBrace,
    Var,
    Ident(&'a str),
    Equals,
    Integer(&'a str),
    LeftParen,
    RightParen,
    Plus,
    Semicolon,
}

pub fn lex_tokens(src: &str) -> impl Iterator<Item=Token> {
    TokenStream {
        src,
        idx: 0,
    }
}

struct TokenStream<'a> {
    src: &'a str,
    idx: usize,
}

impl<'a> TokenStream<'a> {
    fn tail(&self) -> &'a str {
        &self.src[self.idx..]
    }

    fn eof(&self) -> bool {
        self.tail().is_empty()
    }

    fn peek(&self) -> Option<char> {
        self.tail().chars().next()
    }

    fn eat_while<P: Fn(char) -> bool>(&mut self, predicate: P) -> Option<&'a str> {
        let nmatching = self.tail().chars().take_while(|&c| predicate(c)).count();

        if nmatching == 0 {
            None
        } else {
            let s = Some(&self.tail()[0..nmatching]);
            self.idx += nmatching;
            s
        }
    }

    fn consume_whitespace(&mut self) {
        self.eat_while(|ch| ch.is_ascii_whitespace());
    }

    fn lex_bareword(&mut self) -> Option<Token<'a>> {
        let word = self.eat_while(char::is_alphabetic)?;

        Some(
            match word {
                "begin" => Token::Begin,
                "var" => Token::Var,
                _ => Token::Ident(word),
            }
        )
    }

    fn lex_integer(&mut self) -> Option<Token<'a>> {
        self.eat_while(|ch| ch.is_ascii_digit()).map(Token::Integer)
    }

    fn lex_onechar_symbol(&mut self) -> Option<Token<'a>> {
        let tkn = match self.peek()? {
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '=' => Token::Equals,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '+' => Token::Plus,
            ';' => Token::Semicolon,
            _ => return None,
        };

        self.idx += 1;
        Some(tkn)
    }

}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();

        let lexers = [
            TokenStream::lex_bareword,
            TokenStream::lex_onechar_symbol,
            TokenStream::lex_integer,
        ];

        if self.eof() {
            return None;
        }

        for f in lexers {
            if let Some(token) = f(self) {
                return Some(token);
            }
        }

        panic!("Lexing error at idx {}", self.idx);
    }
}
