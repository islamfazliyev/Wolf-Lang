use std::{collections::{HashMap}};
use crate::{NativeFn, ast::{Expr, LiteralValue, Stmt, StmtNode}, error_handler::ParseError, lexer, parser::Parser, tokens::{self, Token}};
use std::rc::Rc;
use std::fs;
use std::cell::RefCell;


#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Token)>,
    pub body: Vec<StmtNode>,
}

#[derive(Clone)]
pub struct Interpreter {
    pub scopes: Vec<HashMap<String, Rc<RefCell<Token>>>>,
    pub functions: Rc<RefCell<HashMap<String, Function>>>,
    pub native_fns: Rc<RefCell<HashMap<String, NativeFn>>>,
    pub struct_defs: HashMap<String, Vec<(String, Token)>>,
    pub impl_defs: HashMap<String, HashMap<String, Function>>,
    pub loaded_modules: HashMap<String, String>,
    pub module_globals: HashMap<String, HashMap<String, Rc<RefCell<Token>>>>,
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
            module_globals: HashMap::new(),
            namespaces: HashMap::new(),
        }
    }
    
    pub fn interpret(&mut self, statements: Vec<StmtNode>) -> Result<(), ParseError> {
        for node in &statements {
            if let Stmt::Func { name, params, body } = &node.stmt { // .stmt eklendi
                let func = Function { 
                    name: name.clone(), 
                    params: params.clone(), 
                    body: body.clone() 
                };
                self.functions.borrow_mut().insert(name.clone(), func);
            }
        }
        for node in statements {
            match node.stmt { // .stmt eklendi
                Stmt::Func { .. } => {} 
                _ => self.execute(node)?, // Artık doğrudan StmtNode yolluyoruz
            }
        }
        Ok(())
    }

    pub fn execute(&mut self, node: StmtNode) -> Result<(), ParseError> {
        let line = node.line;
        match node.stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr, line);
                Ok(())
            }
            Stmt::Print(exprs) => {
                for (index, expr) in exprs.iter().enumerate() {
                    let value = self.evaluate(expr.clone(), line)?;

                    self.print_token_value(&value, line)?;

                    if index < exprs.len() - 1 {
                        print!("");
                    }

                }
                println!();
                Ok(())
            }

            Stmt::Let { name, data_type, value } => {
                let declared_value = self.evaluate(value, line)?;
                if Self::check_type_compatibility(&data_type, &declared_value) {
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert(name, Rc::new(RefCell::new(declared_value)));
                    }
                    Ok(())
                } else {
                    Err(ParseError::TypeMismatch { 
                        expected: data_type, 
                        found: declared_value,
                        line 
                    })
                }
            }

            Stmt::ListAssign { list_name, indices, value } => {
                let new_val = self.evaluate(value, line)?;
                let mut evaluated_indices = Vec::new();
                for expr in indices {
                    match self.evaluate(expr, line)? {
                        Token::Integer(n) => {
                            if n < 0 { return Err(ParseError::RuntimeError { message: "Index cannot be negative!".into(), line }); }
                            evaluated_indices.push(n as usize);
                        }
                        _ => return Err(ParseError::RuntimeError { message: "Index must be an Integer!".into(), line }),
                    }
                }
                if evaluated_indices.is_empty() {
                    return Err(ParseError::RuntimeError { message: "No indices provided!".into(), line });
                }

                let cell = self.get_variable(&list_name)
                    .ok_or_else(|| ParseError::RuntimeError { message: format!("Undeclared list '{}'", list_name), line })?;

                let mut token = cell.borrow_mut();
                let last_idx = evaluated_indices.pop().unwrap();
                let mut current = &mut *token;
                for &idx in &evaluated_indices {
                    current = match current {
                        Token::List(elements) if idx < elements.len() => &mut elements[idx],
                        Token::List(_) => return Err(ParseError::RuntimeError { message: "Index out of bounds!".into(), line }),
                        _ => return Err(ParseError::RuntimeError { message: format!("'{}' is not multi-dimensional!", list_name), line }),
                    };
                }
                match current {
                    Token::List(elements) if last_idx < elements.len() => { elements[last_idx] = new_val; Ok(()) }
                    Token::List(_) => Err(ParseError::RuntimeError { message: "Index out of bounds!".into(), line }),
                    _ => Err(ParseError::RuntimeError { message: "Target is not a list!".into(), line }),
                }
            }

            Stmt::Block(statements) => {
                self.scopes.push(HashMap::new());

                for node in statements {
                    if let Err(e) = self.execute(node) { 
                        self.scopes.pop();
                        return Err(e);
                    }
                }
                self.scopes.pop();
                Ok(())
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let evaluated_cond = self.evaluate(condition, line)?;

                let is_true = match evaluated_cond {
                    Token::Boolean(b) => b,
                    _ => return Err(ParseError::RuntimeError {
                        message: format!("'if' condition must be boolean! Found: {:?}", evaluated_cond),
                        line
                }),
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
                    let evaluated_cond = self.evaluate(condition.clone(), line)?;
                    let is_true = match evaluated_cond {
                        Token::Boolean(b) => b,
                        _ => return Err(ParseError::RuntimeError { message: format!("Runtime Error: 'while' needs to be conditional boolean, found: {:?}", evaluated_cond), line }),
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

                let start_token = self.evaluate(start_value, line)?;
                let end_token = self.evaluate(end_value, line)?;

                let mut current = match start_token {
                    Token::Integer(n) => n,
                    _ => return Err(ParseError::RuntimeError { message: "Runtime Error: For loop start value must be an Integer!".to_string(), line }),
                };

                let limit = match end_token {
                    Token::Integer(n) => n,
                    _ => return Err(ParseError::RuntimeError { message: "Runtime Error: For loop end value must be an Integer!".to_string(), line }),
                };
                
                self.scopes.push(HashMap::new());

                while current < limit {
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert(var_name.clone(), Rc::new(RefCell::new(Token::Integer(current))));
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
                let mut fields = Vec::new();
                for node in body {
                    
                    if let Stmt::Let { name: field_name, data_type: _, .. } = &node.stmt {
                        fields.push((field_name.clone(), Token::Unknown));
                    } else {
                        // Artık doğrudan ana fonksiyondan Err dönebiliriz!
                        return Err(ParseError::RuntimeError {
                            message: "Struct body must only contain field declarations!".to_string(),
                            line: node.line
                        });
                    }
                }
                self.struct_defs.insert(name, fields);
                Ok(())
            }

            Stmt::Impl { name, body } => {
                for node in body { 
                    if let Stmt::Func { name: fn_name, params, body: fn_body } = node.stmt {
                        self.impl_defs
                            .entry(name.clone())
                            .or_default()
                            .insert(fn_name, Function { params, body: fn_body, name: name.clone() });
                    }
                }
                Ok(())
            }

            Stmt::Return { keyword, value } => {
                let return_val = match value {
                    Some(expr) => self.evaluate(expr, line)?,
                    None => Token::Unknown,
                };

                Err(ParseError::Return { value: return_val })
            }

            Stmt::Import { directory, identifier } => {
                if self.loaded_modules.contains_key(&directory) {
                    return Ok(());
                }
                if self.loaded_modules.contains_key(&identifier) {
                    return Err(ParseError::RuntimeError { message: "Runtime error: you can't assign same name in imports".to_string(), line });
                }
                self.loaded_modules.insert(directory.clone(), identifier.clone());

                let import_directory = match fs::read_to_string(&directory) {
                    Ok(c) => c,
                    Err(e) => return Err(ParseError::RuntimeError { message: format!("Runtime Error: could not read file '{}': {}", directory, e), line }),
                };

                let tokens = match lexer::lexer(&import_directory) {
                    Ok(t) => t,
                    Err(e) => return Err(ParseError::RuntimeError { message: format!("Runtime Error: lexer failed in '{}': {}", directory, e), line }),
                };

                let mut parser = Parser::new(tokens);
                let mut ast_tree: Vec<StmtNode> = Vec::new();

                while parser.current_token().is_some() && *parser.current_token().unwrap() != Token::EOF {
                    match parser.parse_statement() {
                        Ok(stmt) => ast_tree.push(stmt),
                        Err(e) => return Err(ParseError::RuntimeError { message: format!("Runtime Error: parser failed in '{}': {:?}", directory, e), line }),
                    }
                }

                let mut sub = Interpreter::new();
                sub.interpret(ast_tree)?;
                let module_scope = sub.scopes.into_iter().next().unwrap_or_default();
                self.module_globals.insert(identifier.clone(), module_scope);

                let module_fns = sub.functions.borrow().clone();
                self.namespaces.insert(identifier.clone(), module_fns);

                for (name, fields) in sub.struct_defs {
                    let namespaced = format!("{}::{}", identifier, name); // e.g. "math::Vector2"
                    self.struct_defs.insert(namespaced, fields);
                }

                // Impl methods → same namespaced key
                for (name, methods) in sub.impl_defs {
                    let namespaced = format!("{}::{}", identifier, name);
                    self.impl_defs.insert(namespaced, methods);
                }

                Ok(())
            }

            _ => Ok(())
        }
    }

    fn evaluate(&mut self, expr: Expr, line: usize) -> Result<Token, ParseError> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                LiteralValue::Int(i) => Token::Integer(i),
                LiteralValue::Float(f) => Token::Float(f),
                LiteralValue::Str(s) => Token::String(s),
                LiteralValue::Bool(b) => Token::Boolean(b),
                LiteralValue::Nil => Token::Unknown,   
            }),

            Expr::Variable(name) => {
                Ok(self.get_variable(&name).map(|c| c.borrow().clone()).unwrap_or(Token::Unknown))
            }

            Expr::Binary { left, op, right } => {
                let left_val = self.evaluate(*left, line)?;
                let right_val = self.evaluate(*right, line)?;

                // 1. Eşitlik Kontrolleri
                if op == Token::Equals {
                    return Ok(Token::Boolean(left_val == right_val));
                }
                if op == Token::NotEquals {
                    return Ok(Token::Boolean(left_val != right_val));
                }

                // 2. Tüm Matematiksel, Karşılaştırma ve String İşlemleri tek bir match altında
                match (left_val, op, right_val) {
                    // Katı Tam Sayı İşlemleri
                    (Token::Integer(l), Token::Plus, Token::Integer(r)) => Ok(Token::Integer(l + r)),
                    (Token::Integer(l), Token::Minus, Token::Integer(r)) => Ok(Token::Integer(l - r)),
                    (Token::Integer(l), Token::Multiply, Token::Integer(r)) => Ok(Token::Integer(l * r)),
                    (Token::Integer(l), Token::Divide, Token::Integer(r)) => {
                        if r == 0 {
                            return Err(ParseError::RuntimeError { message: "Runtime Error: Division by zero!".to_string(), line });
                        }
                        Ok(Token::Integer(l / r))
                    }

                    // Katı Ondalıklı Sayı İşlemleri
                    (Token::Float(l), Token::Plus, Token::Float(r)) => Ok(Token::Float(l + r)),
                    (Token::Float(l), Token::Minus, Token::Float(r)) => Ok(Token::Float(l - r)),
                    (Token::Float(l), Token::Multiply, Token::Float(r)) => Ok(Token::Float(l * r)),
                    (Token::Float(l), Token::Divide, Token::Float(r)) => {
                        if r == 0.0 {
                            return Err(ParseError::RuntimeError { message: "Runtime Error: Division by zero!".to_string(), line });
                        }
                        Ok(Token::Float(l / r))
                    }

                    // String Birleştirme (Sol taraf String ise)
                    (Token::String(l), Token::Plus, other) => {
                        let r_str = match other {
                            Token::Integer(n) => n.to_string(),
                            Token::Float(f) => f.to_string(),
                            Token::Boolean(b) => b.to_string(),
                            Token::String(s) => s,
                            _ => format!("{:?}", other),
                        };
                        Ok(Token::String(format!("{}{}", l, r_str)))
                    }
                    
                    // String Birleştirme (Sağ taraf String ise)
                    (other, Token::Plus, Token::String(r)) => {
                        let l_str = match other {
                            Token::Integer(n) => n.to_string(),
                            Token::Float(f) => f.to_string(),
                            Token::Boolean(b) => b.to_string(),
                            _ => format!("{:?}", other),
                        };
                        Ok(Token::String(format!("{}{}", l_str, r)))
                    }

                    // Genel Sayısal Fallback (Karşılaştırmalar ve Karışık tipler buraya düşer)
                    (l_val, op_tok, r_val) => {
                        if let (Some(l), Some(r)) = (to_float(&l_val), to_float(&r_val)) {
                            match op_tok {
                                Token::Greater => Ok(Token::Boolean(l > r)),
                                Token::Lesser => Ok(Token::Boolean(l < r)),
                                Token::GreaterEquals => Ok(Token::Boolean(l >= r)),
                                Token::LesserEquals => Ok(Token::Boolean(l <= r)),
                                Token::Plus => Ok(Token::Float(l + r)),
                                Token::Minus => Ok(Token::Float(l - r)),
                                Token::Multiply => Ok(Token::Float(l * r)),
                                Token::Divide => {
                                    if r == 0.0 { 
                                        return Err(ParseError::RuntimeError { message: "Runtime Error: Division by zero!".to_string(), line }); 
                                    }
                                    Ok(Token::Float(l / r))
                                }
                                _ => Err(ParseError::RuntimeError { 
                                    message: format!("Type mismatch in binary expression! Left: {:?}, Op: {:?}, Right: {:?}", l_val, op_tok, r_val), 
                                    line 
                                }),
                            }
                        } else {
                            Err(ParseError::RuntimeError { 
                                message: format!("Type mismatch in binary expression! Left: {:?}, Op: {:?}, Right: {:?}", l_val, op_tok, r_val), 
                                line 
                            })
                        }
                    }
                }
            }

            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(*right, line)?;
                match (operator, right_val) {
                    (Token::Minus, Token::Integer(n)) => Ok(Token::Integer(-n)),
                    (Token::Minus, Token::Float(n)) => Ok(Token::Float(-n)),
                    (Token::Bang, Token::Boolean(b)) => Ok(Token::Boolean(!b)),
                    (op, val) => Err(ParseError::RuntimeError { message: format!("{:?} operator cannot used with {:?} .", op, val), line }),
                }
            }

            Expr::Logical { left, operator, right } => {
                let left = self.evaluate(*left, line)?;

                if operator == Token::Or {
                    if let Token::Boolean(b) = left {
                        if b { return Ok(Token::Boolean(true)); }
                    } else {
                        return Err(ParseError::RuntimeError { message: "'or' operator's left needs to be Boolean!".to_string(), line });
                    }
                } else if operator == Token::And {
                    if let Token::Boolean(b) = left {
                        if !b { return Ok(Token::Boolean(false)); }
                    } else {
                        return Err(ParseError::RuntimeError { message: "'and' operator's left needs to be Boolean!".to_string(), line });
                    }
                }

                let right = self.evaluate(*right, line)?;
                if let Token::Boolean(b) = right {
                    Ok(Token::Boolean(b))
                } else {
                    Err(ParseError::RuntimeError { message: "'and'/'or' operator's right needs to be Boolean!".to_string(), line })
                }
            }

            Expr::Assign { name, value } => {
                let new_value = self.evaluate(*value, line)?;
                match self.get_variable(&name) {
                    Some(cell) => {
                        let compatible = Self::check_type_compatibility(&cell.borrow(), &new_value);
                        if compatible {
                            *cell.borrow_mut() = new_value.clone();
                            Ok(new_value)
                        } else {
                            Err(ParseError::TypeMismatch { expected: cell.borrow().clone(), found: new_value, line })
                        }
                    }
                    None => Err(ParseError::UndeclaredVariable { name, line }),
                }
            }

            Expr::Index { list, index } => {
                let list_val = self.evaluate(*list, line)?;
                let index_val = self.evaluate(*index, line)?;
    
                if let (Token::List(elements), Token::Integer(idx)) = (list_val, index_val) {
                    if idx < 0 {
                        return Err(ParseError::RuntimeError { message: format!("Index cannot be negative! Found: {}", idx), line });
                    }
                    let i = idx as usize;
                    if i < elements.len() {
                        Ok(elements[i].clone())
                    } else {
                        Err(ParseError::RuntimeError { message: format!("Index out of bounds! Len: {}, Index: {}", elements.len(), idx), line })
                    }
                } else {
                    Err(ParseError::RuntimeError { message: "Type mismatch. Expected List and Integer index.".to_string(), line })
                } 
            }

            Expr::List(elements) => {
                let mut evaluated_list = Vec::new();
                for expr in elements {
                    let value = self.evaluate(expr, line)?;
                    evaluated_list.push(value);
                }
                Ok(Token::List(evaluated_list))
            }

            Expr::Call { callee, paren: _, arguments } => {
                let name = match *callee {
                    Expr::Variable(ref n) => n.clone(),
                    _ => return Err(ParseError::RuntimeError { message: "Callee must be a named function!".to_string(), line }),
                };

                // Result collect ile argümanları güvenle topluyoruz
                let evaluated_args: Vec<Token> = arguments
                    .into_iter()
                    .map(|arg| self.evaluate(arg, line))
                    .collect::<Result<Vec<Token>, ParseError>>()?;

                if let Some(fields) = self.struct_defs.get(&name).cloned() {
                    let instance_fields = fields.iter().zip(evaluated_args)
                        .map(|((field_name, _), val)| (field_name.clone(), val))
                        .collect();
                    return Ok(Token::StructInstance { type_name: name, fields: instance_fields });
                }

                let (lookup_name, _ns_name) = if name.contains("::") {
                    let parts: Vec<&str> = name.splitn(2, "::").collect();
                    (parts[1].to_string(), Some(parts[0].to_string()))
                } else {
                    (name.clone(), None)
                };

                if let Some(fields) = self.struct_defs.get(&lookup_name).cloned() {
                    let instance_fields = fields.iter().zip(evaluated_args)
                        .map(|((field_name, _), val)| (field_name.clone(), val))
                        .collect();
                    return Ok(Token::StructInstance { type_name: lookup_name, fields: instance_fields });
                }
                
                if let Some(result) = crate::native_functions::dispatch(&name, evaluated_args.clone()) {
                    return Ok(result.unwrap_or(Token::Unknown));
                }

                if let Some(func) = self.native_fns.borrow().get(&name).cloned() {
                    return Ok(func(evaluated_args));
                }

                let func = match self.functions.borrow().get(&name).cloned() {
                    Some(f) => f,
                    None => return Err(ParseError::RuntimeError { message: format!("Undefined function '{}'", name), line }),
                };

                if func.params.len() != evaluated_args.len() {
                    return Err(ParseError::RuntimeError { message: format!("Function '{}' expects {} args but got {}", name, func.params.len(), evaluated_args.len()), line });
                }

                let mut call_scope = HashMap::new();
                for ((param_name, _param_type), arg_val) in func.params.iter().zip(evaluated_args) {
                    call_scope.insert(param_name.clone(), Rc::new(RefCell::new(arg_val)));
                }
                self.scopes.push(call_scope);

                let mut return_value = Token::Unknown;
                let mut encountered_error = None;

                for node in func.body {
                    match self.execute(node) {
                        Ok(_) => {}
                        Err(ParseError::Return { value }) => { 
                            return_value = value; 
                            break; 
                        }
                        Err(e) => { 
                            encountered_error = Some(e); 
                            break; 
                        }
                    }
                }

                // Fonksiyon biter bitmez kendi scope'unu kesinlikle temizle
                let fn_scope = self.scopes.last().cloned().unwrap_or_default();
                self.scopes.pop();

                if let Some(err) = encountered_error {
                    return Err(err);
                }
                
                Ok(return_value)
            }

            Expr::MethodCall { object, method, args } => {
                let obj_name = match *object {
                    Expr::Variable(ref name) => name.clone(),
                    _ => return Err(ParseError::RuntimeError { message: "Invalid method call target".to_string(), line }),
                };

                let evaluated_args: Vec<Token> = args.into_iter()
                    .map(|a| self.evaluate(a, line))
                    .collect::<Result<Vec<Token>, ParseError>>()?;

                if let Some(module_fns) = self.namespaces.get(&obj_name).cloned() {
                    let func = match module_fns.get(&method) {
                        Some(f) => f.clone(),
                        None => return Err(ParseError::RuntimeError { message: format!("Module '{}' has no function '{}'", obj_name, method), line }),
                    };

                    if func.params.len() != evaluated_args.len() {
                        return Err(ParseError::RuntimeError { message: format!("'{}' expects {} args but got {}", method, func.params.len(), evaluated_args.len()), line });
                    }

                    let mut call_scope = HashMap::new();
                    if let Some(globals) = self.module_globals.get(&obj_name).cloned() {
                        for (k, cell) in globals { call_scope.insert(k, cell); }
                    }
                    for ((param_name, _param_type), arg_val) in func.params.iter().zip(evaluated_args) {
                        call_scope.insert(param_name.clone(), Rc::new(RefCell::new(arg_val)));
                    }
                    self.scopes.push(call_scope);

                    let mut return_value = Token::Unknown;
                    for node in func.body {
                        match self.execute(node) {
                            Ok(_) => {}
                            Err(ParseError::Return { value }) => { return_value = value; break; }
                            Err(e) => { self.scopes.pop(); return Err(e); }
                        }
                    }
                    let fn_scope = self.scopes.last().cloned().unwrap_or_default();
                    self.scopes.pop();
                    return Ok(return_value);
                }

                let instance = self.get_variable(&obj_name);
                let type_name = instance.as_ref().and_then(|cell| {
                    if let Token::StructInstance { type_name, .. } = &*cell.borrow() {
                        Some(type_name.clone())
                    } else { None }
                });
                if let Some(type_name) = type_name {
                    let cell = instance.unwrap();
                    let func = match self.impl_defs.get(&type_name).and_then(|m| m.get(&method)).cloned() {
                        Some(f) => f,
                        None => return Err(ParseError::RuntimeError { message: format!("Struct '{}' has no method '{}'", type_name, method), line }),
                    };
                    if func.params.len() != evaluated_args.len() {
                        return Err(ParseError::RuntimeError { message: format!("'{}' expects {} args but got {}", method, func.params.len(), evaluated_args.len()), line });
                    }

                    let mut call_scope = HashMap::new();
                    call_scope.insert("self".to_string(), cell.clone());
                    for ((param_name, _), arg_val) in func.params.iter().zip(evaluated_args) {
                        call_scope.insert(param_name.clone(), Rc::new(RefCell::new(arg_val)));
                    }
                    self.scopes.push(call_scope);

                    let mut return_value = Token::Unknown;
                    for node in func.body {
                        match self.execute(node) {
                            Ok(_) => {}
                            Err(ParseError::Return { value }) => { return_value = value; break; }
                            Err(e) => { self.scopes.pop(); return Err(e); }
                        }
                    }
                    self.scopes.pop();
                    return Ok(return_value);
                }

                let list_cell = self.get_variable(&obj_name);

                match list_cell {
                    Some(cell) => {
                        let mut token = cell.borrow_mut();
                        match &mut *token {
                            Token::List(elements) => match method.as_str() {
                                "push" => {
                                    elements.push(evaluated_args.into_iter().next().unwrap_or(Token::Unknown));
                                    Ok(Token::Unknown)
                                }
                                "pop" => Ok(elements.pop().unwrap_or(Token::Unknown)),
                                "len" => Ok(Token::Integer(elements.len() as i64)),
                                _ => Err(ParseError::RuntimeError { message: format!("Unknown list method '{}'", method), line }),
                            },
                            _ => Err(ParseError::RuntimeError { message: format!("'{}' is not a list", obj_name), line }),
                        }
                    }
                    Some(_) => Err(ParseError::RuntimeError { message: format!("'{}' is not a list, cannot call method '{}'", obj_name, method), line }),
                    None => Err(ParseError::RuntimeError { message: format!("Undefined variable '{}'", obj_name), line }),
                }
            }

            Expr::FieldSet { object, field, value } => {
                let new_val = self.evaluate(*value, line)?;
                let obj_name = match *object {
                    Expr::Variable(ref name) => name.clone(),
                    _ => return Err(ParseError::RuntimeError { message: "Field set on non-variable".to_string(), line }),
                };
                let cell = self.get_variable(&obj_name)
                    .ok_or_else(|| ParseError::RuntimeError { message: format!("Undefined '{}'", obj_name), line })?;
                let mut token = cell.borrow_mut();
                if let Token::StructInstance { fields, .. } = &mut *token {
                    if let Some(f) = fields.iter_mut().find(|(n, _)| n == &field) {
                        f.1 = new_val.clone();
                        return Ok(new_val);
                    }
                }
                Err(ParseError::RuntimeError { message: format!("Field '{}' not found on '{}'", field, obj_name), line })
            }

            Expr::FieldGet { object, field } => {
                let obj = self.evaluate(*object, line)?;
                if let Token::StructInstance { fields, .. } = obj {
                    Ok(fields.into_iter()
                        .find(|(name, _)| name == &field)
                        .map(|(_, val)| val)
                        .unwrap_or(Token::Unknown))
                } else {
                    Err(ParseError::RuntimeError { message: "Field access on non-struct value".to_string(), line })
                }
            }

            Expr::Grouping(inner) => {
                self.evaluate(*inner, line)
            }

            _ => Ok(Token::Unknown),
        }
    }

    fn get_variable(&self, name: &str) -> Option<Rc<RefCell<Token>>> {
        for scope in self.scopes.iter().rev() {
            if let Some(cell) = scope.get(name) {
                return Some(cell.clone());
            }
        }
        None
    }

    fn print_token_value(&self, token: &Token, line: usize) -> Result<(), ParseError> {
        match token {
            Token::String(s) => print!("{} ", s),
            Token::Integer(n) => print!("{} ", n),
            Token::Float(f) => print!("{} ", f),
            Token::Boolean(b) => print!("{} ", b),
            Token::List(elements) => {
                for (i, element) in elements.iter().enumerate() {
                    self.print_token_value(element, line)?;
                    if i < elements.len() - 1 {
                        print!(", ");
                    }
                }
            },
            Token::Identifier(name) => {
                if let Some(cell) = self.get_variable(name) {
                    self.print_token_value(&cell.borrow(), line)?;
                } else {
                    return Err(ParseError::UndeclaredVariable { name: name.clone(), line });
                }
            }
            Token::StructInstance { type_name, fields } => {
                print!("{} {{ ", type_name);
                for (i, (field_name, field_val)) in fields.iter().enumerate() {
                    print!("{}: ", field_name);
                    self.print_token_value(field_val, line)?;
                    if i < fields.len() - 1 { print!(", "); }
                }
                print!("}}");
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: Token::String("a printable value".to_string()),
                found: Some(token.clone()),
                line
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

    #[test]
    fn test_no_caller_scope_leak() {
        let mut engine = WolfEngine::new();
        engine.run(r#"
            struct Counter
                value: int
            end

            impl Counter
                fn tick(value: int)
                    self.value = self.value + value
                    return value
                end
            end

            let value: int = 999
            let c: Counter = Counter(0)
            let result: int = c.tick(5)

            print value
            print result
        "#).unwrap();
    }

    #[test]
    fn test_self_mutation_persists() {
        let mut engine = WolfEngine::new();
        engine.run(r#"
            struct Counter
                value: int
            end

            impl Counter
                fn increment()
                    self.value = self.value + 1
                end
            end

            let c: Counter = Counter(0)
            c.increment()
            c.increment()
            c.increment()
            print c
        "#).unwrap();
    }

}