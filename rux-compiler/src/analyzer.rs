use crate::ast::*;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};

pub struct DependencyAnalyzer {
    dependencies: HashMap<String, HashSet<String>>,
    signals: HashSet<String>,
    components: HashSet<String>,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            signals: HashSet::new(),
            components: HashSet::new(),
        }
    }
    
    pub fn analyze(&mut self, ast: &AST) -> Result<()> {
        // Build dependency graph
        for item in &ast.items {
            match item {
                Item::Component(c) => {
                    self.components.insert(c.name.clone());
                    self.analyze_component(c)?;
                }
                Item::Function(f) => {
                    self.analyze_function(f)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn analyze_component(&mut self, component: &Component) -> Result<()> {
        let mut deps = HashSet::new();
        self.collect_dependencies_from_expr(&component.body, &mut deps);
        self.dependencies.insert(component.name.clone(), deps);
        Ok(())
    }
    
    fn analyze_function(&mut self, function: &Function) -> Result<()> {
        let mut deps = HashSet::new();
        self.collect_dependencies_from_block(&function.body, &mut deps);
        self.dependencies.insert(function.name.clone(), deps);
        Ok(())
    }
    
    fn collect_dependencies_from_expr(&self, expr: &Expr, deps: &mut HashSet<String>) {
        match expr {
            Expr::Variable(name, _) => {
                deps.insert(name.clone());
            }
            Expr::Call { callee, args, .. } => {
                if let Expr::Variable(name, _) = callee.as_ref() {
                    deps.insert(name.clone());
                }
                for arg in args {
                    self.collect_dependencies_from_expr(arg, deps);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_dependencies_from_expr(left, deps);
                self.collect_dependencies_from_expr(right, deps);
            }
            Expr::Unary { expr, .. } => {
                self.collect_dependencies_from_expr(expr, deps);
            }
            Expr::JSXElement(jsx, _) => {
                self.collect_dependencies_from_jsx(jsx, deps);
            }
            Expr::Block(block, _) => {
                self.collect_dependencies_from_block(block, deps);
            }
            Expr::If { condition, then, else_, .. } => {
                self.collect_dependencies_from_expr(condition, deps);
                self.collect_dependencies_from_expr(then, deps);
                if let Some(else_expr) = else_ {
                    self.collect_dependencies_from_expr(else_expr, deps);
                }
            }
            Expr::Match { expr, arms, .. } => {
                self.collect_dependencies_from_expr(expr, deps);
                for arm in arms {
                    self.collect_dependencies_from_expr(&arm.body, deps);
                }
            }
            _ => {}
        }
    }
    
    fn collect_dependencies_from_block(&self, block: &Block, deps: &mut HashSet<String>) {
        for stmt in &block.statements {
            self.collect_dependencies_from_stmt(stmt, deps);
        }
    }
    
    fn collect_dependencies_from_stmt(&self, stmt: &Stmt, deps: &mut HashSet<String>) {
        match stmt {
            Stmt::Let { value, .. } => {
                self.collect_dependencies_from_expr(value, deps);
            }
            Stmt::Expr(expr) => {
                self.collect_dependencies_from_expr(expr, deps);
            }
            Stmt::Return(Some(expr), _) => {
                self.collect_dependencies_from_expr(expr, deps);
            }
            Stmt::Return(None, _) => {
                // No dependencies for return without value
            }
            Stmt::If { condition, then, else_, .. } => {
                self.collect_dependencies_from_expr(condition, deps);
                self.collect_dependencies_from_stmt(then, deps);
                if let Some(else_stmt) = else_ {
                    self.collect_dependencies_from_stmt(else_stmt, deps);
                }
            }
            Stmt::For { iter, body, .. } => {
                self.collect_dependencies_from_expr(iter, deps);
                self.collect_dependencies_from_stmt(body, deps);
            }
            Stmt::While { condition, body, .. } => {
                self.collect_dependencies_from_expr(condition, deps);
                self.collect_dependencies_from_stmt(body, deps);
            }
            Stmt::Match { expr, arms, .. } => {
                self.collect_dependencies_from_expr(expr, deps);
                for arm in arms {
                    self.collect_dependencies_from_expr(&arm.body, deps);
                }
            }
            Stmt::Block(block) => {
                self.collect_dependencies_from_block(block, deps);
            }
        }
    }
    
    fn collect_dependencies_from_jsx(&self, jsx: &JSXElement, deps: &mut HashSet<String>) {
        match jsx {
            JSXElement::SelfClosing { tag, props, .. } => {
                deps.insert(tag.clone());
                for prop in props {
                    self.collect_dependencies_from_jsx_prop(prop, deps);
                }
            }
            JSXElement::WithChildren { tag, props, children, .. } => {
                deps.insert(tag.clone());
                for prop in props {
                    self.collect_dependencies_from_jsx_prop(prop, deps);
                }
                for child in children {
                    match child {
                        JSXChild::Element(elem) => self.collect_dependencies_from_jsx(elem, deps),
                        JSXChild::Expr(expr) => self.collect_dependencies_from_expr(expr, deps),
                        JSXChild::Text(_, _) => {}
                    }
                }
            }
        }
    }
    
    fn collect_dependencies_from_jsx_prop(&self, prop: &JSXProp, deps: &mut HashSet<String>) {
        match &prop.value {
            JSXPropValue::Expr(expr) => {
                self.collect_dependencies_from_expr(expr, deps);
            }
            _ => {}
        }
    }
    
    pub fn get_dependencies(&self, name: &str) -> Option<&HashSet<String>> {
        self.dependencies.get(name)
    }
    
    pub fn track_reactive_dependencies(&self, expr: &Expr) -> HashSet<String> {
        let mut signals = HashSet::new();
        self.collect_reactive_dependencies(expr, &mut signals);
        signals
    }
    
    fn collect_reactive_dependencies(&self, expr: &Expr, signals: &mut HashSet<String>) {
        match expr {
            Expr::Call { callee, .. } => {
                if let Expr::Variable(name, _) = callee.as_ref() {
                    // Check if this is a signal access (simplified)
                    if name.starts_with("use_") || name.ends_with("()") {
                        signals.insert(name.clone());
                    }
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_reactive_dependencies(left, signals);
                self.collect_reactive_dependencies(right, signals);
            }
            Expr::Unary { expr, .. } => {
                self.collect_reactive_dependencies(expr, signals);
            }
            Expr::JSXElement(jsx, _) => {
                self.collect_reactive_from_jsx(jsx, signals);
            }
            _ => {}
        }
    }
    
    fn collect_reactive_from_jsx(&self, jsx: &JSXElement, signals: &mut HashSet<String>) {
        match jsx {
            JSXElement::SelfClosing { props, .. } => {
                for prop in props {
                    if let JSXPropValue::Expr(expr) = &prop.value {
                        self.collect_reactive_dependencies(expr, signals);
                    }
                }
            }
            JSXElement::WithChildren { props, children, .. } => {
                for prop in props {
                    if let JSXPropValue::Expr(expr) = &prop.value {
                        self.collect_reactive_dependencies(expr, signals);
                    }
                }
                for child in children {
                    if let JSXChild::Expr(expr) = child {
                        self.collect_reactive_dependencies(expr, signals);
                    }
                }
            }
        }
    }
}
