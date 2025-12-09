use crate::ast::*;
use crate::errors::Result;
use std::collections::HashSet;

pub struct Optimizer;

impl Optimizer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn optimize(&self, ast: &mut AST) -> Result<()> {
        // Apply optimization passes
        self.dead_code_elimination(ast)?;
        self.constant_folding(ast)?;
        // Component inlining would go here
        Ok(())
    }
    
    fn dead_code_elimination(&self, ast: &mut AST) -> Result<()> {
        // Build symbol usage graph
        let mut used_symbols = HashSet::new();
        self.collect_used_symbols(ast, &mut used_symbols);
        
        // Remove unused items
        ast.items.retain(|item| {
            match item {
                Item::Function(f) => used_symbols.contains(&f.name),
                Item::Component(c) => used_symbols.contains(&c.name),
                Item::Struct(s) => used_symbols.contains(&s.name),
                Item::Enum(e) => used_symbols.contains(&e.name),
                _ => true, // Keep other items
            }
        });
        
        Ok(())
    }
    
    fn collect_used_symbols(&self, ast: &AST, used: &mut HashSet<String>) {
        for item in &ast.items {
            match item {
                Item::Component(c) => {
                    used.insert(c.name.clone());
                    self.collect_symbols_from_expr(&c.body, used);
                }
                Item::Function(f) => {
                    used.insert(f.name.clone());
                    self.collect_symbols_from_block(&f.body, used);
                }
                _ => {}
            }
        }
    }
    
    fn collect_symbols_from_expr(&self, expr: &Expr, used: &mut HashSet<String>) {
        match expr {
            Expr::Variable(name, _) => {
                used.insert(name.clone());
            }
            Expr::Call { callee, args, .. } => {
                self.collect_symbols_from_expr(callee, used);
                for arg in args {
                    self.collect_symbols_from_expr(arg, used);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_symbols_from_expr(left, used);
                self.collect_symbols_from_expr(right, used);
            }
            Expr::Unary { expr, .. } => {
                self.collect_symbols_from_expr(expr, used);
            }
            Expr::JSXElement(jsx, _) => {
                self.collect_symbols_from_jsx(jsx, used);
            }
            Expr::Block(block, _) => {
                self.collect_symbols_from_block(block, used);
            }
            Expr::If { condition, then, else_, .. } => {
                self.collect_symbols_from_expr(condition, used);
                self.collect_symbols_from_expr(then, used);
                if let Some(else_expr) = else_ {
                    self.collect_symbols_from_expr(else_expr, used);
                }
            }
            _ => {}
        }
    }
    
    fn collect_symbols_from_block(&self, block: &Block, used: &mut HashSet<String>) {
        for stmt in &block.statements {
            self.collect_symbols_from_stmt(stmt, used);
        }
    }
    
    fn collect_symbols_from_stmt(&self, stmt: &Stmt, used: &mut HashSet<String>) {
        match stmt {
            Stmt::Let { value, .. } => {
                self.collect_symbols_from_expr(value, used);
            }
            Stmt::Expr(expr) => {
                self.collect_symbols_from_expr(expr, used);
            }
            Stmt::Return(Some(expr), _) => {
                self.collect_symbols_from_expr(expr, used);
            }
            Stmt::Return(None, _) => {
                // No symbols for return without value
            }
            Stmt::If { condition, then, else_, .. } => {
                self.collect_symbols_from_expr(condition, used);
                self.collect_symbols_from_stmt(then, used);
                if let Some(else_stmt) = else_ {
                    self.collect_symbols_from_stmt(else_stmt, used);
                }
            }
            Stmt::For { iter, body, .. } => {
                self.collect_symbols_from_expr(iter, used);
                self.collect_symbols_from_stmt(body, used);
            }
            Stmt::While { condition, body, .. } => {
                self.collect_symbols_from_expr(condition, used);
                self.collect_symbols_from_stmt(body, used);
            }
            Stmt::Match { expr, arms, .. } => {
                self.collect_symbols_from_expr(expr, used);
                for arm in arms {
                    self.collect_symbols_from_expr(&arm.body, used);
                }
            }
            Stmt::Block(block) => {
                self.collect_symbols_from_block(block, used);
            }
        }
    }
    
    fn collect_symbols_from_jsx(&self, jsx: &JSXElement, used: &mut HashSet<String>) {
        match jsx {
            JSXElement::SelfClosing { props, .. } => {
                for prop in props {
                    self.collect_symbols_from_jsx_prop(prop, used);
                }
            }
            JSXElement::WithChildren { props, children, .. } => {
                for prop in props {
                    self.collect_symbols_from_jsx_prop(prop, used);
                }
                for child in children {
                    match child {
                        JSXChild::Element(elem) => self.collect_symbols_from_jsx(elem, used),
                        JSXChild::Expr(expr) => self.collect_symbols_from_expr(expr, used),
                        JSXChild::Text(_, _) => {}
                    }
                }
            }
        }
    }
    
    fn collect_symbols_from_jsx_prop(&self, prop: &JSXProp, used: &mut HashSet<String>) {
        match &prop.value {
            JSXPropValue::Expr(expr) => {
                self.collect_symbols_from_expr(expr, used);
            }
            _ => {}
        }
    }
    
    fn constant_folding(&self, ast: &mut AST) -> Result<()> {
        for item in &mut ast.items {
            match item {
                Item::Component(c) => {
                    self.fold_constants_in_expr(&mut c.body)?;
                }
                Item::Function(f) => {
                    self.fold_constants_in_block(&mut f.body)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn fold_constants_in_expr(&self, expr: &mut Expr) -> Result<()> {
        match expr {
            Expr::Binary { left, op, right, .. } => {
                self.fold_constants_in_expr(left)?;
                self.fold_constants_in_expr(right)?;
                
                if let (Expr::Literal(lit1, _), Expr::Literal(lit2, _)) = (&**left, &**right) {
                    if let Some(result) = self.evaluate_binary(lit1, op, lit2) {
                        *expr = result;
                    }
                }
            }
            Expr::Unary { expr: inner, op, .. } => {
                self.fold_constants_in_expr(inner)?;
                if let Expr::Literal(lit, span) = &**inner {
                    if let Some(result) = self.evaluate_unary(lit, op) {
                        *expr = result;
                    }
                }
            }
            Expr::Block(block, _) => {
                self.fold_constants_in_block(block)?;
            }
            Expr::If { condition, then, else_, .. } => {
                self.fold_constants_in_expr(condition)?;
                self.fold_constants_in_expr(then)?;
                if let Some(else_expr) = else_ {
                    self.fold_constants_in_expr(else_expr)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn fold_constants_in_block(&self, block: &mut Block) -> Result<()> {
        for stmt in &mut block.statements {
            self.fold_constants_in_stmt(stmt)?;
        }
        Ok(())
    }
    
    fn fold_constants_in_stmt(&self, stmt: &mut Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { value, .. } => {
                self.fold_constants_in_expr(value)?;
            }
            Stmt::Expr(expr) => {
                self.fold_constants_in_expr(expr)?;
            }
            Stmt::Return(Some(expr), _) => {
                self.fold_constants_in_expr(expr)?;
            }
            Stmt::Return(None, _) => {
                // No constants to fold for return without value
            }
            Stmt::If { condition, then, else_, .. } => {
                self.fold_constants_in_expr(condition)?;
                self.fold_constants_in_stmt(then)?;
                if let Some(else_stmt) = else_ {
                    self.fold_constants_in_stmt(else_stmt)?;
                }
            }
            Stmt::For { iter, body, .. } => {
                self.fold_constants_in_expr(iter)?;
                self.fold_constants_in_stmt(body)?;
            }
            Stmt::While { condition, body, .. } => {
                self.fold_constants_in_expr(condition)?;
                self.fold_constants_in_stmt(body)?;
            }
            Stmt::Match { expr, arms, .. } => {
                self.fold_constants_in_expr(expr)?;
                for arm in arms {
                    self.fold_constants_in_expr(&mut arm.body)?;
                }
            }
            Stmt::Block(block) => {
                self.fold_constants_in_block(block)?;
            }
        }
        Ok(())
    }
    
    fn evaluate_binary(&self, left: &Literal, op: &BinaryOp, right: &Literal) -> Option<Expr> {
        match (left, op, right) {
            (Literal::Number(l), BinaryOp::Add, Literal::Number(r)) => {
                Some(Expr::Literal(Literal::Number(l + r), crate::lexer::Span::new(0, 0, 0, 0)))
            }
            (Literal::Number(l), BinaryOp::Sub, Literal::Number(r)) => {
                Some(Expr::Literal(Literal::Number(l - r), crate::lexer::Span::new(0, 0, 0, 0)))
            }
            (Literal::Number(l), BinaryOp::Mul, Literal::Number(r)) => {
                Some(Expr::Literal(Literal::Number(l * r), crate::lexer::Span::new(0, 0, 0, 0)))
            }
            (Literal::Number(l), BinaryOp::Div, Literal::Number(r)) => {
                if *r != 0.0 {
                    Some(Expr::Literal(Literal::Number(l / r), crate::lexer::Span::new(0, 0, 0, 0)))
                } else {
                    None
                }
            }
            (Literal::Boolean(l), BinaryOp::And, Literal::Boolean(r)) => {
                Some(Expr::Literal(Literal::Boolean(*l && *r), crate::lexer::Span::new(0, 0, 0, 0)))
            }
            (Literal::Boolean(l), BinaryOp::Or, Literal::Boolean(r)) => {
                Some(Expr::Literal(Literal::Boolean(*l || *r), crate::lexer::Span::new(0, 0, 0, 0)))
            }
            _ => None,
        }
    }
    
    fn evaluate_unary(&self, lit: &Literal, op: &UnaryOp) -> Option<Expr> {
        match (lit, op) {
            (Literal::Number(n), UnaryOp::Neg) => {
                Some(Expr::Literal(Literal::Number(-n), crate::lexer::Span::new(0, 0, 0, 0)))
            }
            (Literal::Boolean(b), UnaryOp::Not) => {
                Some(Expr::Literal(Literal::Boolean(!b), crate::lexer::Span::new(0, 0, 0, 0)))
            }
            _ => None,
        }
    }
}
