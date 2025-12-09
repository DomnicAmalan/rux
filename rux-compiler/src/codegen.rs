use crate::ast::*;
use crate::errors::Result;

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn generate_rust_code(&mut self, ast: &AST) -> Result<String> {
        self.output.clear();
        self.indent_level = 0;

        // Add necessary imports
        self.writeln("use rux_core::virtual_tree::{VirtualNode, NodeType, PropValue};");
        self.writeln("use std::collections::HashMap;");
        self.writeln("");

        // Generate code for each item
        for item in &ast.items {
            match item {
                Item::Component(component) => self.generate_component(component)?,
                Item::Function(function) => self.generate_function(function)?,
                Item::Struct(struct_def) => self.generate_struct(struct_def)?,
                Item::Enum(enum_def) => self.generate_enum(enum_def)?,
                Item::Trait(trait_def) => self.generate_trait(trait_def)?,
                Item::TypeAlias(alias) => self.generate_type_alias(alias)?,
                Item::Use(use_stmt) => self.generate_use(use_stmt)?,
                Item::Mod(mod_def) => self.generate_mod(mod_def)?,
                Item::Impl(impl_block) => {
                    // Impl blocks would go here
                    self.writeln("// TODO: impl block");
                }
            }
            self.writeln("");
        }

        Ok(self.output.clone())
    }

    fn generate_component(&mut self, component: &Component) -> Result<()> {
        // Generate function signature
        let fn_name = self.snake_case(&component.name);
        self.write(&format!("pub fn {}() -> VirtualNode {{\n", fn_name));
        self.indent_level += 1;

        // Generate component body (JSX expression)
        self.generate_expression(&component.body)?;

        self.indent_level -= 1;
        self.writeln("}");
        Ok(())
    }

    fn generate_function(&mut self, function: &Function) -> Result<()> {
        // Generate function signature
        self.write("pub fn ");
        self.write(&self.snake_case(&function.name));
        self.write("(");

        // Generate parameters
        for (i, param) in function.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&self.snake_case(&param.name));
            self.write(": ");
            self.generate_type(&param.param_type)?;
        }

        self.write(")");

        // Generate return type
        if let Some(ref return_type) = function.return_type {
            self.write(" -> ");
            self.generate_type(return_type)?;
        }

        self.writeln(" {");
        self.indent_level += 1;

        // Generate function body
        self.generate_block(&function.body)?;

        self.indent_level -= 1;
        self.writeln("}");
        Ok(())
    }

    fn generate_struct(&mut self, struct_def: &Struct) -> Result<()> {
        self.write("pub struct ");
        self.write(&struct_def.name);
        self.writeln(" {");

        self.indent_level += 1;
        for field in &struct_def.fields {
            self.indent();
            self.write(&field.name);
            self.write(": ");
            self.generate_type(&field.field_type)?;
            self.writeln(",");
        }
        self.indent_level -= 1;

        self.writeln("}");
        Ok(())
    }

    fn generate_enum(&mut self, enum_def: &Enum) -> Result<()> {
        self.write("pub enum ");
        self.write(&enum_def.name);
        self.writeln(" {");

        self.indent_level += 1;
        for variant in &enum_def.variants {
            self.indent();
            self.write(&variant.name);
            if let Some(ref data) = variant.data {
                match data {
                    EnumVariantData::Tuple(types) => {
                        self.write("(");
                        for (i, ty) in types.iter().enumerate() {
                            if i > 0 {
                                self.write(", ");
                            }
                            self.generate_type(ty)?;
                        }
                        self.write(")");
                    }
                    EnumVariantData::Struct(fields) => {
                        self.writeln(" {");
                        self.indent_level += 1;
                        for field in fields {
                            self.indent();
                            self.write(&field.name);
                            self.write(": ");
                            self.generate_type(&field.field_type)?;
                            self.writeln(",");
                        }
                        self.indent_level -= 1;
                        self.indent();
                        self.write("}");
                    }
                }
            }
            self.writeln(",");
        }
        self.indent_level -= 1;

        self.writeln("}");
        Ok(())
    }

    fn generate_trait(&mut self, trait_def: &Trait) -> Result<()> {
        self.write("pub trait ");
        self.write(&trait_def.name);
        self.writeln(" {");
        // Trait methods would go here
        self.writeln("}");
        Ok(())
    }

    fn generate_type_alias(&mut self, alias: &TypeAlias) -> Result<()> {
        self.write("pub type ");
        self.write(&alias.name);
        self.write(" = ");
        self.generate_type(&alias.aliased_type)?;
        self.writeln(";");
        Ok(())
    }

    fn generate_use(&mut self, use_stmt: &Use) -> Result<()> {
        self.write("use ");
        self.write(&use_stmt.path.join("::"));
        if let Some(ref alias) = use_stmt.alias {
            self.write(" as ");
            self.write(alias);
        }
        self.writeln(";");
        Ok(())
    }

    fn generate_mod(&mut self, mod_def: &Mod) -> Result<()> {
        self.write("pub mod ");
        self.write(&mod_def.name);
        self.writeln(" {");
        // Module contents would go here
        self.writeln("}");
        Ok(())
    }


    fn generate_expression(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit, _) => self.generate_literal(lit)?,
            Expr::Variable(name, _) => {
                self.write(&self.snake_case(name));
            }
            Expr::Binary { left, op, right, .. } => {
                self.write("(");
                self.generate_expression(left)?;
                self.write(" ");
                self.generate_binary_op(op)?;
                self.write(" ");
                self.generate_expression(right)?;
                self.write(")");
            }
            Expr::Unary { op, expr, .. } => {
                self.generate_unary_op(op)?;
                self.write("(");
                self.generate_expression(expr)?;
                self.write(")");
            }
            Expr::Call { callee, args, .. } => {
                self.generate_expression(callee)?;
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expression(arg)?;
                }
                self.write(")");
            }
            Expr::MethodCall { receiver, method, args, .. } => {
                self.generate_expression(receiver)?;
                self.write(".");
                self.write(method);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expression(arg)?;
                }
                self.write(")");
            }
            Expr::FieldAccess { object, field, .. } => {
                self.generate_expression(object)?;
                self.write(".");
                self.write(field);
            }
            Expr::Index { object, index, .. } => {
                self.generate_expression(object)?;
                self.write("[");
                self.generate_expression(index)?;
                self.write("]");
            }
            Expr::JSXElement(jsx, _) => {
                self.generate_jsx(jsx)?;
            }
            Expr::Block(block, _) => {
                self.writeln("{");
                self.indent_level += 1;
                self.generate_block(block)?;
                self.indent_level -= 1;
                self.indent();
                self.write("}");
            }
            Expr::If { condition, then, else_, .. } => {
                self.write("if ");
                self.generate_expression(condition)?;
                self.write(" {\n");
                self.indent_level += 1;
                self.generate_expression(then)?;
                self.indent_level -= 1;
                self.indent();
                self.write("}");
                if let Some(ref else_expr) = else_ {
                    self.write(" else {\n");
                    self.indent_level += 1;
                    self.generate_expression(else_expr)?;
                    self.indent_level -= 1;
                    self.indent();
                    self.write("}");
                }
            }
            Expr::Match { expr, arms, .. } => {
                self.write("match ");
                self.generate_expression(expr)?;
                self.write(" {\n");
                self.indent_level += 1;
                for arm in arms {
                    self.indent();
                    // Pattern matching would go here (simplified)
                    self.write("_ => ");
                    self.generate_expression(&arm.body)?;
                    self.writeln(",");
                }
                self.indent_level -= 1;
                self.indent();
                self.write("}");
            }
            Expr::Lambda { params, body, .. } => {
                self.write("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&self.snake_case(&param.name));
                }
                self.write("| ");
                self.generate_expression(body)?;
            }
            Expr::Tuple(exprs, _) => {
                self.write("(");
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expression(expr)?;
                }
                if exprs.len() == 1 {
                    self.write(",");
                }
                self.write(")");
            }
            Expr::Array(exprs, _) => {
                self.write("vec![");
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expression(expr)?;
                }
                self.write("]");
            }
            Expr::Struct { name, fields, .. } => {
                self.write(name);
                self.write(" {\n");
                self.indent_level += 1;
                for (key, value) in fields {
                    self.indent();
                    self.write(key);
                    self.write(": ");
                    self.generate_expression(value)?;
                    self.writeln(",");
                }
                self.indent_level -= 1;
                self.indent();
                self.write("}");
            }
        }
        Ok(())
    }

    fn generate_jsx(&mut self, jsx: &JSXElement) -> Result<()> {
        match jsx {
            JSXElement::SelfClosing { tag, props, .. } => {
                self.generate_virtual_node(tag, props, &[])?;
            }
            JSXElement::WithChildren { tag, props, children, .. } => {
                // Generate children as VirtualNodes
                let mut child_nodes = Vec::new();
                for child in children {
                    match child {
                        JSXChild::Text(text, _) => {
                            child_nodes.push(format!(
                                "VirtualNode {{ id: rux_core::virtual_tree::NodeId(0), node_type: NodeType::Text(\"{}\".to_string()), props: HashMap::new(), children: vec![], key: None }}",
                                self.escape_string(text)
                            ));
                        }
                        JSXChild::Element(jsx) => {
                            // Recursively generate JSX elements
                            let mut child_gen = CodeGenerator::new();
                            child_gen.generate_jsx(jsx)?;
                            // For now, use a placeholder - full implementation would generate proper code
                            child_nodes.push("VirtualNode { id: rux_core::virtual_tree::NodeId(0), node_type: NodeType::Text(\"TODO\".to_string()), props: HashMap::new(), children: vec![], key: None }".to_string());
                        }
                        JSXChild::Expr(expr) => {
                            // For expressions, we'd need to generate the expression and convert to VirtualNode
                            // For now, use a placeholder
                            child_nodes.push("VirtualNode { id: rux_core::virtual_tree::NodeId(0), node_type: NodeType::Text(\"TODO\".to_string()), props: HashMap::new(), children: vec![], key: None }".to_string());
                        }
                    }
                }

                self.generate_virtual_node(tag, props, &child_nodes)?;
            }
        }
        Ok(())
    }

    fn generate_virtual_node(
        &mut self,
        tag: &str,
        props: &[JSXProp],
        children: &[String],
    ) -> Result<()> {
        self.writeln("VirtualNode {");
        self.indent_level += 1;

        // id
        self.indent();
        self.writeln("id: rux_core::virtual_tree::NodeId(0),");

        // node_type
        self.indent();
        self.write("node_type: NodeType::Element(\"");
        self.write(tag);
        self.writeln("\".to_string()),");

        // props
        self.indent();
        self.write("props: {");
        self.writeln("");
        self.indent_level += 1;
        self.indent();
        self.writeln("let mut props = HashMap::new();");
        for prop in props {
            self.indent();
            self.write("props.insert(\"");
            self.write(&prop.name);
            self.write("\".to_string(), ");
            self.generate_prop_value(&prop.value)?;
            self.writeln(");");
        }
        self.indent();
        self.writeln("props");
        self.indent_level -= 1;
        self.indent();
        self.writeln("},");

        // children
        self.indent();
        self.write("children: vec![");
        if !children.is_empty() {
            self.writeln("");
            self.indent_level += 1;
            for child in children {
                self.indent();
                self.write(child);
                self.writeln(",");
            }
            self.indent_level -= 1;
            self.indent();
        }
        self.writeln("],");

        // key
        self.indent();
        self.writeln("key: None,");

        self.indent_level -= 1;
        self.indent();
        self.write("}");
        Ok(())
    }

    fn generate_prop_value(&mut self, value: &JSXPropValue) -> Result<()> {
        match value {
            JSXPropValue::Literal(lit) => {
                match lit {
                    Literal::String(s) => {
                        self.write("PropValue::String(\"");
                        self.write(&self.escape_string(s));
                        self.write("\".to_string())");
                    }
                    Literal::Number(n) => {
                        self.write("PropValue::Number(");
                        self.write(&n.to_string());
                        self.write(")");
                    }
                    Literal::Boolean(b) => {
                        self.write("PropValue::Boolean(");
                        self.write(if *b { "true" } else { "false" });
                        self.write(")");
                    }
                    _ => {
                        self.write("PropValue::String(\"TODO\".to_string())");
                    }
                }
            }
            JSXPropValue::Bool(b) => {
                self.write("PropValue::Boolean(");
                self.write(if *b { "true" } else { "false" });
                self.write(")");
            }
            JSXPropValue::Expr(expr) => {
                // For expressions in props, we'd need to generate the expression
                // For now, use a placeholder
                self.write("PropValue::String(\"TODO\".to_string())");
            }
        }
        Ok(())
    }

    fn generate_literal(&mut self, lit: &Literal) -> Result<()> {
        match lit {
            Literal::String(s) => {
                self.write("\"");
                self.write(&self.escape_string(s));
                self.write("\"");
            }
            Literal::Number(n) => {
                self.write(&n.to_string());
            }
            Literal::Boolean(b) => {
                self.write(if *b { "true" } else { "false" });
            }
            Literal::Char(c) => {
                self.write("'");
                self.write(&self.escape_char(*c));
                self.write("'");
            }
            Literal::Unit => {
                self.write("()");
            }
        }
        Ok(())
    }

    fn generate_block(&mut self, block: &Block) -> Result<()> {
        for stmt in &block.statements {
            self.generate_statement(stmt)?;
        }
        Ok(())
    }

    fn generate_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, mutable, .. } => {
                self.indent();
                if *mutable {
                    self.write("let mut ");
                } else {
                    self.write("let ");
                }
                self.write(&self.snake_case(name));
                self.write(" = ");
                self.generate_expression(value)?;
                self.writeln(";");
            }
            Stmt::Expr(expr) => {
                self.indent();
                self.generate_expression(expr)?;
                self.writeln(";");
            }
            Stmt::Return(Some(expr), _) => {
                self.indent();
                self.write("return ");
                self.generate_expression(expr)?;
                self.writeln(";");
            }
            Stmt::Return(None, _) => {
                self.indent();
                self.writeln("return;");
            }
            Stmt::If { condition, then, else_, .. } => {
                self.indent();
                self.write("if ");
                self.generate_expression(condition)?;
                self.write(" {\n");
                self.indent_level += 1;
                self.generate_statement(then)?;
                self.indent_level -= 1;
                self.indent();
                self.write("}");
                if let Some(ref else_stmt) = else_ {
                    self.write(" else {\n");
                    self.indent_level += 1;
                    self.generate_statement(else_stmt)?;
                    self.indent_level -= 1;
                    self.indent();
                    self.write("}");
                }
                self.writeln("");
            }
            Stmt::For { var, iter, body, .. } => {
                self.indent();
                self.write("for ");
                self.write(&self.snake_case(var));
                self.write(" in ");
                self.generate_expression(iter)?;
                self.write(" {\n");
                self.indent_level += 1;
                self.generate_statement(body)?;
                self.indent_level -= 1;
                self.indent();
                self.writeln("}");
            }
            Stmt::While { condition, body, .. } => {
                self.indent();
                self.write("while ");
                self.generate_expression(condition)?;
                self.write(" {\n");
                self.indent_level += 1;
                self.generate_statement(body)?;
                self.indent_level -= 1;
                self.indent();
                self.writeln("}");
            }
            Stmt::Match { expr, arms, .. } => {
                self.indent();
                self.write("match ");
                self.generate_expression(expr)?;
                self.write(" {\n");
                self.indent_level += 1;
                for arm in arms {
                    self.indent();
                    // Pattern matching would go here
                    self.write("_ => ");
                    self.generate_expression(&arm.body)?;
                    self.writeln(",");
                }
                self.indent_level -= 1;
                self.indent();
                self.writeln("}");
            }
            Stmt::Block(block) => {
                self.writeln("{");
                self.indent_level += 1;
                self.generate_block(block)?;
                self.indent_level -= 1;
                self.indent();
                self.writeln("}");
            }
        }
        Ok(())
    }

    fn generate_type(&mut self, ty: &Type) -> Result<()> {
        match &ty.kind {
            TypeKind::Ident(name) => {
                self.write(name);
            }
            TypeKind::Path(path) => {
                self.write(&path.join("::"));
            }
            TypeKind::Unit => {
                self.write("()");
            }
            TypeKind::Tuple(types) => {
                self.write("(");
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_type(t)?;
                }
                if types.len() == 1 {
                    self.write(",");
                }
                self.write(")");
            }
            TypeKind::Array(elem_type) => {
                self.write("Vec<");
                self.generate_type(elem_type)?;
                self.write(">");
            }
            TypeKind::Slice(elem_type) => {
                self.write("[");
                self.generate_type(elem_type)?;
                self.write("]");
            }
            TypeKind::Function { params, return_type } => {
                self.write("fn(");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_type(param)?;
                }
                self.write(") -> ");
                self.generate_type(return_type)?;
            }
            TypeKind::Reference { mutable, inner } => {
                self.write("&");
                if *mutable {
                    self.write("mut ");
                }
                self.generate_type(inner)?;
            }
            TypeKind::Option(inner) => {
                self.write("Option<");
                self.generate_type(inner)?;
                self.write(">");
            }
            TypeKind::Result { ok, err } => {
                self.write("Result<");
                self.generate_type(ok)?;
                self.write(", ");
                self.generate_type(err)?;
                self.write(">");
            }
        }
        Ok(())
    }

    fn generate_binary_op(&mut self, op: &BinaryOp) -> Result<()> {
        match op {
            BinaryOp::Add => self.write("+"),
            BinaryOp::Sub => self.write("-"),
            BinaryOp::Mul => self.write("*"),
            BinaryOp::Div => self.write("/"),
            BinaryOp::Rem => self.write("%"),
            BinaryOp::Eq => self.write("=="),
            BinaryOp::Ne => self.write("!="),
            BinaryOp::Lt => self.write("<"),
            BinaryOp::Le => self.write("<="),
            BinaryOp::Gt => self.write(">"),
            BinaryOp::Ge => self.write(">="),
            BinaryOp::And => self.write("&&"),
            BinaryOp::Or => self.write("||"),
            BinaryOp::BitAnd => self.write("&"),
            BinaryOp::BitOr => self.write("|"),
            BinaryOp::BitXor => self.write("^"),
            BinaryOp::Shl => self.write("<<"),
            BinaryOp::Shr => self.write(">>"),
        }
        Ok(())
    }

    fn generate_unary_op(&mut self, op: &UnaryOp) -> Result<()> {
        match op {
            UnaryOp::Not => self.write("!"),
            UnaryOp::Neg => self.write("-"),
            UnaryOp::Deref => self.write("*"),
            UnaryOp::Ref => self.write("&"),
        }
        Ok(())
    }

    // Helper methods
    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn snake_case(&self, s: &str) -> String {
        // Simple conversion - in production, use a proper library
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        result
    }

    fn escape_string(&self, s: &str) -> String {
        s.replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
    }

    fn escape_char(&self, c: char) -> String {
        match c {
            '\\' => "\\\\".to_string(),
            '\'' => "\\'".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            _ => c.to_string(),
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
