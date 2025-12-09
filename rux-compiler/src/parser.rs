use crate::ast::*;
use crate::errors::{Error, Result};
use crate::lexer::{Token, TokenWithSpan};

pub struct Parser {
    tokens: Vec<TokenWithSpan>,
    current: usize,
    source: String,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithSpan>, source: String) -> Self {
        Self {
            tokens,
            current: 0,
            source,
        }
    }
    
    pub fn parse(&mut self) -> Result<AST> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        
        Ok(AST { items })
    }
    
    fn parse_item(&mut self) -> Result<Item> {
        if self.check(&Token::Fn) {
            self.parse_function_or_component()
        } else if self.check(&Token::Struct) {
            Ok(Item::Struct(self.parse_struct()?))
        } else if self.check(&Token::Enum) {
            Ok(Item::Enum(self.parse_enum()?))
        } else if self.check(&Token::Trait) {
            Ok(Item::Trait(self.parse_trait()?))
        } else if self.check(&Token::Impl) {
            Ok(Item::Impl(self.parse_impl()?))
        } else if self.check(&Token::Use) {
            Ok(Item::Use(self.parse_use()?))
        } else if self.check(&Token::Mod) {
            Ok(Item::Mod(self.parse_mod()?))
        } else if self.check(&Token::Type) {
            Ok(Item::TypeAlias(self.parse_type_alias()?))
        } else {
            Err(self.error("Expected item (fn, struct, enum, etc.)"))
        }
    }
    
    fn parse_function_or_component(&mut self) -> Result<Item> {
        let start_span = self.previous().span;
        self.advance(); // consume 'fn'
        
        let name = self.parse_identifier()?;
        let params = self.parse_params()?;
        
        // Check if this is a component (returns Element) or regular function
        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        if self.check(&Token::LBrace) {
            // Block body
            let block = self.parse_block()?;
            if let Some(ref ret_type) = return_type {
                // Check if return type is Element (component)
                if self.is_element_type(ret_type) {
                    return Ok(Item::Component(Component {
                        name,
                        props: params,
                        return_type: ret_type.clone(),
                        body: Expr::Block(block, start_span),
                        span: start_span,
                    }));
                }
            }
            Ok(Item::Function(Function {
                name,
                params,
                return_type,
                body: block,
                span: start_span,
            }))
        } else {
            // Expression body (for components)
            if let Some(ref ret_type) = return_type {
                if self.is_element_type(ret_type) {
                    let expr = self.parse_expression()?;
                    return Ok(Item::Component(Component {
                        name,
                        props: params,
                        return_type: ret_type.clone(),
                        body: expr,
                        span: start_span,
                    }));
                }
            }
            Err(self.error("Function body expected"))
        }
    }
    
    fn is_element_type(&self, ty: &Type) -> bool {
        matches!(&ty.kind, TypeKind::Ident(name) if name == "Element")
    }
    
    fn parse_params(&mut self) -> Result<Vec<Param>> {
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        
        if !self.check(&Token::RParen) {
            loop {
                let name = self.parse_identifier()?;
                self.expect(&Token::Colon)?;
                let param_type = self.parse_type()?;
                let span = self.previous().span;
                
                params.push(Param {
                    name,
                    param_type,
                    span,
                });
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.expect(&Token::RParen)?;
        Ok(params)
    }
    
    fn parse_type(&mut self) -> Result<Type> {
        let span = self.peek().span;
        let kind = if self.match_token(&Token::LParen) {
            // Tuple type
            let mut types = Vec::new();
            if !self.check(&Token::RParen) {
                loop {
                    types.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.expect(&Token::RParen)?;
            TypeKind::Tuple(types)
        } else if self.match_token(&Token::LBracket) {
            // Array or slice type
            if self.match_token(&Token::RBracket) {
                // Slice type
                TypeKind::Slice(Box::new(self.parse_type()?))
            } else {
                let inner = self.parse_type()?;
                self.expect(&Token::Semicolon)?;
                let size = self.parse_expression()?;
                self.expect(&Token::RBracket)?;
                TypeKind::Array(Box::new(inner))
            }
        } else if self.match_token(&Token::And) {
            // Reference type
            let mutable = self.match_token(&Token::Mut);
            let inner = self.parse_type()?;
            TypeKind::Reference {
                mutable,
                inner: Box::new(inner),
            }
        } else if matches!(self.peek().token, Token::Ident(_)) {
            let name = self.parse_identifier()?;
            if self.match_token(&Token::ColonColon) {
                // Path type
                let mut path = vec![name];
                loop {
                    path.push(self.parse_identifier()?);
                    if !self.match_token(&Token::ColonColon) {
                        break;
                    }
                }
                TypeKind::Path(path)
            } else {
                TypeKind::Ident(name)
            }
        } else {
            return Err(self.error("Expected type"));
        };
        
        Ok(Type { kind, span })
    }
    
    fn parse_block(&mut self) -> Result<Block> {
        let span = self.expect(&Token::LBrace)?.span;
        let mut statements = Vec::new();
        
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Block { statements, span })
    }
    
    fn parse_statement(&mut self) -> Result<Stmt> {
        if self.match_token(&Token::Let) {
            let mutable = self.match_token(&Token::Mut);
            let name = self.parse_identifier()?;
            self.expect(&Token::Eq)?;
            let value = self.parse_expression()?;
            let span = self.previous().span;
            self.expect(&Token::Semicolon)?;
            Ok(Stmt::Let {
                name,
                value,
                mutable,
                span,
            })
        } else if self.match_token(&Token::Return) {
            let span = self.previous().span;
            let value = if !self.check(&Token::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.expect(&Token::Semicolon)?;
            Ok(Stmt::Return(value, span))
        } else if self.match_token(&Token::If) {
            self.parse_if_statement()
        } else if self.match_token(&Token::For) {
            self.parse_for_statement()
        } else if self.match_token(&Token::While) {
            self.parse_while_statement()
        } else if self.match_token(&Token::Match) {
            self.parse_match_statement()
        } else if self.check(&Token::LBrace) {
            Ok(Stmt::Block(self.parse_block()?))
        } else {
            let expr = self.parse_expression()?;
            self.expect(&Token::Semicolon)?;
            Ok(Stmt::Expr(expr))
        }
    }
    
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }
    
    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_or()?;
        // Assignment parsing would go here
        Ok(expr)
    }
    
    fn parse_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&Token::Or) {
            let op = BinaryOp::Or;
            let right = self.parse_and()?;
            let span = self.previous().span;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&Token::And) {
            let op = BinaryOp::And;
            let right = self.parse_equality()?;
            let span = self.previous().span;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        
        while self.match_token(&Token::EqEq) || self.match_token(&Token::Ne) {
            let op = match self.previous().token {
                Token::EqEq => BinaryOp::Eq,
                Token::Ne => BinaryOp::Ne,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let span = self.previous().span;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;
        
        while self.match_token(&Token::Gt)
            || self.match_token(&Token::Ge)
            || self.match_token(&Token::Lt)
            || self.match_token(&Token::Le)
        {
            let op = match self.previous().token {
                Token::Gt => BinaryOp::Gt,
                Token::Ge => BinaryOp::Ge,
                Token::Lt => BinaryOp::Lt,
                Token::Le => BinaryOp::Le,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            let span = self.previous().span;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;
        
        while self.match_token(&Token::Plus) || self.match_token(&Token::Minus) {
            let op = match self.previous().token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            let span = self.previous().span;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        
        while self.match_token(&Token::Star) || self.match_token(&Token::Slash) {
            let op = match self.previous().token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let span = self.previous().span;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_token(&Token::Not) {
            let op = UnaryOp::Not;
            let expr = self.parse_unary()?;
            let span = self.previous().span;
            Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span,
            })
        } else if self.match_token(&Token::Minus) {
            let op = UnaryOp::Neg;
            let expr = self.parse_unary()?;
            let span = self.previous().span;
            Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span,
            })
        } else {
            self.parse_call()
        }
    }
    
    fn parse_call(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&Token::LParen) {
                let mut args = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                let span = self.expect(&Token::RParen)?.span;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            } else if self.match_token(&Token::Dot) {
                let field = self.parse_identifier()?;
                let span = self.previous().span;
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field,
                    span,
                };
            } else if self.match_token(&Token::LBracket) {
                let index = self.parse_expression()?;
                let span = self.expect(&Token::RBracket)?.span;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expr> {
        if self.match_token(&Token::JSXOpen) {
            self.parse_jsx_element()
        } else if self.match_token(&Token::LParen) {
            let expr = self.parse_expression()?;
            self.expect(&Token::RParen)?;
            Ok(expr)
        } else if let Token::Ident(name) = &self.peek().token {
            let name = name.clone();
            let span = self.advance().span;
            Ok(Expr::Variable(name, span))
        } else if let Token::String(s) = &self.peek().token {
            let s = s.clone();
            let span = self.advance().span;
            Ok(Expr::Literal(Literal::String(s), span))
        } else if let Token::Number(n) = &self.peek().token {
            let n = *n;
            let span = self.advance().span;
            Ok(Expr::Literal(Literal::Number(n), span))
        } else if let Token::Boolean(b) = &self.peek().token {
            let b = *b;
            let span = self.advance().span;
            Ok(Expr::Literal(Literal::Boolean(b), span))
        } else if let Token::Char(c) = &self.peek().token {
            let c = *c;
            let span = self.advance().span;
            Ok(Expr::Literal(Literal::Char(c), span))
        } else {
            Err(self.error("Expected expression"))
        }
    }
    
    fn parse_jsx_element(&mut self) -> Result<Expr> {
        let start_span = self.previous().span;
        
        if let Token::JSXOpenTag(tag) = &self.peek().token {
            let tag = tag.clone();
            self.advance();
            
            let mut props = Vec::new();
            while !self.check(&Token::JSXSlash) && !self.check(&Token::JSXClose) {
                let name = self.parse_identifier()?;
                let prop_span = self.previous().span;
                
                if self.match_token(&Token::Eq) {
                    let value = if self.check(&Token::LBrace) {
                        self.advance();
                        let expr = self.parse_expression()?;
                        self.expect(&Token::RBrace)?;
                        JSXPropValue::Expr(expr)
                    } else if let Token::String(s) = &self.peek().token {
                        let s = s.clone();
                        self.advance();
                        JSXPropValue::Literal(Literal::String(s))
                    } else if let Token::Boolean(b) = &self.peek().token {
                        let b = *b;
                        self.advance();
                        JSXPropValue::Bool(b)
                    } else {
                        return Err(self.error("Expected JSX prop value"));
                    };
                    
                    props.push(JSXProp {
                        name,
                        value,
                        span: prop_span,
                    });
                } else {
                    // Boolean prop (shorthand)
                    props.push(JSXProp {
                        name,
                        value: JSXPropValue::Bool(true),
                        span: prop_span,
                    });
                }
            }
            
            if self.match_token(&Token::JSXSelfClose) {
                return Ok(Expr::JSXElement(
                JSXElement::SelfClosing {
                    tag,
                    props,
                    span: start_span,
                },
                    start_span,
                ));
            }
            
            self.expect(&Token::JSXClose)?;
            
            let mut children = Vec::new();
            loop {
                if self.is_at_end() {
                    break;
                }
                if self.check(&Token::JSXOpen) {
                    // Check if this is a closing tag
                    if self.current + 1 < self.tokens.len() {
                        if let Token::JSXCloseTag(_) = &self.tokens[self.current + 1].token {
                            break;
                        }
                    }
                    children.push(JSXChild::Element(self.parse_jsx_element_inner()?));
                } else if let Token::String(s) = &self.peek().token {
                    let s = s.clone();
                    let span = self.advance().span;
                    children.push(JSXChild::Text(s, span));
                } else if self.check(&Token::JSXClose) {
                    // Check if next is closing tag
                    if self.current + 1 < self.tokens.len() {
                        if let Token::JSXCloseTag(_) = &self.tokens[self.current + 1].token {
                            break;
                        }
                    }
                    children.push(JSXChild::Expr(self.parse_expression()?));
                } else {
                    children.push(JSXChild::Expr(self.parse_expression()?));
                }
            }
            
            if let Token::JSXCloseTag(expected_tag) = &self.peek().token {
                if expected_tag == &tag {
                    self.advance();
                    self.expect(&Token::JSXClose)?;
                } else {
                    return Err(self.error(&format!("Expected closing tag </{}>", tag)));
                }
            } else {
                return Err(self.error("Expected closing tag"));
            }
            
            Ok(Expr::JSXElement(
                JSXElement::WithChildren {
                    tag,
                    props,
                    children,
                    span: start_span,
                },
                start_span,
            ))
        } else {
            Err(self.error("Expected JSX tag"))
        }
    }
    
    fn parse_jsx_element_inner(&mut self) -> Result<JSXElement> {
        // Simplified - would need full implementation
        self.parse_jsx_element().map(|e| {
            if let Expr::JSXElement(jsx, _) = e {
                jsx
            } else {
                unreachable!()
            }
        })
    }
    
    fn parse_if_statement(&mut self) -> Result<Stmt> {
        let span = self.previous().span;
        let condition = self.parse_expression()?;
        let then = Box::new(self.parse_statement()?);
        let else_ = if self.match_token(&Token::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then,
            else_,
            span,
        })
    }
    
    fn parse_for_statement(&mut self) -> Result<Stmt> {
        let span = self.previous().span;
        let var = self.parse_identifier()?;
        self.expect(&Token::In)?;
        let iter = self.parse_expression()?;
        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::For {
            var,
            iter,
            body,
            span,
        })
    }
    
    fn parse_while_statement(&mut self) -> Result<Stmt> {
        let span = self.previous().span;
        let condition = self.parse_expression()?;
        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }
    
    fn parse_match_statement(&mut self) -> Result<Stmt> {
        let span = self.previous().span;
        let expr = self.parse_expression()?;
        self.expect(&Token::LBrace)?;
        let mut arms = Vec::new();
        
        while !self.check(&Token::RBrace) {
            arms.push(self.parse_match_arm()?);
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Stmt::Match {
            expr,
            arms,
            span,
        })
    }
    
    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let pattern = self.parse_pattern()?;
        let guard = if self.match_token(&Token::If) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(&Token::FatArrow)?;
        let body = self.parse_expression()?;
        let span = self.previous().span;
        if self.match_token(&Token::Comma) {
            // Optional comma
        }
        Ok(MatchArm {
            pattern,
            guard,
            body,
            span,
        })
    }
    
    fn parse_pattern(&mut self) -> Result<Pattern> {
        let span = self.peek().span;
        if let Token::Ident(name) = &self.peek().token {
            let name = name.clone();
            self.advance();
            Ok(Pattern::Ident(name, span))
        } else if let Token::Underscore = &self.peek().token {
            self.advance();
            Ok(Pattern::Wildcard(span))
        } else {
            Err(self.error("Expected pattern"))
        }
    }
    
    fn parse_struct(&mut self) -> Result<Struct> {
        let span = self.expect(&Token::Struct)?.span;
        let name = self.parse_identifier()?;
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        
        while !self.check(&Token::RBrace) {
            let field_name = self.parse_identifier()?;
            self.expect(&Token::Colon)?;
            let field_type = self.parse_type()?;
            let field_span = self.previous().span;
            fields.push(StructField {
                name: field_name,
                field_type,
                span: field_span,
            });
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Struct { name, fields, span })
    }
    
    fn parse_enum(&mut self) -> Result<Enum> {
        let span = self.expect(&Token::Enum)?.span;
        let name = self.parse_identifier()?;
        self.expect(&Token::LBrace)?;
        let mut variants = Vec::new();
        
        while !self.check(&Token::RBrace) {
            let variant_name = self.parse_identifier()?;
            let variant_span = self.previous().span;
            let data = if self.match_token(&Token::LParen) {
                // Tuple variant
                let mut types = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&Token::RParen)?;
                Some(EnumVariantData::Tuple(types))
            } else if self.match_token(&Token::LBrace) {
                // Struct variant
                let mut fields = Vec::new();
                while !self.check(&Token::RBrace) {
                    let field_name = self.parse_identifier()?;
                    self.expect(&Token::Colon)?;
                    let field_type = self.parse_type()?;
                    let field_span = self.previous().span;
                    fields.push(StructField {
                        name: field_name,
                        field_type,
                        span: field_span,
                    });
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RBrace)?;
                Some(EnumVariantData::Struct(fields))
            } else {
                None
            };
            
            variants.push(EnumVariant {
                name: variant_name,
                data,
                span: variant_span,
            });
            
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Enum {
            name,
            variants,
            span,
        })
    }
    
    fn parse_trait(&mut self) -> Result<Trait> {
        let span = self.expect(&Token::Trait)?.span;
        let name = self.parse_identifier()?;
        self.expect(&Token::LBrace)?;
        let mut items = Vec::new();
        
        while !self.check(&Token::RBrace) {
            if self.check(&Token::Fn) {
                items.push(TraitItem::Method(self.parse_function_inner()?));
            } else if self.match_token(&Token::Type) {
                let name = self.parse_identifier()?;
                let ty = if self.match_token(&Token::Eq) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&Token::Semicolon)?;
                items.push(TraitItem::Type(name, ty));
            } else {
                return Err(self.error("Expected trait item"));
            }
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Trait { name, items, span })
    }
    
    fn parse_impl(&mut self) -> Result<Impl> {
        let span = self.expect(&Token::Impl)?.span;
        let trait_name = if matches!(self.peek().token, Token::Ident(_)) {
            let name = self.parse_identifier()?;
            if self.match_token(&Token::For) {
                Some(name)
            } else {
                // Not a trait impl, reset
                self.current -= 1;
                None
            }
        } else {
            None
        };
        let type_name = self.parse_identifier()?;
        self.expect(&Token::LBrace)?;
        let mut items = Vec::new();
        
        while !self.check(&Token::RBrace) {
            items.push(self.parse_function_inner()?);
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Impl {
            trait_name,
            type_name,
            items,
            span,
        })
    }
    
    fn parse_function_inner(&mut self) -> Result<Function> {
        let span = self.expect(&Token::Fn)?.span;
        let name = self.parse_identifier()?;
        let params = self.parse_params()?;
        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Function {
            name,
            params,
            return_type,
            body,
            span,
        })
    }
    
    fn parse_use(&mut self) -> Result<Use> {
        let span = self.expect(&Token::Use)?.span;
        let mut path = Vec::new();
        path.push(self.parse_identifier()?);
        
        while self.match_token(&Token::ColonColon) {
            path.push(self.parse_identifier()?);
        }
        
        let alias = if self.match_token(&Token::As) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        self.expect(&Token::Semicolon)?;
        Ok(Use { path, alias, span })
    }
    
    fn parse_mod(&mut self) -> Result<Mod> {
        let span = self.expect(&Token::Mod)?.span;
        let name = self.parse_identifier()?;
        self.expect(&Token::LBrace)?;
        let mut items = Vec::new();
        
        while !self.check(&Token::RBrace) {
            items.push(self.parse_item()?);
        }
        
        self.expect(&Token::RBrace)?;
        Ok(Mod { name, items, span })
    }
    
    fn parse_type_alias(&mut self) -> Result<TypeAlias> {
        let span = self.expect(&Token::Type)?.span;
        let name = self.parse_identifier()?;
        self.expect(&Token::Eq)?;
        let aliased_type = self.parse_type()?;
        self.expect(&Token::Semicolon)?;
        Ok(TypeAlias {
            name,
            aliased_type,
            span,
        })
    }
    
    fn parse_identifier(&mut self) -> Result<String> {
        if let Token::Ident(name) = &self.peek().token {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error("Expected identifier"))
        }
    }
    
    // Helper methods
    fn is_at_end(&self) -> bool {
        matches!(self.peek().token, Token::Eof)
    }
    
    fn peek(&self) -> &TokenWithSpan {
        &self.tokens[self.current]
    }
    
    fn advance(&mut self) -> &TokenWithSpan {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn previous(&self) -> &TokenWithSpan {
        &self.tokens[self.current - 1]
    }
    
    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            match (&self.peek().token, token) {
                (Token::Ident(_), Token::Ident(_)) => true,
                (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
            }
        }
    }
    
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn expect(&mut self, token: &Token) -> Result<&TokenWithSpan> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(self.error(&format!("Expected {:?}", token)))
        }
    }
    
    fn error(&self, message: &str) -> Error {
        let span = if self.current < self.tokens.len() {
            self.tokens[self.current].span
        } else if !self.tokens.is_empty() {
            self.tokens[self.tokens.len() - 1].span
        } else {
            crate::lexer::Span::new(0, 0, 1, 1)
        };
        
        Error::parser(
            message,
            self.source.clone(),
            span.to_source_span(),
        )
    }
}
