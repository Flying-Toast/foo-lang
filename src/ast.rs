use crate::lex::Token;

pub fn parse_items<'a>(tokens: impl Iterator<Item=Token<'a>>) -> impl Iterator<Item=Item<'a>> {
    ItemStream { tokens: tokens.peekable(), }
}

struct ItemStream<'a, T: Iterator<Item=Token<'a>>> {
    tokens: std::iter::Peekable<T>,
}

impl<'a, T: Iterator<Item=Token<'a>>> ItemStream<'a, T> {
    fn parse_expr(&mut self) -> Expr<'a> {
        let lexpr = match self.tokens.next().unwrap() {
            Token::Ident(ident) => Expr::VarRef { variable: ident },
            Token::Integer(i) => Expr::IntLit { value: i.parse().unwrap() },
            _ => panic!(),
        };

        match self.tokens.peek() {
            Some(Token::Plus) => {
                // eat the Token::Plus:
                self.tokens.next();
                Expr::Add { lhs: Box::new(lexpr), rhs: Box::new(self.parse_expr()) }
            },
            _ => lexpr,
        }
    }

    fn maybe_parse_statement(&mut self) -> Option<Statement<'a>> {
        let stmt = match self.tokens.peek()? {
            Token::Var => {
                // eat Token::Var:
                self.tokens.next();

                let varname = match self.tokens.next() {
                    Some(Token::Ident(ident)) => ident,
                   _ => panic!(),
                };

                assert!(matches!(self.tokens.next(), Some(Token::Equals)));

                Some(Statement::VarDeclaration {
                    variable: varname,
                    value: self.parse_expr(),
                })
            },
            Token::Ident(_) => {
                // eat Token::Ident:
                let ident = match self.tokens.next() {
                    Some(Token::Ident(i)) => i,
                    _ => panic!(),
                };

                assert!(matches!(self.tokens.next(), Some(Token::Equals)));

                Some(Statement::Assignment { variable: ident, value: self.parse_expr(), })
            },
            _ => return None,
        };

        assert!(matches!(self.tokens.next(), Some(Token::Semicolon)));
        stmt
    }

    fn parse_block_as_stmt_list(&mut self) -> Vec<Statement<'a>> {
        assert!(matches!(self.tokens.next(), Some(Token::LeftBrace)), "expected '{{'");

        let mut stmts = Vec::new();
        while let Some(stmt) = self.maybe_parse_statement() {
            stmts.push(stmt);
        }

        assert!(matches!(self.tokens.next(), Some(Token::RightBrace)), "expected '}}'");

        stmts
    }
}

impl<'a, T: Iterator<Item=Token<'a>>> Iterator for ItemStream<'a, T> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.tokens.next()? {
            Token::Begin => Item::EntryBlock {
                body: self.parse_block_as_stmt_list(),
            },
            tkn => panic!("Unexpected token {tkn:?}"),
        };

        Some(item)
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
    IntLit {
        value: u32,
    },

    VarRef {
        variable: &'a str,
    },

    Add {
        lhs: Box<Expr<'a>>,
        rhs: Box<Expr<'a>>,
    },
}

#[derive(Debug)]
pub enum Statement<'a> {
    VarDeclaration {
        variable: &'a str,
        value: Expr<'a>,
    },

    Assignment {
        variable: &'a str,
        value: Expr<'a>,
    },
}

/// A top-level thing
#[derive(Debug)]
pub enum Item<'a> {
    EntryBlock {
        body: Vec<Statement<'a>>,
    },
}
