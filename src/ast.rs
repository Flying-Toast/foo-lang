use crate::lex::Token;

pub fn parse_items<'a>(tokens: impl Iterator<Item=Token<'a>>) -> impl Iterator<Item=Item<'a>> {
    ItemStream { tokens: tokens.peekable(), }
}

struct ItemStream<'a, T: Iterator<Item=Token<'a>>> {
    tokens: std::iter::Peekable<T>,
}

impl<'a, T: Iterator<Item=Token<'a>>> ItemStream<'a, T> {
    /// Parse the parenthesized args of a function call
    fn parse_call(&mut self) -> Vec<Expr<'a>> {
        assert!(matches!(self.tokens.next(), Some(Token::LeftParen)));

        if let Some(Token::RightParen) = self.tokens.peek() {
            self.tokens.next();
            // empty params list
            Vec::new()
        } else {
            let mut args = Vec::new();
            loop {
                args.push(self.parse_expr());
                match self.tokens.next() {
                    Some(Token::Comma) => {},
                    Some(Token::RightParen) => break,
                    other => panic!("Expected ',' or ')' but found: {other:#?}"),
                }
            }
            args
        }
    }

    fn parse_expr(&mut self) -> Expr<'a> {
        let lexpr = match self.tokens.next().unwrap() {
            Token::Ident(ident) => {
                if matches!(self.tokens.peek(), Some(Token::LeftParen)) {
                    Expr::FuncCall {
                        func_name: ident,
                        args: self.parse_call(),
                    }
                } else {
                    Expr::VarRef { variable: ident }
                }
            },
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
            Token::Return => {
                // eat Return
                self.tokens.next();
                Some(
                    Statement::Return { value: self.parse_expr() },
                )
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
            Token::Func => {
                let funcname = match self.tokens.next() {
                    Some(Token::Ident(ident)) => ident,
                    other => panic!("Unexpected token: {other:?}"),
                };

                assert!(matches!(self.tokens.next(), Some(Token::LeftParen)), "expected '('");
                let mut arg_names = Vec::new();
                loop {
                    match self.tokens.next().unwrap() {
                        Token::Ident(ident) => {
                            arg_names.push(ident);
                            if let Token::Comma = self.tokens.peek().unwrap() {
                                self.tokens.next();
                                assert!(matches!(self.tokens.peek(), Some(Token::Ident(_))), "expected identifier");
                            }
                        },
                        Token::RightParen => break,
                        other => panic!("unexpected token: {other:?}"),
                    }
                }
                assert!(matches!(self.tokens.next(), Some(Token::LeftBrace)), "Expected '{{'");
                let mut body = Vec::new();
                while let Some(stmt) = self.maybe_parse_statement() {
                    body.push(stmt);
                }
                match self.tokens.next() {
                    Some(Token::RightBrace) => {},
                    other => panic!("Expected '}}', found: {other:?}"),
                }

                Item::FuncDef {
                    name: funcname,
                    arg_names,
                    body,
                }
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

    FuncCall {
        func_name: &'a str,
        args: Vec<Expr<'a>>,
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

    Return {
        value: Expr<'a>,
    },
}

/// A top-level thing
#[derive(Debug)]
pub enum Item<'a> {
    EntryBlock {
        body: Vec<Statement<'a>>,
    },

    FuncDef {
        name: &'a str,
        arg_names: Vec<&'a str>,
        body: Vec<Statement<'a>>,
    },
}
