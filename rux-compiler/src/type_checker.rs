use crate::ast::*;
use crate::errors::{Error, Result};
use crate::lexer::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    bindings: HashMap<String, Type>,
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(parent: TypeEnvironment) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    pub fn bind(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, ty);
    }
    
    pub fn lookup(&self, name: &str) -> Option<Type> {
        if let Some(ty) = self.bindings.get(name) {
            Some(ty.clone())
        } else if let Some(ref parent) = self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

pub struct TypeChecker {
    env: TypeEnvironment,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::new(),
        }
    }
    
    pub fn check(&mut self, ast: &AST) -> Result<()> {
        for item in &ast.items {
            self.check_item(item)?;
        }
        Ok(())
    }
    
    fn check_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Component(component) => self.check_component(component),
            Item::Function(function) => self.check_function(function),
            Item::Struct(struct_def) => self.check_struct(struct_def),
            Item::Enum(enum_def) => self.check_enum(enum_def),
            Item::Trait(trait_def) => self.check_trait(trait_def),
            Item::Impl(impl_def) => self.check_impl(impl_def),
            Item::Use(_) => Ok(()), // Use statements don't need type checking
            Item::Mod(mod_def) => {
                let mut new_env = TypeEnvironment::with_parent(self.env.clone());
                let old_env = std::mem::replace(&mut self.env, new_env);
                let result = self.check_mod(mod_def);
                self.env = old_env;
                result
            }
            Item::TypeAlias(alias) => self.check_type_alias(alias),
        }
    }
    
    fn check_component(&mut self, component: &Component) -> Result<()> {
        // Create new scope for component parameters
        let mut param_env = TypeEnvironment::with_parent(self.env.clone());
        
        // Add parameters to environment
        for param in &component.props {
            param_env.bind(param.name.clone(), param.param_type.clone());
        }
        
        // Check that return type is Element
        if !self.is_element_type(&component.return_type) {
            return Err(Error::type_error(
                format!(
                    "Component '{}' must return Element, found {:?}",
                    component.name, component.return_type.kind
                ),
                String::new(), // Would need source code
                component.return_type.span.to_source_span(),
            ));
        }
        
        // Check body expression
        let old_env = std::mem::replace(&mut self.env, param_env);
        let body_type = self.check_expression(&component.body)?;
        self.env = old_env;
        
        // Verify body type matches return type
        if !self.types_match(&body_type, &component.return_type) {
            return Err(Error::type_error(
                format!(
                    "Component '{}' body type mismatch: expected Element, found {:?}",
                    component.name, body_type.kind
                ),
                String::new(),
                component.body.span().to_source_span(),
            ));
        }
        
        Ok(())
    }
    
    fn check_function(&mut self, function: &Function) -> Result<()> {
        // Create new scope for function parameters
        let mut param_env = TypeEnvironment::with_parent(self.env.clone());
        
        for param in &function.params {
            param_env.bind(param.name.clone(), param.param_type.clone());
        }
        
        // Check function body
        let old_env = std::mem::replace(&mut self.env, param_env);
        self.check_block(&function.body)?;
        self.env = old_env;
        
        Ok(())
    }
    
    fn check_struct(&mut self, struct_def: &Struct) -> Result<()> {
        // Struct definitions don't need type checking beyond syntax
        Ok(())
    }
    
    fn check_enum(&mut self, enum_def: &Enum) -> Result<()> {
        // Enum definitions don't need type checking beyond syntax
        Ok(())
    }
    
    fn check_trait(&mut self, trait_def: &Trait) -> Result<()> {
        // Trait definitions don't need type checking beyond syntax
        Ok(())
    }
    
    fn check_impl(&mut self, impl_def: &Impl) -> Result<()> {
        // Check implementation methods
        for function in &impl_def.items {
            self.check_function(function)?;
        }
        Ok(())
    }
    
    fn check_mod(&mut self, mod_def: &Mod) -> Result<()> {
        for item in &mod_def.items {
            self.check_item(item)?;
        }
        Ok(())
    }
    
    fn check_type_alias(&mut self, alias: &TypeAlias) -> Result<()> {
        // Type aliases don't need type checking beyond syntax
        Ok(())
    }
    
    fn check_block(&mut self, block: &Block) -> Result<()> {
        for stmt in &block.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }
    
    fn check_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let value_type = self.check_expression(value)?;
                self.env.bind(name.clone(), value_type);
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            Stmt::Return(Some(expr), _) => {
                self.check_expression(expr)?;
                Ok(())
            }
            Stmt::Return(None, _) => Ok(()),
            Stmt::If { condition, then, else_, .. } => {
                let cond_type = self.check_expression(condition)?;
                if !self.is_bool_type(&cond_type) {
                    return Err(Error::type_error(
                        "If condition must be boolean",
                        String::new(),
                        condition.span().to_source_span(),
                    ));
                }
                self.check_statement(then)?;
                if let Some(else_stmt) = else_ {
                    self.check_statement(else_stmt)?;
                }
                Ok(())
            }
            Stmt::For { iter, body, .. } => {
                let iter_type = self.check_expression(iter)?;
                // Check that iter is iterable (simplified)
                self.check_statement(body)?;
                Ok(())
            }
            Stmt::While { condition, body, .. } => {
                let cond_type = self.check_expression(condition)?;
                if !self.is_bool_type(&cond_type) {
                    return Err(Error::type_error(
                        "While condition must be boolean",
                        String::new(),
                        condition.span().to_source_span(),
                    ));
                }
                self.check_statement(body)?;
                Ok(())
            }
            Stmt::Match { expr, arms, .. } => {
                let expr_type = self.check_expression(expr)?;
                for arm in arms {
                    self.check_match_arm(arm, &expr_type)?;
                }
                Ok(())
            }
            Stmt::Block(block) => self.check_block(block),
        }
    }
    
    fn check_expression(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Literal(lit, span) => Ok(self.type_of_literal(lit, *span)),
            Expr::Variable(name, span) => {
                self.env.lookup(name).ok_or_else(|| Error::type_error(
                    format!("Undefined variable: {}", name),
                    String::new(),
                    span.to_source_span(),
                ))
            }
            Expr::Binary { left, op, right, span } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;
                self.type_of_binary_op(op, &left_type, &right_type, *span)
            }
            Expr::Unary { op, expr, span } => {
                let expr_type = self.check_expression(expr)?;
                self.type_of_unary_op(op, &expr_type, *span)
            }
            Expr::Call { callee, args, span } => {
                let callee_type = self.check_expression(callee)?;
                let arg_types: Result<Vec<Type>> = args.iter().map(|a| self.check_expression(a)).collect();
                let arg_types = arg_types?;
                self.type_of_call(&callee_type, &arg_types, *span)
            }
            Expr::MethodCall { receiver, method, args, span } => {
                let receiver_type = self.check_expression(receiver)?;
                let arg_types: Result<Vec<Type>> = args.iter().map(|a| self.check_expression(a)).collect();
                let arg_types = arg_types?;
                self.type_of_method_call(&receiver_type, method, &arg_types, *span)
            }
            Expr::FieldAccess { object, field, span } => {
                let object_type = self.check_expression(object)?;
                self.type_of_field_access(&object_type, field, *span)
            }
            Expr::Index { object, index, span } => {
                let object_type = self.check_expression(object)?;
                let index_type = self.check_expression(index)?;
                self.type_of_index(&object_type, &index_type, *span)
            }
            Expr::JSXElement(jsx, span) => {
                // JSX elements always return Element type
                Ok(Type {
                    kind: TypeKind::Ident("Element".to_string()),
                    span: *span,
                })
            }
            Expr::Block(block, span) => {
                self.check_block(block)?;
                // Block returns unit type unless last expression
                Ok(Type {
                    kind: TypeKind::Unit,
                    span: *span,
                })
            }
            Expr::If { condition, then, else_, span } => {
                let cond_type = self.check_expression(condition)?;
                if !self.is_bool_type(&cond_type) {
                    return Err(Error::type_error(
                        "If condition must be boolean",
                        String::new(),
                        condition.span().to_source_span(),
                    ));
                }
                let then_type = self.check_expression(then)?;
                if let Some(else_expr) = else_ {
                    let else_type = self.check_expression(else_expr)?;
                    if !self.types_match(&then_type, &else_type) {
                        return Err(Error::type_error(
                            "If/else branches must have matching types",
                            String::new(),
                            span.to_source_span(),
                        ));
                    }
                }
                Ok(then_type)
            }
            Expr::Match { expr, arms, span } => {
                let expr_type = self.check_expression(expr)?;
                let mut arm_types = Vec::new();
                for arm in arms {
                    let arm_type = self.check_match_arm_expr(arm, &expr_type)?;
                    arm_types.push(arm_type);
                }
                // All arms must have the same type
                if let Some(first_type) = arm_types.first() {
                    for arm_type in &arm_types[1..] {
                        if !self.types_match(first_type, arm_type) {
                            return Err(Error::type_error(
                                "Match arms must have matching types",
                                String::new(),
                                span.to_source_span(),
                            ));
                        }
                    }
                    Ok(first_type.clone())
                } else {
                    Err(Error::type_error(
                        "Match expression must have at least one arm",
                        String::new(),
                        span.to_source_span(),
                    ))
                }
            }
            Expr::Lambda { params, body, span } => {
                let mut lambda_env = TypeEnvironment::with_parent(self.env.clone());
                for param in params {
                    lambda_env.bind(param.name.clone(), param.param_type.clone());
                }
                let old_env = std::mem::replace(&mut self.env, lambda_env);
                let body_type = self.check_expression(body)?;
                self.env = old_env;
                
                let param_types: Vec<Type> = params.iter().map(|p| p.param_type.clone()).collect();
                Ok(Type {
                    kind: TypeKind::Function {
                        params: param_types,
                        return_type: Box::new(body_type),
                    },
                    span: *span,
                })
            }
            Expr::Tuple(exprs, span) => {
                let types: Result<Vec<Type>> = exprs.iter().map(|e| self.check_expression(e)).collect();
                Ok(Type {
                    kind: TypeKind::Tuple(types?),
                    span: *span,
                })
            }
            Expr::Array(exprs, span) => {
                if exprs.is_empty() {
                    return Err(Error::type_error(
                        "Cannot infer type of empty array",
                        String::new(),
                        span.to_source_span(),
                    ));
                }
                let first_type = self.check_expression(&exprs[0])?;
                for expr in &exprs[1..] {
                    let expr_type = self.check_expression(expr)?;
                    if !self.types_match(&first_type, &expr_type) {
                        return Err(Error::type_error(
                            "Array elements must have matching types",
                            String::new(),
                            span.to_source_span(),
                        ));
                    }
                }
                Ok(Type {
                    kind: TypeKind::Array(Box::new(first_type)),
                    span: *span,
                })
            }
            Expr::Struct { name, fields, span } => {
                // Check struct fields
                for (field_name, field_expr) in fields {
                    self.check_expression(field_expr)?;
                }
                Ok(Type {
                    kind: TypeKind::Ident(name.clone()),
                    span: *span,
                })
            }
        }
    }
    
    fn check_match_arm(&mut self, arm: &MatchArm, expr_type: &Type) -> Result<()> {
        // Simplified pattern matching type checking
        self.check_expression(&arm.body)?;
        Ok(())
    }
    
    fn check_match_arm_expr(&mut self, arm: &MatchArm, _expr_type: &Type) -> Result<Type> {
        self.check_expression(&arm.body)
    }
    
    fn type_of_literal(&self, lit: &Literal, span: Span) -> Type {
        let kind = match lit {
            Literal::String(_) => TypeKind::Ident("String".to_string()),
            Literal::Number(_) => TypeKind::Ident("f64".to_string()),
            Literal::Boolean(_) => TypeKind::Ident("bool".to_string()),
            Literal::Char(_) => TypeKind::Ident("char".to_string()),
            Literal::Unit => TypeKind::Unit,
        };
        Type { kind, span }
    }
    
    fn type_of_binary_op(
        &self,
        op: &BinaryOp,
        left: &Type,
        right: &Type,
        span: Span,
    ) -> Result<Type> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
                // Numeric operations
                if self.is_numeric_type(left) && self.is_numeric_type(right) {
                    Ok(left.clone()) // Return left type (simplified)
                } else {
                    Err(Error::type_error(
                        format!("Binary operator {:?} requires numeric types", op),
                        String::new(),
                        span.to_source_span(),
                    ))
                }
            }
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                // Comparison operations return bool
                Ok(Type {
                    kind: TypeKind::Ident("bool".to_string()),
                    span,
                })
            }
            BinaryOp::And | BinaryOp::Or => {
                // Logical operations require bool
                if self.is_bool_type(left) && self.is_bool_type(right) {
                    Ok(Type {
                        kind: TypeKind::Ident("bool".to_string()),
                        span,
                    })
                } else {
                    Err(Error::type_error(
                        "Logical operators require boolean types",
                        String::new(),
                        span.to_source_span(),
                    ))
                }
            }
            _ => Err(Error::type_error(
                format!("Unsupported binary operator: {:?}", op),
                String::new(),
                span.to_source_span(),
            )),
        }
    }
    
    fn type_of_unary_op(&self, op: &UnaryOp, expr_type: &Type, span: Span) -> Result<Type> {
        match op {
            UnaryOp::Not => {
                if self.is_bool_type(expr_type) {
                    Ok(Type {
                        kind: TypeKind::Ident("bool".to_string()),
                        span,
                    })
                } else {
                    Err(Error::type_error(
                        "Not operator requires boolean type",
                        String::new(),
                        span.to_source_span(),
                    ))
                }
            }
            UnaryOp::Neg => {
                if self.is_numeric_type(expr_type) {
                    Ok(expr_type.clone())
                } else {
                    Err(Error::type_error(
                        "Negation requires numeric type",
                        String::new(),
                        span.to_source_span(),
                    ))
                }
            }
            _ => Err(Error::type_error(
                format!("Unsupported unary operator: {:?}", op),
                String::new(),
                span.to_source_span(),
            )),
        }
    }
    
    fn type_of_call(&self, callee_type: &Type, _arg_types: &[Type], span: Span) -> Result<Type> {
        match &callee_type.kind {
            TypeKind::Function { return_type, .. } => Ok(*return_type.clone()),
            _ => Err(Error::type_error(
                "Cannot call non-function type",
                String::new(),
                span.to_source_span(),
            )),
        }
    }
    
    fn type_of_method_call(
        &self,
        _receiver_type: &Type,
        _method: &str,
        _arg_types: &[Type],
        span: Span,
    ) -> Result<Type> {
        // Simplified - would need method resolution
        Err(Error::type_error(
            "Method calls not yet fully implemented",
            String::new(),
            span.to_source_span(),
        ))
    }
    
    fn type_of_field_access(&self, object_type: &Type, _field: &str, span: Span) -> Result<Type> {
        // Simplified - would need struct field lookup
        Err(Error::type_error(
            "Field access not yet fully implemented",
            String::new(),
            span.to_source_span(),
        ))
    }
    
    fn type_of_index(&self, object_type: &Type, _index_type: &Type, span: Span) -> Result<Type> {
        match &object_type.kind {
            TypeKind::Array(inner) | TypeKind::Slice(inner) => Ok(*inner.clone()),
            _ => Err(Error::type_error(
                "Index operation requires array or slice type",
                String::new(),
                span.to_source_span(),
            )),
        }
    }
    
    fn is_bool_type(&self, ty: &Type) -> bool {
        matches!(&ty.kind, TypeKind::Ident(name) if name == "bool")
    }
    
    fn is_numeric_type(&self, ty: &Type) -> bool {
        matches!(&ty.kind, TypeKind::Ident(name) if matches!(name.as_str(), "i32" | "i64" | "f32" | "f64" | "u32" | "u64"))
    }
    
    fn is_element_type(&self, ty: &Type) -> bool {
        matches!(&ty.kind, TypeKind::Ident(name) if name == "Element")
    }
    
    fn types_match(&self, t1: &Type, t2: &Type) -> bool {
        match (&t1.kind, &t2.kind) {
            (TypeKind::Ident(n1), TypeKind::Ident(n2)) => n1 == n2,
            (TypeKind::Unit, TypeKind::Unit) => true,
            (TypeKind::Tuple(t1), TypeKind::Tuple(t2)) => {
                t1.len() == t2.len() && t1.iter().zip(t2.iter()).all(|(a, b)| self.types_match(a, b))
            }
            (TypeKind::Array(a1), TypeKind::Array(a2)) => self.types_match(a1, a2),
            (TypeKind::Slice(s1), TypeKind::Slice(s2)) => self.types_match(s1, s2),
            _ => false,
        }
    }
}

// Helper trait for getting span from expressions
trait HasSpan {
    fn span(&self) -> Span;
}

impl HasSpan for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Variable(_, span) => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::JSXElement(_, span) => *span,
            Expr::Block(_, span) => *span,
            Expr::If { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::Tuple(_, span) => *span,
            Expr::Array(_, span) => *span,
            Expr::Struct { span, .. } => *span,
        }
    }
}
