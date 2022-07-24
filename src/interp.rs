use crate::ast::{Expr, Statement, Item};
use std::collections::HashMap;

#[derive(Debug)]
struct Context<'a> {
    vars: HashMap<&'a str, Value>,
    func_ret: Option<Value>,
    global_context: &'a GlobalContext<'a>,
}

impl<'a> Context<'a> {
    fn new(global_context: &'a GlobalContext<'a>) -> Self {
        Self {
            vars: HashMap::new(),
            func_ret: None,
            global_context,
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
            Expr::FuncCall { func_name, args } => {
                self.global_context
                    .call_func(func_name, args.iter().map(|i| self.reduce_expr(i)))
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
            Statement::Return { value } => {
                assert!(self.func_ret.is_none(), "function already returned a value");
                self.func_ret = Some(self.reduce_expr(value));
            },
        }
    }
}

#[derive(Debug)]
struct Function<'a> {
    arg_names: Vec<&'a str>,
    body: Vec<Statement<'a>>,
}

impl<'a> Function<'a> {
    fn new(arg_names: Vec<&'a str>, body: Vec<Statement<'a>>) -> Self {
        Self { arg_names, body, }
    }

    fn call(&self, args: impl ExactSizeIterator<Item=Value>, global_ctx: &'a GlobalContext<'a>) -> Value {
        assert!(self.arg_names.len() == args.len());
        let mut ctx = Context::new(global_ctx);
        for (name, argval) in self.arg_names.iter().zip(args) {
            ctx.create_var(name, argval);
        }

        for stmt in self.body.iter() {
            ctx.eval(stmt);
        }

        ctx.func_ret.expect("Function did not return a value")
    }
}

#[derive(Debug)]
struct GlobalContext<'a> {
    functions: HashMap<&'a str, Function<'a>>,
}

impl<'a> GlobalContext<'a> {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    fn call_func(&'a self, func_name: &'a str, args: impl ExactSizeIterator<Item=Value>) -> Value {
        self.functions
            .get(func_name)
            .expect(&format!("no func {func_name} is defined"))
            .call(args, self)
    }

    fn add_func(&mut self, func_name: &'a str, func: Function<'a>) {
        assert!(!self.functions.contains_key(func_name), "func {func_name} already defined");
        self.functions.insert(func_name, func);
    }
}

#[derive(Debug)]
pub struct Program<'a> {
    begin_body: Vec<Statement<'a>>,
    global: GlobalContext<'a>,
}

impl<'a> Program<'a> {
    pub fn from_items(items: impl Iterator<Item=Item<'a>>) -> Self {
        let mut begin_body = None;

        let mut global = GlobalContext::new();

        for i in items {
            match i {
                Item::EntryBlock { body } => {
                    assert!(begin_body.is_none(), "Multiple begin blocks not allowed");
                    begin_body = Some(body);
                },
                Item::FuncDef { name, arg_names, body } => {
                    global.add_func(name, Function::new(arg_names, body));
                },
            }
        }

        Self {
            begin_body: begin_body.unwrap(),
            global,
        }
    }

    pub fn execute(&'a mut self) {
        let mut ctx = Context::new(&self.global);
        for stmt in self.begin_body.iter() {
            if matches!(stmt, Statement::Return { .. }) {
                panic!("Can't return from begin block");
            }
            ctx.eval(stmt);
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    Int(u32),
}
