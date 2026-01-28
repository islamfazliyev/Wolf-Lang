use std::{collections::HashMap, convert::identity, ptr::null, thread::Scope, vec};

use crate::{ast::Stmt, ast::LiteralValue, ast::Expr, error_handler::ParseError, tokens::{self, Token}};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Token)>,
    pub body: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    output: Vec<String>,
    // A stack of HashMaps to manage variable scopes.
    // The last HashMap in the Vec is the innermost (current) scope.
    pub scopes: Vec<HashMap<String, Token>>,
    pub functions: HashMap<String, Function>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            output: Vec::new(),
            // Initialize with one, global scope.
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    fn token_to_literal(&self, token: Token) -> LiteralValue {
        match token {
            Token::Integer(n) => LiteralValue::Int(n),
            Token::Float(f) => LiteralValue::Float(f),
            Token::String(s) => LiteralValue::Str(s),
            Token::Boolean(b) => LiteralValue::Bool(b),
            _ => LiteralValue::Nil,
        }
    }

    /// Returns a reference to the current token without consuming it.
    pub fn current_token(&self) -> Option<&Token> {
        // get token of current position
        self.tokens.get(self.pos)
    }

    /// Returns a reference to the next token without consuming it.
    pub fn peek(&self) -> Option<&Token> {
        // get token of next position
        self.tokens.get(self.pos + 1)
    }

    /// Checks if the current token matches the expected `token_type`.
    /// If it matches, consumes the token and advances the parser. If not, it returns an `Err`.
    fn eat(&mut self, token_type: Token) -> Result<(), ParseError> {
        // gets token type from current token
        if let Some(tok) = self.current_token() {
            // Looks if tok equals to token type
            if *tok == token_type {
                // Matched! Consume the token by advancing the position.
                self.pos += 1;
                Ok(())
            }
            else {
                // Didn't match. Return an error.
                Err(ParseError::UnexpectedToken {
                    expected: token_type,
                    found: Some(tok.clone()),
                })
            }
        }
        else {
            // Reached the end of tokens, but expected something.
            Err(ParseError::UnexpectedToken {
                expected: token_type,
                found: None,
            })
        }
    }

    /// Defines a new variable in the **current** (innermost) scope.
    fn define_variable(&mut self, name: String, value: Token) {
        // .last_mut() gets the innermost scope.
        // .unwrap() is safe because we initialize the scopes vec with one scope in `new()`.
        self.scopes.last_mut().unwrap().insert(name, value);
    }

    /// Assigns a new value to an **existing** variable.
    /// It searches for the variable from the innermost scope outwards to respect variable shadowing.
    /// If found, it performs a type check before updating the value. Returns an error
    /// if the variable is not declared or if the types mismatch.
    fn assign_variable(&mut self, name: &str, value: Token) -> Result<(), ParseError> {
        // .iter_mut().rev() iterates from the last scope (innermost) to the first (global).
        for scope in self.scopes.iter_mut().rev() {
            if let Some(existing_val) = scope.get_mut(name) {
                // Found the variable.
                let existing_val = scope.get_mut(name).unwrap();
                
                // Check if the *type* (enum variant) of the new value matches the old one.
                if std::mem::discriminant(existing_val) == std::mem::discriminant(&value) {
                    
                    // Types match, update the value.
                    *existing_val = value;
                    return Ok(());
                } else {
                    // Type mismatch.
                    return Err(ParseError::TypeMismatch { expected: existing_val.clone(), found: value });
                }
            }
        }
        // If we looped through all scopes and didn't find it, it's undeclared.
        Err(ParseError::UndeclaredVariable { name: name.to_string() })
    }

    /// Retrieves a reference to a variable's value token from the scopes.
    /// It searches for the variable from the innermost scope outwards, returning the first one it finds.
    fn get_variable(&self, name: &str) -> Option<&Token> {
        // .iter().rev() iterates from the innermost scope outwards.
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                // Return the first match we find (respecting shadowing).
                return Some(value);
            }
        }
        // Not found in any scope.
        None
    }
    
    /// Parses a variable declaration statement like 'let int x = 10'.
    /// A 'let' statement must follow the structure: let <type> <identifier> = <value>
    pub fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        // Consume the 'let' keyword to start the statement.
        self.eat(Token::Let)?;


        let var_name = if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.pos += 1;
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::Identifier("name".to_string()),
                found: self.current_token().cloned(),
            });
        };

        self.eat(Token::Colon)?;

        let declared_type = self.parse_type()?;
        self.output.push(format!("type {:?}", declared_type));
        // After type → expect '='
        self.eat(Token::Assign)?;

        let value_expr = self.parse_expr()?;

        Ok(Stmt::Let {
            name: var_name,
            data_type: declared_type,
            value: value_expr,
        })
        
    }

    fn parse_type(&mut self) -> Result<Token, ParseError> {
        // After `idenitifier` → expect type (number/string/bool)
        if let Some(next) = self.current_token().cloned() {
            match next {
                Token::TypeInt | Token::TypeString | Token::TypeBool | Token::TypeFloat => {
                    self.pos += 1;
                    Ok(next)
                }

                Token::TypeList(_) => {
                    self.pos += 1;
                    self.eat(Token::Lesser)?;
                    let inner_type = self.parse_type()?;
                    self.eat(Token::Greater)?;

                    Ok(Token::TypeList(Box::new(inner_type)))

                },

                _ => {
                    // throw error
                    Err(ParseError::UnexpectedToken {
                        expected: Token::TypeInt, // Placeholder
                        found: Some(next),
                    })
                }
            }
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::TypeInt,
                found: None,
            });
        }
        
    }

    fn check_type_compatibility(&mut self, expected: &Token, value: &Token) -> bool {
        match (expected, value) {
            
            (Token::TypeInt, Token::Integer(_)) => true,
            (Token::TypeString, Token::String(_)) => true,
            (Token::TypeBool, Token::Boolean(_)) => true,
            (Token::TypeFloat, Token::Float(_)) => true,

            
            (Token::TypeList(inner_rule), Token::List(elements)) => {
                
                if elements.is_empty() {
                    return true;
                }

                for element in elements {
            
                    if !self.check_type_compatibility(inner_rule, element) {
                        return false;
                    }
                }

                true
            },

            
            _ => false,
        }
    }

    // Note: Typo in function name, should be "parse_assignment"
    fn parse_assigment(&mut self) -> Result<Stmt, ParseError> {
        // 1. Get the variable name
        let var_name = if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.pos += 1;
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::Identifier("variable name".to_string()),
                found: self.current_token().cloned(),
            });
        };

        // 2. Consume '='
        self.eat(Token::Assign)?;

        // 3. Parse the value (the right-hand side)
        let value_expr = self.parse_expr()?;

        // 4. Return an Expression Statement containing the Assignment Expression
        // This wraps the assignment logic into a statement so it can sit in a block.
        Ok(Stmt::Expression(Expr::Assign {
            name: var_name,
            value: Box::new(value_expr),
        }))
    }

    fn parse_list_index(&mut self, list_name: String) -> Result<Expr, ParseError> {
        // Start with the variable: 'arr'
        let mut current_expr = Expr::Variable(list_name);

        // While we see '['...
        while self.check(Token::LBracket) {
            self.eat(Token::LBracket)?; 
            
            // Parse the index expression (e.g., 5 + x)
            let index_expr = self.parse_expr()?;
            
            self.eat(Token::RBracket)?; 

            // Wrap the current expression in an Index node
            // arr[0] becomes Index(arr, 0)
            // arr[0][1] becomes Index(Index(arr, 0), 1)
            current_expr = Expr::Index {
                list: Box::new(current_expr),
                index: Box::new(index_expr),
            };
        }

        Ok(current_expr)
    }

    fn parse_list_assignment(&mut self, list_name: String) -> Result<Stmt, ParseError> {
        let mut indices = Vec::new();

        // 1. Collect all indices: [0][1]...
        while self.check(Token::LBracket) {
            self.eat(Token::LBracket)?;
            indices.push(self.parse_expr()?);
            self.eat(Token::RBracket)?;
        }

        // 2. Expect Assignment '='
        self.eat(Token::Assign)?;

        // 3. Parse the value to assign
        let value = self.parse_expr()?;

        // 4. Return the Statement
        Ok(Stmt::ListAssign {
            list_name,
            indices,
            value,
        })
    }

    /// Parses a 'print' statement. e.g., print "Hello", 10 + 5
    fn parse_print(&mut self) -> Result<Stmt, ParseError> {
        self.eat(Token::Print)?;

        let value = self.parse_expr()?;

        Ok(Stmt::Print(value))
    }

    /// Helper function for `parse_print` to handle the actual printing.
    fn print_token_value(&self, token: &Token) -> Result<(), ParseError> {
        match token {
            // Print literal values directly.
            Token::String(s) => print!("{:?} ", s),
            Token::Integer(n) => print!("{:?} ", n),
            Token::Float(f) => print!("{:?} ", f),
            Token::Boolean(b) => print!("{:?} ", b),
            Token::List(elements) => {
                for (i, element) in elements.iter().enumerate() {
                    self.print_token_value(element)?;
                }
            },
            // If it's an identifier, we need to look up its value.
            Token::Identifier(name) => {
                if let Some(value_token) = self.get_variable(name) {
                    self.print_token_value(value_token)?;
                } else {
                    return Err(ParseError::UndeclaredVariable { name: name.clone() });
                }
            }
            // Error if trying to print something non-printable (like a keyword).
            _ => return Err(ParseError::UnexpectedToken {
                expected: Token::String("a printable value".to_string()),
                found: Some(token.clone()),
            })
        }
        Ok(())
    }

    /// Parses a condition (e.g., 'x > 10', 'name == "iso"').
    fn parse_condition(&mut self) -> Result<Expr, ParseError> {
        self.parse_logic_or()
    }
    fn find_loop_block(&self, start_pos: usize) -> Result<(Vec<Token>, usize), ParseError> {
        let mut temp_pos = start_pos;
        let mut scope_depth = 1;

        while let Some(tok) = self.tokens.get(temp_pos) {
            if *tok == Token::While || *tok == Token::If || *tok == Token::For || *tok == Token::Else { 
                scope_depth += 1;
            } else if *tok == Token::EndOfCondition { // 'end' token'ı
                scope_depth -= 1;
                if scope_depth == 0 {
                    break; 
                }
            }
            temp_pos += 1;
        }

        if scope_depth != 0 {
            return Err(ParseError::UnexpectedToken {
                expected: Token::EndOfCondition,
                found: None,
            });
        }
        
        let tokens: Vec<Token> = self.tokens[start_pos..temp_pos].to_vec();
        let end_pos = temp_pos + 1; // 'end' token'ından sonrası
        Ok((tokens, end_pos))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        
        while let Some(tok) = self.current_token().cloned() {
            match tok {
                Token::EndOfCondition | Token::Else => break,
                Token::EOF => break,

                _ => {
                    statements.push(self.parse_statement()?);
                }
            }
        }
        Ok(statements)
    }

    /// Parses an 'if' statement and its block.
    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.eat(Token::If)?;

        // 1. Parse Condition (returns Expr, doesn't evaluate it)
        let condition = self.parse_condition()?;

        // 2. Parse the "Then" block
        // We keep parsing statements until we hit 'else' or 'end'
        let then_stmts = self.parse_block()?;
        let then_branch = Stmt::Block(then_stmts);

        // 3. Parse the Optional "Else" block
        let else_branch = if let Some(Token::Else) = self.current_token() {
            self.eat(Token::Else)?;
            
            let else_stmts = self.parse_block()?;
            Some(Box::new(Stmt::Block(else_stmts)))
        } else {
            None
        };

        // 4. Consume the final 'end'
        self.eat(Token::EndOfCondition)?;

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    /// Parses a 'while' loop.
    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.eat(Token::While)?;

        // 1. Parse Condition
        let condition = self.parse_condition()?;

        // 2. Parse Body (Collects statements until 'end')
        let body_stmts = self.parse_block()?;
        let body = Stmt::Block(body_stmts);

        // 3. Consume 'end'
        self.eat(Token::EndOfCondition)?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        // 1. Consume 'for'
        self.eat(Token::For)?;
        
        // 2. Expect 'int' (based on your old grammar rules)
        if let Some(Token::TypeInt) = self.current_token() {
            self.eat(Token::TypeInt)?; 
        } else {
            return Err(ParseError::UnexpectedToken { 
                expected: Token::TypeInt, 
                found: self.current_token().cloned() 
            });
        }

        // 3. Get the loop variable name
        let var_name = if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.pos += 1;
            name
        } else {
            return Err(ParseError::UnexpectedToken { 
                expected: Token::Identifier("variable".to_string()), 
                found: self.current_token().cloned() 
            });
        };

        // 4. Parse the range: " = start .. end"
        self.eat(Token::Assign)?;
        let start_value = self.parse_expr()?;

        self.eat(Token::Range)?;
        let end_value = self.parse_expr()?;

        // 5. Parse the body
        // This reuses the 'parse_block' helper we discussed earlier
        let body_stmts = self.parse_block()?;

        // 6. Return the Stmt node
        Ok(Stmt::For {
            var_name,
            start_value,
            end_value,
            body: Box::new(Stmt::Block(body_stmts)),
        })

    }
    
    fn parse_fn(&mut self) -> Result<Stmt, ParseError> {
        // 1. Consume 'fn' keyword
        self.eat(Token::Func)?;

        // 2. Parse Function Name
        let name = if let Some(Token::Identifier(n)) = self.current_token().cloned() {
            self.pos += 1;
            n
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::Identifier("function name".to_string()),
                found: self.current_token().cloned(),
            });
        };

        // 3. Parse Parameters: (name: type, name: type)
        self.eat(Token::LParen)?;
        let mut params = Vec::new();

        // Check if the next token is NOT ')', meaning we have params
        if !self.check(Token::RParen) {
            loop {
                // Get Param Name
                let param_name = if let Some(Token::Identifier(n)) = self.current_token().cloned() {
                    self.pos += 1;
                    n
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: Token::Identifier("param name".to_string()),
                        found: self.current_token().cloned(),
                    });
                };

                // Get Param Type
                self.eat(Token::Colon)?;
                let param_type = self.parse_type()?;
                
                params.push((param_name, param_type));

                // If there's a comma, consume it and continue. Otherwise, stop.
                if let Some(Token::Comma) = self.current_token() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        self.eat(Token::RParen)?;

        // 4. Parse Body
        // We reuse the parse_block() helper to recursively parse statements inside the function
        let body = self.parse_block()?;

        // 5. Consume 'end'
        self.eat(Token::EndOfCondition)?;

        Ok(Stmt::Func {
            name,
            params,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.current_token().cloned().unwrap();
        self.eat(Token::Return)?;

        // Check if there is a value to return.
        // We look ahead to see if the next token starts a new statement or block end.
        // If it's not a terminator, we assume it's an expression.
        let value = if !self.check(Token::EndOfCondition) && !self.check(Token::Else) && self.current_token().is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Stmt::Return {
            keyword,
            value,
        })
    }

    fn check(&self, token: Token) -> bool {
        if let Some(t) = self.current_token() {
            *t == token
        } else {
            false
        }
    }

    /// Parses an expression (handles addition and subtraction).
    /// This has lower precedence than `parse_term`.
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;

        while let Some(tok) = self.current_token().cloned() {
            match tok {
                Token::Plus | Token::Minus => {
                    self.pos += 1;
                    let right = self.parse_term()?;

                    left = Expr::Binary {
                        left: Box::new(left),
                        op: tok,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }
        
        Ok(left)
    }

    /// Parses a term (handles multiplication and division).
    /// This has higher precedence than `parse_expr`.
    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor()?;

        while let Some(tok) = self.current_token().cloned() {
            match tok {
                Token::Multiply | Token::Divide => {
                    self.pos += 1;
                    let right = self.parse_factor()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op: tok,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Parses a factor (the highest precedence: literals, variables, unary ops, parentheses).
    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let tok = self.current_token().cloned().ok_or(ParseError::UnexpectedToken {
            expected: Token::Integer(0), found: None,
        })?;

        match tok {
            // --- 1. Literals ---
            Token::Integer(_) | Token::Float(_) | Token::String(_) | Token::Boolean(_) => {
                self.pos += 1;
                Ok(Expr::Literal(self.token_to_literal(tok)))
            }
            
            // --- 2. Identifiers (Variables, Calls, Lists) ---
            Token::Identifier(name) => {
                self.pos += 1; // Consume the identifier name immediately

                if self.check(Token::LParen) {
                    // It is a Function Call: run()
                    self.parse_call_expr(name)
                } 
                else if self.check(Token::LBracket) {
                    // It is a List Access: arr[0]
                    self.parse_list_index(name)
                }
                else if self.check(Token::Dot) {
                    // It is a Method Call: list.push()
                    self.parse_method_call(name)
                }
                else {
                    // It is just a Variable: x
                    Ok(Expr::Variable(name))
                }
            },

            // --- 3. Parentheses (Grouping) ---
            Token::LParen => {
                self.eat(Token::LParen)?;
                let expr = self.parse_expr()?;
                self.eat(Token::RParen)?;
                Ok(Expr::Grouping(Box::new(expr)))
            },

            Token::LBracket => {
                self.eat(Token::LBracket)?; // Consume '['
                
                let mut elements = Vec::new();
                
                // Check if list is not empty (i.e. next token is NOT ']')
                if !self.check(Token::RBracket) {
                    loop {
                        // Parse the element (it can be any expression: 1, x, 5+5)
                        elements.push(self.parse_expr()?);

                        // If there is a comma, consume it and continue
                        if self.check(Token::Comma) {
                            self.eat(Token::Comma)?;
                        } else {
                            // No comma means end of list elements
                            break;
                        }
                    }
                }
                
                self.eat(Token::RBracket)?; // Consume ']'
                
                // Return the AST node for a List
                Ok(Expr::List(elements))
            },

            // --- 4. Unary Minus (-5) ---
            Token::Minus => {
                self.eat(Token::Minus)?;
                let right = self.parse_factor()?;
                Ok(Expr::Unary {
                    operator: Token::Minus,
                    right: Box::new(right),
                })
            },
            
            _ => Err(ParseError::UnexpectedToken { expected: Token::Unknown, found: Some(tok) })
        }
    }

    fn parse_call_expr(&mut self, name: String) -> Result<Expr, ParseError> {
        // We already consumed the identifier in parse_factor, now consume '('
        self.eat(Token::LParen)?;
        
        let mut arguments = Vec::new();
        if self.current_token() != Some(&Token::RParen) {
            loop {
                arguments.push(self.parse_expr()?); // These now return Expr
                if let Some(Token::Comma) = self.current_token() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        
        self.eat(Token::RParen)?;

        // Return the Call variant from your ast::Expr
        Ok(Expr::Call {
            callee: Box::new(Expr::Variable(name)),
            paren: Token::RParen, // Stored for error reporting later
            arguments,
        })
    }

    fn parse_logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_logic_and()?;

        while let Some(tok) = self.current_token().cloned() {
            if tok == Token::Or {
                self.pos += 1;
                // 'right' must also be an Expr
                let right = self.parse_logic_and()?;
                
                left = Expr::Logical {
                    left: Box::new(left),
                    operator: tok,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;

        while let Some(tok) = self.current_token().cloned() {
            if tok == Token::And {
                self.pos += 1;
                let right = self.parse_comparison()?;
                left = Expr::Logical {
                    left: Box::new(left),
                    operator: tok,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }



    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        // 1. Get the left side (Math level: 5 + 5)
        let mut left = self.parse_expr()?; 

        // 2. Check for comparison operators
        if let Some(tok) = self.current_token().cloned() {
            match tok {
                Token::Equals | Token::NotEquals | Token::Greater | 
                Token::Lesser | Token::GreaterEquals | Token::LesserEquals => {
                    self.pos += 1;
                    // 3. Get the right side
                    let right = self.parse_expr()?;
                    
                    // 4. Return the Binary Node (NOT the result)
                    return Ok(Expr::Binary {
                        left: Box::new(left),
                        op: tok,
                        right: Box::new(right),
                    });
                }
                _ => {}
            }
        }
        // If no operator, just return the math expression (e.g., "if (x)")
        Ok(left)
    }

    fn parse_method_call(&mut self, var_name: String) -> Result<Expr, ParseError> {
        // 1. Consume the dot '.'
        self.eat(Token::Dot)?;

        // 2. Get the method name (e.g., "push", "len")
        let method_name = if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.pos += 1;
            name
        } else {
            return Err(ParseError::UnexpectedToken { 
                expected: Token::Identifier("method name".to_string()), 
                found: self.current_token().cloned() 
            });
        };

        // 3. Consume '('
        self.eat(Token::LParen)?;

        // 4. Parse arguments: (arg1, arg2)
        let mut args: Vec<Expr> = Vec::new();

        // Check if the next token is NOT ')', meaning we have params
        if !self.check(Token::RParen) {
            loop {
                // Recursively parse the argument as an expression
                args.push(self.parse_expr()?);

                if let Some(Token::Comma) = self.current_token() {
                    self.eat(Token::Comma)?;
                } else {
                    break;
                }
            }
        }

        // 5. Consume ')'
        self.eat(Token::RParen)?;

        // 6. Return the AST Node
        // We wrap the object name (var_name) in an Expr::Variable so the Interpreter
        // knows to look it up later.
        Ok(Expr::MethodCall {
            object: Box::new(Expr::Variable(var_name)),
            method: method_name,
            args,
        })
    }

    /// The main dispatch function. It "senses" what the current token is
    /// and dispatches to the correct parsing function for that statement.
    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        if self.current_token().is_none() {
            return Err(ParseError::UnexpectedToken { 
                expected: Token::Unknown, found: None 
            });
        }

        let token = self.current_token().unwrap().clone();

        match token {
            Token::Let => self.parse_let(),
            Token::Print => self.parse_print(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Func => self.parse_fn(),
            Token::Return => self.parse_return(),
            
            // --- The Tricky Part: Identifiers ---
            Token::Identifier(name) => {
                // We consume the identifier here because we need to see what comes AFTER it.
                self.pos += 1; 

                // Case A: Variable Assignment (x = 10)
                if self.check(Token::Assign) {
                    self.eat(Token::Assign)?;
                    let value = self.parse_expr()?;
                    Ok(Stmt::Expression(Expr::Assign {
                        name,
                        value: Box::new(value),
                    }))
                }
                // Case B: List Assignment (arr[0] = 10) OR List Access (arr[0])
                else if self.check(Token::LBracket) {
                    // We parse the indices: [0][1]
                    let mut indices = Vec::new();
                    while self.check(Token::LBracket) {
                        self.eat(Token::LBracket)?;
                        indices.push(self.parse_expr()?);
                        self.eat(Token::RBracket)?;
                    }

                    // Check if an '=' follows the brackets
                    if self.check(Token::Assign) {
                        // It is a List Assignment: arr[0] = 5
                        self.eat(Token::Assign)?;
                        let value = self.parse_expr()?;
                        Ok(Stmt::ListAssign { list_name: name, indices, value })
                    } else {
                        // It was just a List Access expression: arr[0]
                        // We reconstruct the expression tree
                        let mut expr = Expr::Variable(name);
                        for idx in indices {
                            expr = Expr::Index { 
                                list: Box::new(expr), 
                                index: Box::new(idx) 
                            };
                        }
                        Ok(Stmt::Expression(expr))
                    }
                }
                // Case C: Method Call (list.push())
                else if self.check(Token::Dot) {
                    // We call the helper we wrote earlier
                    // Note: we pass 'name' because we already consumed the identifier
                    let expr = self.parse_method_call(name)?;
                    Ok(Stmt::Expression(expr))
                }
                // Case D: Function Call (run())
                else if self.check(Token::LParen) {
                    let expr = self.parse_call_expr(name)?;
                    Ok(Stmt::Expression(expr))
                }
                // Case E: Just a standalone variable (x)
                else {
                    Ok(Stmt::Expression(Expr::Variable(name)))
                }
            },
            
            _ => Err(ParseError::UnexpectedToken {
                expected: Token::Unknown,
                found: Some(token),
            }),
        }
    }
        
}