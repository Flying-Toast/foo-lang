use crate::ast::{Expr, Statement, Item};
use std::collections::HashMap;

#[derive(Debug)]
struct Context<'a> {
    vars: HashMap<&'a str, Value>,
}

impl<'a> Context<'a> {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    fn get_var_mut(&mut self, varname: &str) -> Option<&mut Value> {
        self.vars.get_mut(varname)
    }

    fn get_var(&self, varname: &str) -> Option<&Value> {
        self.vars.get(varname)
    }

    fn create_var(&mut self, varname: &'a str, val: Value) {
        self.vars.insert(varname, val);
    }

    fn has_var(&self, varname: &str) -> bool {
        self.get_var(varname).is_some()
    }

    fn reduce_expr(&self, expr: &Expr) -> Value {
        match expr {
            Expr::IntLit { value } => Value::Int(*value),
            // TODO: remove need for the clone:
            Expr::VarRef { variable } => self.get_var(&variable).expect(&format!("No variable {variable}")).clone(),
            Expr::Add { lhs, rhs } => {
                match (self.reduce_expr(lhs), self.reduce_expr(rhs)) {
                    (Value::Int(l), Value::Int(r)) => Value::Int(l + r),
                    _ => panic!("Can't add non-ints"),
                }
            },
        }
    }

    fn eval(&mut self, stmt: &'a Statement) {
        match stmt {
            Statement::VarDeclaration { variable, value } => {
                if self.has_var(variable) {
                    panic!("Redeclaration of variable {variable}");
                }
                self.create_var(variable, self.reduce_expr(value));
            },
            Statement::Assignment { variable, value } => {
                if !self.has_var(&variable) {
                    panic!("Variable {variable} is not defined");
                }

                *self.get_var_mut(&variable).unwrap() = self.reduce_expr(value);
            },
        }
    }
}

#[derive(Debug)]
pub struct Program<'a> {
    begin_block_ctx: Context<'a>,
    begin_body: Vec<Statement<'a>>,
}

impl<'a> Program<'a> {
    pub fn from_items(items: impl Iterator<Item=Item<'a>>) -> Self {
        let mut begin_body = None;

        for i in items {
            match i {
                Item::EntryBlock { body } => {
                    assert!(begin_body.is_none(), "Multiple begin blocks not allowed");
                    begin_body = Some(body);
                },
            }
        }

        Self {
            begin_block_ctx: Context::new(),
            begin_body: begin_body.unwrap(),
        }
    }

    pub fn execute(&'a mut self) {
        for stmt in self.begin_body.iter() {
            self.begin_block_ctx.eval(stmt);
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    Int(u32),
}
