use core::panic;
use std::{collections::{HashMap}};
use crate::{NativeFn, ast::{Expr, LiteralValue, Stmt}, error_handler::ParseError, lexer, parser::Parser, tokens::{self, Token}};
use std::rc::Rc;
use std::fs;
use std::cell::RefCell;


#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Token)>,
    pub body: Vec<Stmt>,
}

#[derive(Clone)]
pub struct Interpreter {
    pub scopes: Vec<HashMap<String, Token>>,
    pub functions: Rc<RefCell<HashMap<String, Function>>>,
    pub native_fns: Rc<RefCell<HashMap<String, NativeFn>>>,
    pub struct_defs: HashMap<String, Vec<(String, Token)>>,
    pub impl_defs: HashMap<String, HashMap<String, Function>>,
    pub loaded_modules: HashMap<String, String>,
    pub namespaces: HashMap<String, HashMap<String, Function>>,
}

impl std::fmt::Debug for Interpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("scopes", &self.scopes)
            .field("functions", &self.functions)
            .field("native_fns", &"<native functions>") 
            .finish()
    }
}

impl PartialEq for Interpreter {
    fn eq(&self, other: &Self) -> bool {
        self.scopes == other.scopes &&
        self.functions == other.functions
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            scopes: vec![HashMap::new()],
            functions: Rc::new(RefCell::new(HashMap::new())),
            native_fns: Rc::new(RefCell::new(HashMap::new())),
            struct_defs: HashMap::new(),
            impl_defs: HashMap::new(),
            loaded_modules: HashMap::new(),
            namespaces: HashMap::new(),
        }
    }
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), ParseError> {
        for stmt in &statements {
            if let Stmt::Func { name, params, body } = stmt {
                let func = Function { 
                    name: name.clone(), 
                    params: params.clone(), 
                    body: body.clone() 
                };
                self.functions.borrow_mut().insert(name.clone(), func);
            }
        }
        for stmt in statements {
            match stmt {
                Stmt::Func { .. } => {} 
                
                _ => self.execute(stmt)?,
            }
        }
        Ok(())
    }

    pub fn execute(&mut self, stmt: Stmt) -> Result<(), ParseError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr);
                Ok(())
            }
            Stmt::Print(exprs) => {
                for (index, expr) in exprs.iter().enumerate() {
                    let value = self.evaluate(expr.clone());

                    self.print_token_value(&value)?;

                    if index < exprs.len() - 1 {
                        print!("");
                    }

                }
                println!();
                Ok(())
            }

            Stmt::Let { name, data_type, value } => {
                let declared_value = self.evaluate(value);
                if Self::check_type_compatibility(&data_type, &declared_value) {
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert(name, declared_value);
                    }
                    Ok(())
                } else {
                    Err(ParseError::TypeMismatch { 
                        expected: data_type, 
                        found: declared_value 
                    })
                }
            }

            Stmt::ListAssign { list_name, indices, value } => {
                let new_val = self.evaluate(value);

                let mut evaluated_indices = Vec::new();

                for expr in indices {
                    match self.evaluate(expr) {
                        Token::Integer(n) => {
                            if n < 0 { panic!("Runtime Error: Index cannot be negative!"); }
                            evaluated_indices.push(n as usize);
                        }

                        _ => panic!("Runtime Error: Index must be an Integer!"),
                    }
                }

                if evaluated_indices.is_empty() {
                    panic!("Runtime Error: No indices provided!");
                }

                let mut is_assigned = false;

                for scope in self.scopes.iter_mut().rev() {
                    if let Some(mut token) = scope.get_mut(&list_name) {
                        is_assigned = true;
                        let last_idx = evaluated_indices.pop().unwrap();
                        for &idx in &evaluated_indices {
                            match token {
                                Token::List(elements) => {
                                    if idx < elements.len() {
                                        
                                        token = &mut elements[idx];
                                    } else {
                                        panic!("Runtime Error: Index out of bounds!");
                                    }
                                }
                                _ => panic!("Runtime Error: Variable '{}' is not a multi-dimensional list!", list_name),
                            }
                        }

                        match token {
                            Token::List(elements) => {
                                if last_idx < elements.len() {
                                    
                                    elements[last_idx] = new_val.clone();
                                    break;
                                } else {
                                    panic!("Runtime Error: Index out of bounds!");
                                }
                            }
                            _ => panic!("Runtime Error: Target is not a list!"),
                        }
                    }
                }

                if !is_assigned {
                    panic!("Runtime Error: Undeclared list '{}'", list_name);
                }

                Ok(())
            }

            Stmt::Block(statements) => {
                self.scopes.push(HashMap::new());

                for stmt in statements {
                    if let Err(e) = self.execute(stmt) {
                        self.scopes.pop();
                        return Err(e);
                    }
                }
                self.scopes.pop();
                Ok(())
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let evaluated_cond = self.evaluate(condition);

                let is_true = match evaluated_cond {
                    Token::Boolean(b) => b,
                    _ => panic!("Runtime Error: 'if' statement needs to be boolean! Found: {:?}", evaluated_cond),
                };

                if is_true {
                    self.execute(*then_branch)?;
                } else if let Some(else_stmt) = else_branch {
                    self.execute(*else_stmt)?;
                }
                
                Ok(())
            }

            Stmt::While { condition, body } => {
                loop {
                    let evaluated_cond = self.evaluate(condition.clone());
                    let is_true = match evaluated_cond {
                        Token::Boolean(b) => b,
                        _ => panic!("Runtime Error: 'while' koşulu boolean olmalı! Bulunan: {:?}", evaluated_cond),
                    };
    
                    if is_true {
                        self.execute(*body.clone())?;
                    } else {
                        break;
                    }
                }
                Ok(())
            }

            Stmt::For { var_name, start_value, end_value, body } => {

                let start_token = self.evaluate(start_value);
                let end_token = self.evaluate(end_value);

                let mut current = match start_token {
                    Token::Integer(n) => n,
                    _ => panic!("Runtime Error: For loop start value must be an Integer!"),
                };

                let limit = match end_token {
                    Token::Integer(n) => n,
                    _ => panic!("Runtime Error: For loop end value must be an Integer!"),
                };
                
                self.scopes.push(HashMap::new());

                while current < limit {
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert(var_name.clone(), Token::Integer(current));
                    }

                    self.execute(*body.clone())?;

                    current += 1;
                }
                self.scopes.pop();
                Ok(())
            }

            Stmt::Func { name, params, body } => {
                let func = Function {name: name.clone(), params, body};
                self.functions.borrow_mut().insert(name, func);
                Ok(())
            }

            Stmt::Struct { name, body } => {
                let fields = body.iter().map(|stmt| {
                    if let Stmt::Let { name, data_type, .. } = stmt {
                        (name.clone(), Token::Unknown)
                    } else {
                        panic!("Struct body must only contain field declarations!");
                    }
                }).collect();
                self.struct_defs.insert(name, fields);
                Ok(())
            }

            Stmt::Impl { name, body } => {
                for stmt in body {
                    if let Stmt::Func { name: fn_name, params, body } = stmt {
                        self.impl_defs
                            .entry(name.clone())
                            .or_default()
                            .insert(fn_name, Function { params, body, name: name.clone() });
                    }
                }
                Ok(())
            }

            Stmt::Return { keyword, value } => {
                let return_val = match value {
                    Some(expr) => self.evaluate(expr),
                    None => Token::Unknown,
                };

                Err(ParseError::Return { value: return_val })
            }

            Stmt::Import { directory, identifier } => {
                if self.loaded_modules.contains_key(&directory) {
                    return Ok(());
                }
                if self.loaded_modules.contains_key(&identifier) {
                    panic!("Runtime error: you can't assign same name in imports");
                }
                self.loaded_modules.insert(directory.clone(), identifier.clone());

                let import_directory = match fs::read_to_string(&directory) {
                    Ok(c) => c,
                    Err(e) => panic!("Runtime Error: could not read file '{}': {}", directory, e),
                };

                let tokens = match lexer::lexer(&import_directory) {
                    Ok(t) => t,
                    Err(e) => panic!("Runtime Error: lexer failed in '{}': {}", directory, e),
                };

                let mut parser = Parser::new(tokens);
                let mut ast_tree: Vec<Stmt> = Vec::new();

                while parser.current_token().is_some() && *parser.current_token().unwrap() != Token::EOF {
                    match parser.parse_statement() {
                        Ok(stmt) => ast_tree.push(stmt),
                        Err(e) => panic!("Runtime Error: parser failed in '{}': {:?}", directory, e),
                    }
                }

                let mut sub = Interpreter::new();
                sub.interpret(ast_tree)?;

                let module_fns = sub.functions.borrow().clone();
                self.namespaces.insert(identifier.clone(), module_fns);

                for (name, fields) in sub.struct_defs {
                    self.struct_defs.insert(name, fields);
                }
                for (name, methods) in sub.impl_defs {
                    self.impl_defs.insert(name, methods);
                }

                Ok(())
            }

            _ => Ok(())
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Token {
        match expr {
            Expr::Literal(lit) => match lit {
                LiteralValue::Int(i) => Token::Integer(i),
                LiteralValue::Float(f) => Token::Float(f),
                LiteralValue::Str(s) => Token::String(s),
                LiteralValue::Bool(b) => Token::Boolean(b),
                LiteralValue::Nil => Token::Unknown,   
            }

            Expr::Variable(name) => {
                self.get_variable(&name).cloned().unwrap_or(Token::Unknown)
            }

            Expr::Binary { left, op, right } => {
                let left = self.evaluate(*left);
                let right = self.evaluate(*right);

                if op == Token::Equals {
                    return Token::Boolean(left == right);
                }
                if op == Token::NotEquals {
                    return Token::Boolean(left != right);
                }

                if let (Some(l_num), Some(r_num)) = (to_float(&left), to_float(&right)) {
                    match op {
                        Token::Greater => return Token::Boolean(l_num > r_num),
                        Token::Lesser => return Token::Boolean(l_num < r_num),
                        Token::GreaterEquals => return Token::Boolean(l_num >= r_num),
                        Token::LesserEquals => return Token::Boolean(l_num <= r_num),
                        _ => {} 
                    }
                }

                match (left, op, right) {
                    (Token::Integer(l), Token::Plus, Token::Integer(r)) => Token::Integer(l + r),
                    (Token::Float(l), Token::Plus, Token::Float(r)) => Token::Float(l + r),
                    (Token::Integer(l), Token::Minus, Token::Integer(r)) => Token::Integer(l - r),
                    (Token::Float(l), Token::Minus, Token::Float(r)) => Token::Float(l - r),
                    (Token::Integer(l), Token::Multiply, Token::Integer(r)) => Token::Integer(l * r),
                    (Token::Float(l), Token::Multiply, Token::Float(r)) => Token::Float(l * r),
                    (Token::Integer(l), Token::Divide, Token::Integer(r)) => Token::Integer(l / r),
                    (Token::Float(l), Token::Divide, Token::Float(r)) => Token::Float(l / r),
                    (Token::String(l), Token::Plus, Token::String(r)) => Token::String(format!("{}{}", l, r)),
                    _ => panic!("Runtime Error: Type mismatch"),
                }
            }

            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(*right);
                match (operator, right_val) {
                    (Token::Minus, Token::Integer(n)) => Token::Integer(-n),
                    (Token::Minus, Token::Float(n)) => Token::Float(-n),
                    (Token::Bang, Token::Boolean(b)) => Token::Boolean(!b),
                    (op, val) => panic!("Runtime Error: {:?} operator cannot used with {:?} .", op, val),
                }
            }

            Expr::Logical { left, operator, right } => {
                let left = self.evaluate(*left);

                if operator == Token::Or {
                    if let Token::Boolean(b) = left {
                        if b {
                            return Token::Boolean(true);
                        }
                    } else {
                        panic!("Runtime Error: 'or' operator's left needs to be Boolean!");
                    }
                }
                else if operator == Token::And {
                    if let Token::Boolean(b) = left {
                        if !b { 
                            return Token::Boolean(false);
                        }
                    } else {
                        panic!("Runtime Error: 'and' operator's left needs to be Boolean!");
                    }
                }
                let right = self.evaluate(*right);

                if let Token::Boolean(b) = right {
                    return Token::Boolean(b);
                } else {
                    panic!("Runtime Error: 'and'/'or' operator's right needs to be Boolean!");
                }
            }

            Expr::Assign { name, value } => {
                let new_value = self.evaluate(*value);

                for scope in self.scopes.iter_mut().rev() {
                    if let Some(old_value) = scope.get_mut(&name) {
                        if Self::check_type_compatibility(old_value, &new_value) {
                            *old_value = new_value.clone();
                            return new_value;
                        } else {
                            panic!("Runtime Error: Type mismatch! Variable '{}' is {:?} but you tried to assign {:?}", name, old_value, new_value);
                        }
                    }
                }

                panic!("Runtime Error: Variable '{}' not declared.", name);
            }

            Expr::Index { list, index } => {
                let list_val = self.evaluate(*list);
                let index_val = self.evaluate(*index);
    
                if let (Token::List(elements), Token::Integer(idx)) = (list_val, index_val) {
                    if idx < 0 {
                        panic!("Runtime Error: Index cannot be negative! Found: {}", idx);
                    }
                    let i = idx as usize;
                    if i < elements.len() {
                        return elements[i].clone();
                    } else {
                        panic!("Runtime Error: Index out of bounds! Len: {}, Index: {}", elements.len(), idx);
                    }
                } else {
                    panic!("Runtime Error: Type mismatch. Expected List and Integer index.");
                } 
            }

            Expr::List(elements) => {
                let mut evaluated_list = Vec::new();
                for expr in elements {
                    let value = self.evaluate(expr);
                    evaluated_list.push(value);
                }
                Token::List(evaluated_list)
            }

            Expr::Call { callee, paren, arguments } => {
                let name = match *callee {
                    Expr::Variable(ref n) => n.clone(),
                    _ => panic!("Runtime Error: Callee must be a named function!"),
                };

                let evaluated_args: Vec<Token> = arguments
                    .into_iter()
                    .map(|arg| self.evaluate(arg))
                    .collect();

                // --- Struct constructor: Point(10, 20) ---
                if let Some(fields) = self.struct_defs.get(&name).cloned() {
                    let instance_fields = fields.iter().zip(evaluated_args)
                        .map(|((field_name, _), val)| (field_name.clone(), val))
                        .collect();
                    return Token::StructInstance { type_name: name, fields: instance_fields };
                }
                
                if let Some(result) = crate::native_functions::dispatch(&name, evaluated_args.clone()) {
                    return result.unwrap_or(Token::Unknown);
                }

                if let Some(func) = self.native_fns.borrow().get(&name).cloned() {
                    return func(evaluated_args);
                }

                let func = self.functions.borrow().get(&name).cloned();
                let func = match func {
                    Some(f) => f,
                    None => panic!("Runtime Error: Undefined function '{}'", name),
                };

                if func.params.len() != evaluated_args.len() {
                    panic!(
                        "Runtime Error: Function '{}' expects {} args but got {}",
                        name, func.params.len(), evaluated_args.len()
                    );
                }

                let mut call_scope = HashMap::new();
                for ((param_name, _param_type), arg_val) in func.params.iter().zip(evaluated_args) {
                    call_scope.insert(param_name.clone(), arg_val);
                }
                self.scopes.push(call_scope);

                let mut return_value = Token::Unknown;
                for stmt in func.body {
                    match self.execute(stmt) {
                        Ok(_) => {}
                        Err(ParseError::Return { value }) => {
                            return_value = value;
                            break;
                        }
                        Err(_) => {
                            self.scopes.pop();
                            return Token::Unknown;
                        }
                    }
                }
                self.scopes.pop();
                return_value
            }

            Expr::MethodCall { object, method, args } => {
                let obj_name = match *object {
                    Expr::Variable(ref name) => name.clone(),
                    _ => panic!("Runtime Error: invalid method call target"),
                };

                // --- Namespace call: math.add(1, 2) ---
                if let Some(module_fns) = self.namespaces.get(&obj_name).cloned() {
                    let func = match module_fns.get(&method) {
                        Some(f) => f.clone(),
                        None => panic!("Runtime Error: module '{}' has no function '{}'", obj_name, method),
                    };

                    let evaluated_args: Vec<Token> = args.into_iter()
                        .map(|a| self.evaluate(a))
                        .collect();

                    if func.params.len() != evaluated_args.len() {
                        panic!(
                            "Runtime Error: '{}' expects {} args but got {}",
                            method, func.params.len(), evaluated_args.len()
                        );
                    }

                    let mut call_scope = HashMap::new();
                    for ((param_name, _param_type), arg_val) in func.params.iter().zip(evaluated_args) {
                        call_scope.insert(param_name.clone(), arg_val);
                    }
                    self.scopes.push(call_scope);

                    let mut return_value = Token::Unknown;
                    for stmt in func.body {
                        match self.execute(stmt) {
                            Ok(_) => {}
                            Err(ParseError::Return { value }) => {
                                return_value = value;
                                break;
                            }
                            Err(_) => {
                                self.scopes.pop();
                                return Token::Unknown;
                            }
                        }
                    }
                    self.scopes.pop();
                    return return_value;
                }

                // --- Struct method call: p.get_x() ---
                let instance = self.scopes.iter()
                    .rev()
                    .find_map(|scope| scope.get(&obj_name).cloned());

                if let Some(Token::StructInstance { ref type_name, .. }) = instance {
                    let func = self.impl_defs
                        .get(type_name)
                        .and_then(|methods| methods.get(&method))
                        .cloned()
                        .unwrap_or_else(|| panic!(
                            "Runtime Error: struct '{}' has no method '{}'",
                            type_name, method
                        ));

                    let evaluated_args: Vec<Token> = args.into_iter()
                        .map(|a| self.evaluate(a))
                        .collect();

                    if func.params.len() != evaluated_args.len() {
                        panic!(
                            "Runtime Error: '{}' expects {} args but got {}",
                            method, func.params.len(), evaluated_args.len()
                        );
                    }

                    // Inject `self` + args into the call scope
                    let mut call_scope = HashMap::new();
                    call_scope.insert("self".to_string(), instance.unwrap());
                    for ((param_name, _), arg_val) in func.params.iter().zip(evaluated_args) {
                        call_scope.insert(param_name.clone(), arg_val);
                    }
                    self.scopes.push(call_scope);

                    let mut return_value = Token::Unknown;
                    for stmt in func.body {
                        match self.execute(stmt) {
                            Ok(_) => {}
                            Err(ParseError::Return { value }) => {
                                return_value = value;
                                break;
                            }
                            Err(_) => {
                                self.scopes.pop();
                                return Token::Unknown;
                            }
                        }
                    }
                    self.scopes.pop();
                    return return_value;
                }

                // --- List method call: mylist.push(x), mylist.pop(), mylist.len() ---
                let evaluated_args: Vec<Token> = args.into_iter()
                    .map(|a| self.evaluate(a))
                    .collect();

                let list = self.scopes.iter()
                    .rev()
                    .find_map(|scope| scope.get(&obj_name).cloned());

                match list {
                    Some(Token::List(mut elements)) => {
                        match method.as_str() {
                            "push" => {
                                let val = evaluated_args.into_iter().next()
                                    .unwrap_or(Token::Unknown);
                                elements.push(val);
                                for scope in self.scopes.iter_mut().rev() {
                                    if scope.contains_key(&obj_name) {
                                        scope.insert(obj_name, Token::List(elements));
                                        break;
                                    }
                                }
                                Token::Unknown
                            }
                            "pop" => {
                                let popped = elements.pop().unwrap_or(Token::Unknown);
                                for scope in self.scopes.iter_mut().rev() {
                                    if scope.contains_key(&obj_name) {
                                        scope.insert(obj_name, Token::List(elements));
                                        break;
                                    }
                                }
                                popped
                            }
                            "len" => {
                                Token::Integer(elements.len() as i64)
                            }
                            _ => panic!("Runtime Error: unknown list method '{}'", method),
                        }
                    }
                    Some(_) => panic!(
                        "Runtime Error: '{}' is not a list, cannot call method '{}'",
                        obj_name, method
                    ),
                    None => panic!("Runtime Error: undefined variable '{}'", obj_name),
                }
            }

            Expr::FieldGet { object, field } => {
                let obj = self.evaluate(*object);
                if let Token::StructInstance { fields, .. } = obj {
                    fields.into_iter()
                        .find(|(name, _)| name == &field)
                        .map(|(_, val)| val)
                        .unwrap_or(Token::Unknown)
                } else {
                    panic!("Runtime Error: field access on non-struct value")
                }
            }

            _ => Token::Unknown,
        }
    }

    fn get_variable(&self, name: &str) -> Option<&Token> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

    fn print_token_value(&self, token: &Token) -> Result<(), ParseError> {
        match token {
            Token::String(s) => print!("{} ", s),
            Token::Integer(n) => print!("{} ", n),
            Token::Float(f) => print!("{} ", f),
            Token::Boolean(b) => print!("{} ", b),
            Token::List(elements) => {
                for (i, element) in elements.iter().enumerate() {
                    self.print_token_value(element)?;
                    if i < elements.len() - 1 {
                        print!(", ");
                    }
                }
            },
            Token::Identifier(name) => {
                if let Some(value_token) = self.get_variable(name) {
                    self.print_token_value(value_token)?;
                } else {
                    return Err(ParseError::UndeclaredVariable { name: name.clone() });
                }
            }
            Token::StructInstance { type_name, fields } => {
                print!("{} {{ ", type_name);
                for (i, (field_name, field_val)) in fields.iter().enumerate() {
                    print!("{}: ", field_name);
                    self.print_token_value(field_val)?;
                    if i < fields.len() - 1 { print!(", "); }
                }
                print!("}}");
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: Token::String("a printable value".to_string()),
                found: Some(token.clone()),
            })
        }
        Ok(())
    }

    fn check_type_compatibility(expected_type: &Token, actual_value: &Token) -> bool {
        match (expected_type, actual_value) {
            (Token::TypeInt, Token::Integer(_)) => true,
            (Token::TypeFloat, Token::Float(_)) => true,
            (Token::TypeString, Token::String(_)) => true,
            (Token::TypeBool, Token::Boolean(_)) => true,
            (Token::TypeList(_), Token::List(_)) => true,

            (Token::Integer(_), Token::Integer(_)) => true,
            (Token::Float(_), Token::Float(_)) => true,
            (Token::String(_), Token::String(_)) => true,
            (Token::Boolean(_), Token::Boolean(_)) => true,
            (Token::List(_), Token::List(_)) => true,

            (Token::Identifier(type_name), Token::StructInstance { type_name: instance_type, .. }) => {
                type_name == instance_type
            }

            _ => false
        }        
    }
}

fn to_float(token: &Token) -> Option<f64> {
    match token {
        Token::Integer(n) => Some(*n as f64),
        Token::Float(f) => Some(*f),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::WolfEngine;

    #[test]
    fn test_struct() {
        let mut engine = WolfEngine::new();
        engine.run(r#"
            struct Point
                x: int
                y: int
            end

            let p: Point = Point(10, 20)
            print p
        "#).unwrap();
    }

    #[test]
    fn test_impl() {
        let mut engine = WolfEngine::new();
        engine.run(r#"
            struct Point
                x: int
                y: int
            end

            impl Point
                fn get_x()
                    return self.x
                end
            end

            let p: Point = Point(10, 20)
            let result: int = p.get_x()
            print result
        "#).unwrap();
    }
}