use std::{collections::HashMap, ptr::null, thread::Scope, vec};

use crate::{error_handler::ParseError, tokens::{self, Token}};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
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
            if scope.contains_key(name) {
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
    pub fn parse_let(&mut self) -> Result<(), ParseError> {
        // Consume the 'let' keyword to start the statement.
        self.eat(Token::Let)?;

        // These variables will hold the parts of the declaration as we parse them.
        let mut var_name: Option<String> = None;
        let mut assigned_value: Option<Token> = None;
        let mut declared_type: Option<Token> = None;

        // After `let` → expect type (number/string/bool)
        if let Some(next) = self.current_token() {
            match next {
                Token::TypeInt | Token::TypeString | Token::TypeBool | Token::TypeFloat | Token::TypeList => {
                    declared_type = Some(next.clone());
                    let ty = format!("{:?}", next);
                    self.output.push(format!("type {}", ty));
                    self.pos += 1; // consume type
                }
                _ => {
                    // throw error
                    return Err(ParseError::UnexpectedToken {
                        expected: Token::TypeInt, // just a placeholder
                        found: Some(next.clone()),
                    })
                }
            }
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::TypeInt,
                found: None,
            });
        }

        // After the type, we expect the name of the variable.
        if let Some(next) = self.current_token() {
            match next {
                Token::Identifier(name) => {
                    var_name = Some(name.clone());
                    self.output.push(format!("var {}", name));
                    self.pos += 1; // consume identifier
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: Token::Identifier("x".to_string()),
                        found: Some(next.clone()),
                    })
                }
            }
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::Identifier("x".to_string()),
                found: None,
            })
        }

        // After identifier → expect '='
        self.eat(Token::Assign)?;

        let assigned_value = self.parse_expr()?;



        // Check if type and value matches or not 
        if let Some(decl_ty) = declared_type.as_ref() {
            let matches = match (decl_ty, &assigned_value) {
                (Token::TypeBool, Token::Boolean(_)) => true,
                (Token::TypeInt, Token::Integer(_)) => true,
                (Token::TypeFloat, Token::Float(_)) => true,
                (Token::TypeString, Token::String(_)) => true,
                (Token::TypeList, Token::List(_)) => true,
                _ => false
            };

            // if doesn't match return error
            if !matches {
                return Err(ParseError::TypeMismatch {
                    expected: decl_ty.clone(),
                    found: assigned_value.clone(),
                });
            }
        }

        // If all parts are parsed successfully, define the variable in the current scope.
        if let Some(name) = var_name {
            self.define_variable(name, assigned_value); 
        }
        Ok(())
    }

    // Note: Typo in function name, should be "parse_assignment"
    fn parse_assigment(&mut self) -> Result<(), ParseError> {
        // `sense()` already checked that the current token is an Identifier.
        let var_name = if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.pos += 1; // Consume the identifier
            name
        } else {
            unreachable!(); // Should not happen if called from `sense()`
        };

        // `sense()` already checked that the next token is Assign.
        // We should use `?` to propagate errors from `eat`.
        self.eat(Token::Assign)?;
        
        // Parse the expression on the right-hand side.
        let new_value_token = self.parse_expr()?;

        // Try to assign the new value, which also handles type checking.
        // We should also propagate the result of `assign_variable`.
        self.assign_variable(&var_name, new_value_token)?;
        
        Ok(())
    }

    fn parse_list_index(&mut self, list_name: &str) -> Result<Token, ParseError> {
        self.eat(Token::Identifier(list_name.to_string()))?;
        self.eat(Token::LBracket)?;
        let index_expr = self.parse_expr()?;
        
        self.eat(Token::RBracket)?;

        let index_val = if let Token::Integer(n) = index_expr {
            if n < 0 {
                return Err(ParseError::UnkownType { 
                    type_name: format!("Index cannot be negative: {}", n) 
                });
            }
            n as usize
        } else {
            return Err(ParseError::TypeMismatch {
                expected: Token::TypeInt,
                found: index_expr,
            });
        };

        let variable_token = self.get_variable(list_name)
            .ok_or(ParseError::UndeclaredVariable { name: list_name.to_string() })?
            .clone();

        if let Token::List(elements) = variable_token {
            if index_val < elements.len() {
                Ok(elements[index_val].clone())
            } else {
                return Err(ParseError::UnkownType {
                    type_name: format!("Index out of bounds! Index: {}, Length: {}", index_val, elements.len())
                })
            }
        } else {
            return Err(ParseError::TypeMismatch {
                expected: Token::TypeList,
                found: variable_token,
            })
        }
    }

    /// Parses a 'print' statement. e.g., print "Hello", 10 + 5
    fn parse_print(&mut self) -> Result<(), ParseError> {
        // Consume the 'print' keyword.
        self.eat(Token::Print)?;

        // Parse the first expression to be printed.
        let first_value = self.parse_expr()?;
        self.print_token_value(&first_value)?; 

        // Loop as long as we see commas, allowing for multiple arguments.
        while let Some(Token::Comma) = self.current_token() {
            self.eat(Token::Comma)?; // Consume the comma

            // Parse the next expression.
            let next_value = self.parse_expr()?;
            // Print its value.
            self.print_token_value(&next_value)?;
        }

        // Print a newline after the statement is done.
        println!(); 

        Ok(())
    }

    /// Helper function for `parse_print` to handle the actual printing.
    fn print_token_value(&self, token: &Token) -> Result<(), ParseError> {
        match token {
            // Print literal values directly.
            Token::String(s) => print!("{:?} ", s),
            Token::Integer(n) => print!("{:?} ", n),
            Token::Float(f) => print!("{:?} ", f),
            Token::Boolean(b) => print!("{:?} ", b),
            // If it's an identifier, we need to look up its value.
            Token::Identifier(name) => {
                if let Some(value_token) = self.get_variable(name) {
                    // Found the variable, now print its stored value.
                    match value_token {
                        Token::String(s) => print!("{:?} ", s),
                        Token::Integer(n) => print!("{:?} ", n),
                        Token::Float(f) => print!("{:?} ", f),
                        Token::Boolean(b) => print!("{:?} ", b),
                        _ => {} // Should not happen if types are managed correctly
                    }
                } else {
                    // Tried to print a variable that doesn't exist.
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
    fn parse_condition(&mut self) -> Result<bool, ParseError> {
        // Parse the left-hand side of the comparison.
        let lhs_token = self.parse_expr()?;

        // Get the comparison operator.
        let op_token_opt = self.current_token().cloned();

        if let Some(op_token) = op_token_opt {
            match op_token {
                // Check if it's a valid comparison operator.
                Token::Equals | Token::NotEquals | Token::Greater | 
                Token::Lesser | Token::GreaterEquals | Token::LesserEquals => {
                    // Consume the operator.
                    self.pos += 1;

                    // Parse the right-hand side.
                    let rhs_token = self.parse_expr()?;
                    
                    // Perform type-checked comparison.
                    match (&lhs_token, &rhs_token) {
                        // Both are numbers.
                        (Token::Integer(lhs), Token::Integer(rhs)) => {
                            let result = match op_token {
                                Token::Equals => lhs == rhs,
                                Token::NotEquals => lhs != rhs,
                                Token::Greater => lhs > rhs,
                                Token::Lesser => lhs < rhs,
                                Token::GreaterEquals => lhs >= rhs,
                                Token::LesserEquals => lhs <= rhs,
                                _ => unreachable!(), // We matched these tokens above
                            };
                            return Ok(result);
                        }

                        (Token::Float(lhs), Token::Float(rhs)) => {
                            let result = match op_token {
                                Token::Equals => lhs == rhs,
                                Token::NotEquals => lhs != rhs,
                                Token::Greater => lhs > rhs,
                                Token::Lesser => lhs < rhs,
                                Token::GreaterEquals => lhs >= rhs,
                                Token::LesserEquals => lhs <= rhs,
                                _ => unreachable!(), // We matched these tokens above
                            };
                            return Ok(result);
                        }

                        // Both are strings.
                        (Token::String(lhs), Token::String(rhs)) => {
                            let result = match op_token {
                                Token::Equals => lhs == rhs,
                                Token::NotEquals => lhs != rhs,
                                // Other comparisons (>, <) are not allowed for strings.
                                _ => {
                                    return Err(ParseError::UnexpectedToken {
                                        expected: Token::Equals, // or NotEquals
                                        found: Some(op_token),
                                    })
                                },
                            };
                            return Ok(result);
                        }
                        // Both are booleans.
                        (Token::Boolean(lhs), Token::Boolean(rhs)) => {
                            let result = match op_token {
                                Token::Equals => lhs == rhs,
                                Token::NotEquals => lhs != rhs,
                                // Other comparisons (>, <) are not allowed for booleans.
                                _ => {
                                    return Err(ParseError::UnexpectedToken {
                                        expected: Token::Equals, // or NotEquals
                                        found: Some(op_token),
                                    })
                                },
                            };
                            return Ok(result);
                        }
                        // Types do not match (e.g., comparing a number to a string).
                        _ => {
                            return Err(ParseError::TypeMismatch {
                                expected: lhs_token.clone(),
                                found: rhs_token,
                            });
                        }
                    }

                }
                // Not a comparison operator, fall through to the error.
                _ => {}
            }
        }
        
        // If we're here, we parsed an LHS but didn't find a valid operator.
        Err(ParseError::UnexpectedToken {
            expected: Token::Equals, // Placeholder for "any comparison op"
            found: self.current_token().cloned(),
        })
    }

    fn find_loop_block(&self, start_pos: usize) -> Result<(Vec<Token>, usize), ParseError> {
        let mut temp_pos = start_pos;
        let mut scope_depth = 1;

        while let Some(tok) = self.tokens.get(temp_pos) {
            if *tok == Token::While || *tok == Token::If || *tok == Token::For { 
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

    /// Parses an 'if' statement and its block.
    fn parse_if(&mut self) -> Result<(), ParseError> {
        // Consume the 'if' keyword.
        self.eat(Token::If)?;

        // Evaluate the condition.
        let condition_result = self.parse_condition()?; 
        
        // --- Find the block of tokens belonging to this 'if' ---
        let (block_tokens, end_of_loop_pos) = self.find_loop_block(self.pos)?;
        
        // We found the block. Advance the *main* parser's position to *after* the 'end'.
        self.pos = end_of_loop_pos;

        // --- Execute the block if the condition was true ---
        if condition_result {
            // Create a new scope for the 'if' block.
            self.scopes.push(HashMap::new());
            
            // Create a new, temporary parser just for the block's tokens.
            let mut block_parser = Parser::new(block_tokens);
            // Give it a *copy* of the current scopes (so it can read outer variables).
            block_parser.scopes = self.scopes.clone(); 
           
            block_parser.functions = self.functions.clone();

            // Parse all statements inside the block.
            while let Some(_) = block_parser.current_token() {
                block_parser.sense()?;
            }
            
            // Retrieve the scopes from the block parser. This is important
            // if we want to allow variable modifications, but for 'let',
            // we are just popping the scope anyway.
            self.scopes = block_parser.scopes;
            
            // Pop the 'if' block's scope, discarding its new variables.
            self.scopes.pop();
        }
        
        Ok(())
    }

    /// Parses a 'while' loop.
    fn parse_while(&mut self) -> Result<(), ParseError> {
        // Consume 'while' keyword.
        self.eat(Token::While)?;
        
        // Save the token position where the condition starts.
        // We need this to re-evaluate it on each loop.
        let condition_start_pos = self.pos;

        // --- Find the start of the block ---
        // We create a temporary parser to parse the condition *without*
        // executing it, just to find out where it ends.
        let block_start_pos = {
            let mut temp_parser = self.clone();
            temp_parser.parse_condition()?;
            temp_parser.pos // This is the position *after* the condition.
        };
        
        // --- Find the end of the block and get all tokens inside it ---
        let (block_tokens, end_of_loop_pos) = self.find_loop_block(block_start_pos)?;


        // --- Main loop execution ---
        loop {
            // **Rewind** the parser to the start of the condition.
            self.pos = condition_start_pos;
            
            // **Re-evaluate** the condition.
            let condition_result = self.parse_condition()?;
            
            if condition_result {
                // Condition is true, execute the block.
                
                // Create a new scope for the loop iteration.
                self.scopes.push(HashMap::new());
                
                // Create a new parser for the block tokens.
                let mut block_parser = Parser::new(block_tokens.clone());
                block_parser.scopes = self.scopes.clone(); // Give it current scopes
                
                block_parser.functions = self.functions.clone();

                // Parse all statements in the block.
                while let Some(_) = block_parser.current_token() {
                    block_parser.sense()?;
                }
                
                // Discard the inner-loop-parser's scopes, but keep our own.
                self.scopes = block_parser.scopes;
                // Pop the loop iteration's scope.
                self.scopes.pop();
                
                // Go back to the top of the `loop` to re-evaluate.
                continue;
            } else {
                // Condition is false, stop looping.
                break;
            }
        }
        
        // The loop is finished. Set the main parser's position
        // to *after* the 'end' token we found earlier.
        self.pos = end_of_loop_pos;
        
        Ok(())
    }

    fn parse_for(&mut self) -> Result<(), ParseError> {

        // Consume For token
        self.eat(Token::For)?;
        
        match self.current_token() {
            Some(Token::TypeInt) => {
                self.eat(Token::TypeInt)?; 
            }

            Some(tok) => {
                // ERROR: 'int' expected after 'for'
                return Err(ParseError::TypeMismatch { expected: Token::TypeInt, found: tok.clone() });
            },
            None => {
                // ERROR: File ended after 'for'
                return Err(ParseError::UnexpectedToken { expected: Token::TypeInt, found: None });
            }
            
            _ => {
                
            }
        }

        let var_name = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name_clone = name.clone();
                self.pos += 1; // Consume Idenifier
                name_clone
            },

            _ => return Err(ParseError::UnexpectedToken { expected: Token::Identifier("identifier".to_string()), found: self.current_token().cloned() })
        };

        self.eat(Token::Assign)?;

        let start_val = match self.current_token() {
            Some(Token::Integer(n)) => {
                let val = *n;
                self.pos += 1;
                val
            }
            _ => return Err(ParseError::UnexpectedToken { expected: Token::Integer(0), found: self.current_token().cloned() })
        };

        self.eat(Token::Range)?;

        let end_val = match self.current_token() {
            Some(Token::Integer(n)) => {
                let val = *n;
                self.pos += 1; // Consume number
                val
            },
            _ => return Err(ParseError::UnexpectedToken { expected: Token::Integer(0), found: self.current_token().cloned() })
        };

        let (block_tokens, end_of_loop_pos) = self.find_loop_block(self.pos)?;

        let mut current_val = start_val;

        while current_val < end_val {
            self.scopes.push(HashMap::new());

            self.define_variable(var_name.clone(), Token::Integer(current_val));

            let mut block_parser = Parser::new(block_tokens.clone());
            block_parser.scopes = self.scopes.clone();

            block_parser.functions = self.functions.clone();

            while let Some(_) = block_parser.current_token() {
                
                block_parser.sense()?;
            }
            
            self.scopes = block_parser.scopes;
            self.scopes.pop(); // 'scopes.pop()'
            
            current_val += 1;
        }

        self.pos = end_of_loop_pos;

        Ok(())

    }
    
    fn parse_fn(&mut self) -> Result<(), ParseError> {
        self.eat(Token::Func)?;

        let name = match self.current_token() {
            Some(Token::Identifier(n)) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken { expected: Token::Identifier("identifier".to_string()), found: self.current_token().cloned() }),
        };
        self.pos += 1;

        self.eat(Token::LParen)?;

        let mut params = Vec::new();
        while let Some(Token::Identifier(param_name)) = self.current_token() {
            params.push(param_name.clone());
            self.pos += 1;
            if let Some(Token::Comma) = self.current_token() {
                self.pos += 1;
            }
            else {
                break;
            }
        }

        self.eat(Token::RParen)?;

        let (body_tokens, end_pos) = self.find_loop_block(self.pos)?;

        let func = Function {name: name.clone(), params, body: body_tokens};
        self.functions.insert(name, func);

        self.pos = end_pos;

        Ok(())
    }

    fn parse_return(&mut self) -> Result<(), ParseError>{
        self.eat(Token::Return)?;
        let value = self.parse_expr()?;
        return Err(ParseError::Return { value });
    }

    /// Parses an expression (handles addition and subtraction).
    /// This has lower precedence than `parse_term`.
    fn parse_expr(&mut self) -> Result<Token, ParseError> {
        // First, parse a 'term' (which handles multiplication/division).
        let mut result = self.parse_term()?; 

        // Loop as long as we find '+' or '-' tokens.
        while let Some(tok) = self.current_token() {
            match tok {
                Token::Plus => {
                    self.eat(Token::Plus)?;
                    // Parse the next term.
                    let next_term = self.parse_term()?;
                    match (&result, &next_term) {
                        (Token::Integer(lhs), Token::Integer(rhs)) => {
                            result = Token::Integer(lhs + rhs);
                        },
                        (Token::Float(lhs), Token::Float(rhs)) => {
                            result = Token::Float(lhs + rhs);
                        },
                        
                        (Token::String(lhs), Token::String(rhs)) => {
                            result = Token::String(format!("{}{}", lhs, rhs));
                        },

                        (Token::String(lhs), Token::Integer(rhs)) => {
                            result = Token::String(format!("{}{}", lhs, rhs));
                        },

                        (Token::Integer(lhs), Token::String(rhs)) => {
                            result = Token::String(format!("{}{}", lhs, rhs));
                        },

                        (Token::String(lhs), Token::Boolean(rhs)) => {
                            result = Token::String(format!("{}{}", lhs, rhs));
                        },
                        
                        _ => {
                            return Err(ParseError::TypeMismatch {
                                expected: Token::TypeInt,
                                found: next_term,
                            });
                        }
                    }
                    
                }
                Token::Minus => {
                    self.eat(Token::Minus)?;
                    // Parse the next term.
                    let next_term = self.parse_term()?;
                    // Type-check and perform the operation.
                    if let (Token::Integer(lhs), Token::Integer(rhs)) = (&result, &next_term) {
                        result = Token::Integer(lhs - rhs);
                    } else if let (Token::Float(lhs), Token::Float(rhs)) = (&result, &next_term) {
                        result = Token::Float(lhs - rhs);
                        
                    } else {
                        // Error: "Cannot subtract {type} from {type}"
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeInt,
                            found: next_term,
                        });
                    }
                }
                // Not a '+' or '-', so the expression is done.
                _ => break,
            }
        }
        Ok(result)
    }

    /// Parses a term (handles multiplication and division).
    /// This has higher precedence than `parse_expr`.
    fn parse_term(&mut self) -> Result<Token, ParseError> {
        // First, parse a 'factor' (handles numbers, parens, etc.).
        let mut result = self.parse_factor()?; 

        // Loop as long as we find '*' or '/' tokens.
        while let Some(tok) = self.current_token() {
            match tok {
                Token::Multiply => {
                    self.eat(Token::Multiply)?;
                    // Parse the next factor.
                    let next_factor = self.parse_factor()?;
                    // Type-check and perform the operation.
                    if let (Token::Integer(lhs), Token::Integer(rhs)) = (&result, &next_factor) {
                        result = Token::Integer(lhs * rhs);
                    } else if let (Token::Float(lhs), Token::Float(rhs)) = (&result, &next_factor) {
                        result = Token::Float(lhs * rhs);
                    } else {
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeInt,
                            found: next_factor,
                        });
                    }
                }
                Token::Divide => {
                    self.eat(Token::Divide)?;
                    // Parse the next factor.
                    let next_factor = self.parse_factor()?;
                    // Type-check and perform the operation.
                    if let (Token::Integer(lhs), Token::Integer(rhs)) = (&result, &next_factor) {
                        // Note: This will panic if `rhs` is 0.
                        // A real language should handle this division-by-zero error.
                        result = Token::Integer(lhs / rhs);
                    } else if let (Token::Float(lhs), Token::Float(rhs)) = (&result, &next_factor) {
                        // Note: This will panic if `rhs` is 0.
                        // A real language should handle this division-by-zero error.
                        result = Token::Float(lhs / rhs);
                    } else {
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeInt,
                            found: next_factor,
                        });
                    }
                }
                // Not a '*' or '/', so the term is done.
                _ => break,
            }
        }
        Ok(result)
    }

    /// Parses a factor (the highest precedence: literals, variables, unary ops, parentheses).
    fn parse_factor(&mut self) -> Result<Token, ParseError> {
        let tok = match self.current_token().cloned() {
            Some(t) => t,
            None => {
                // Reached end of input unexpectedly.
                return Err(ParseError::UnexpectedToken {
                    expected: Token::Integer(0), // Placeholder
                    found: None,
                });
            }
        };

        match tok {
            // A literal number.
            Token::Integer(n) => {
                self.eat(Token::Integer(n))?;
                Ok(Token::Integer(n))
            }

            Token::Float(f) => {
                self.eat(Token::Float(f))?;
                Ok(Token::Float(f))
            }

            // A literal string.
            Token::String(s) => {
                self.eat(Token::String(s.clone()))?;
                Ok(Token::String(s))
            }
            // A literal boolean.
            Token::Boolean(b) => {
                self.eat(Token::Boolean(b))?;
                Ok(Token::Boolean(b))
            }
            // Unary plus (e.g., "+5").
            Token::Plus => {
                self.eat(Token::Plus)?;
                self.parse_factor() // Parse the factor that follows.
            }
            // Unary minus (e.g., "-5").
            Token::Minus => {
                self.eat(Token::Minus)?;
                let value_token = self.parse_factor()?;
                // Apply the negation.
                if let Token::Integer(n) = value_token {
                    Ok(Token::Integer(-n))
                } else if let Token::Float(f) = value_token {
                    Ok(Token::Float(-f))
                } else {
                    // Error: "Cannot apply unary minus to {type}"
                    Err(ParseError::TypeMismatch {
                        expected: Token::TypeInt,
                        found: value_token,
                    })
                }
            }
            // Parentheses (e.g., "(10 + 5)").
            Token::LParen => {
                self.eat(Token::LParen)?; // Consume '('
                let result = self.parse_expr()?; // Parse the expression inside.
                self.eat(Token::RParen)?; // Consume ')'
                Ok(result) // Return the result of the inner expression.
            }
            // Brackets (e.g., "[string "hello world", int 20]").
            Token::LBracket => {    
                // Consume the opening bracket '['.
                self.eat(Token::LBracket)?;
                
                // A vector to hold the parsed elements of the list.
                let mut elements: Vec<Token> = Vec::new(); 

                // Check if the list is empty (i.e., next token is immediately ']').
                if self.current_token() != Some(&Token::RBracket) {
                    loop {
                        // Parse the expression for the current element (e.g., 10, "text", 5+5).
                        let element = self.parse_expr()?;
                        elements.push(element);

                        // If there is a comma, consume it and parse the next element.
                        // Otherwise, we reached the end of the list elements.
                        if let Some(Token::Comma) = self.current_token() {
                            self.eat(Token::Comma)?;
                        } else {
                            break;
                        }
                    }
                }

                // Consume the closing bracket ']'.
                self.eat(Token::RBracket)?;
                
                // Wrap the collected elements in a Token::List and return.
                Ok(Token::List(elements))
            }
            // A variable.
            Token::Identifier(name) => {
                if let Some(Token::LParen) = self.peek() {
                    self.parse_function_call(&name)
                } else if let Some(Token::LBracket) = self.peek() {
                    self.parse_list_index(&name)
                } else {
                    
                    self.eat(Token::Identifier(name.clone()))?;
                    // Look up the variable's value.
                    if let Some(value_token) = self.get_variable(&name) {
                        Ok(value_token.clone()) // Return its value.
                    } else {
                        // Error: "Variable not found"
                        Err(ParseError::UndeclaredVariable { name })
                    }
                }
            }
            
            // Any other token is unexpected at this point in an expression.
            _ => {
                println!("error here");
                Err(ParseError::UnexpectedToken {
                    expected: Token::Integer(0), // Placeholder
                    found: Some(tok),
                })
            }
        }
       
    }

    fn parse_function_call(&mut self, name: &str) -> Result<Token, ParseError> {
        // 1. Consume the function name and the opening parenthesis '('.
        self.eat(Token::Identifier(name.to_string()))?;
        self.eat(Token::LParen)?;
        
        // Vector to hold the evaluated arguments passed to the function.
        let mut args: Vec<Token> = Vec::new();

        // 2. Parse arguments if the parenthesis is not immediately closed.
        if self.current_token() != Some(&Token::RParen) {
            loop {
                // Evaluate the expression for the current argument.
                let arg_value = self.parse_expr()?;
                args.push(arg_value);

                // If there is a comma, consume it and continue; otherwise, break.
                if let Some(Token::Comma) = self.current_token() {
                    self.eat(Token::Comma)?;
                } else {
                    break;
                }
            }
        }

        // 3. Consume the closing parenthesis ')'.
        self.eat(Token::RParen)?;

        // 4. Retrieve the function definition from the registry.
        // If not found, return an UndeclaredVariable error.
        let func = match self.functions.get(name) {
            Some(f) => f.clone(),
            None => return Err(ParseError::UndeclaredVariable { name: name.to_string() }),
        };

        // 5. Prepare the local scope for the function execution.
        let mut func_scope = HashMap::new();

        // Map the passed arguments to the function's parameter names.
        for (param_name, arg_value) in func.params.iter().zip(args.into_iter()) {
            func_scope.insert(param_name.clone(), arg_value);
        }

        // 6. Create a temporary Parser instance to execute the function body.
        // This ensures complete isolation and proper execution flow.
        let mut func_parser = Parser::new(func.body);

        // Inject scopes: 
        // We pass the Global Scope (index 0) so the function can access global variables,
        // and the Local Scope we just created for parameters.
        if let Some(global_scope) = self.scopes.first() {
            func_parser.scopes = vec![global_scope.clone(), func_scope];
        } else {
            func_parser.scopes = vec![func_scope];
        }

        // Pass the function registry to the new parser.
        // This enables the function to call other functions or itself (recursion).
        func_parser.functions = self.functions.clone();

        // Default return value is void (represented as boolean true).
        let mut return_val = Token::Boolean(true);

        // 7. Execute the function body tokens.
        while func_parser.current_token().is_some() {
            match func_parser.sense() {
                Ok(_) => { /* Continue execution normally */ }
                
                // Capture the 'Return' signal.
                // We treat Return as a special error type to break the execution loop immediately.
                Err(ParseError::Return { value }) => {
                    return_val = value;
                    break;
                }
                
                // Propagate any other actual parsing errors.
                Err(e) => return Err(e),
            }
        }

        // Return the final value of the function execution.
        Ok(return_val)
    }

    /// The main dispatch function. It "senses" what the current token is
    /// and dispatches to the correct parsing function for that statement.
    pub fn sense(&mut self) -> Result<(), ParseError> {
        if let Some(tok) = self.current_token() {
                match tok {
                    // Statement starts with 'let'.
                    Token::Let => self.parse_let(),
                    // Statement starts with 'print'.
                    Token::Print => self.parse_print(),
                    // Statement starts with 'if'.
                    Token::If => self.parse_if(),
                    // Statement starts with 'while'.
                    Token::While => self.parse_while(),
                    Token::For => self.parse_for(),
                    Token::Func => self.parse_fn(),
                    Token::Return => self.parse_return(),
                    // Statement starts with an identifier.
                    Token::Identifier(name) => {
                        let name_clone = name.clone();
                        // We must "peek" to see what comes next.
                        if let Some(Token::Assign) = self.peek() {
                            // If it's 'Identifier' followed by '=', it's an assignment.
                            self.parse_assigment(); // Typo: parse_assignment
                            Ok(())
                        } 
                        else if let Some(Token::LParen) = self.peek() {
                            self.parse_function_call(&name_clone)?;
                            Ok(())
                        } 
                        else {
                            // An identifier by itself is not a valid statement.
                            Err(ParseError::UnkownType { type_name: format!("Unexpected token after identifier: {:?}", self.peek()) })
                        }
                    }
                    // Any other token is an unknown/invalid start to a statement.
                    _ => {
                        let name = format!("{:?}", tok);
                        self.output.push("unknown".to_string());
                        Err(ParseError::UnkownType { type_name: name })
                    }
                }
        } else {
            // No more tokens, end of file. This is successful.
            Ok(())
        }
    }
    
}