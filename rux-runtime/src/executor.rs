use rux_core::virtual_tree::{VirtualNode, NodeType, PropValue};
use rux_compiler::ast::{AST, Component, JSXElement, JSXChild, JSXProp, JSXPropValue, Expr, Literal};
use std::collections::HashMap;

/// Converts RUX AST to VirtualNode for rendering
pub struct ComponentExecutor;

impl ComponentExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a component from AST and return a VirtualNode
    pub fn execute_component(&self, component: &Component) -> VirtualNode {
        // Convert component body (JSX expression) to VirtualNode
        self.expr_to_virtual_node(&component.body, 0)
    }

    /// Convert an expression to a VirtualNode
    fn expr_to_virtual_node(&self, expr: &Expr, node_id: usize) -> VirtualNode {
        match expr {
            Expr::JSXElement(jsx, _) => self.jsx_to_virtual_node(jsx, node_id),
            Expr::Literal(lit, _) => {
                VirtualNode {
                    id: rux_core::virtual_tree::NodeId(node_id),
                    node_type: NodeType::Text(self.literal_to_string(lit)),
                    props: HashMap::new(),
                    children: vec![],
                    key: None,
                }
            }
            _ => {
                // For other expressions, create a placeholder
                VirtualNode {
                    id: rux_core::virtual_tree::NodeId(node_id),
                    node_type: NodeType::Text("TODO".to_string()),
                    props: HashMap::new(),
                    children: vec![],
                    key: None,
                }
            }
        }
    }

    /// Convert JSX element to VirtualNode
    fn jsx_to_virtual_node(&self, jsx: &JSXElement, node_id: usize) -> VirtualNode {
        match jsx {
            JSXElement::SelfClosing { tag, props, .. } => {
                VirtualNode {
                    id: rux_core::virtual_tree::NodeId(node_id),
                    node_type: NodeType::Element(tag.clone()),
                    props: self.jsx_props_to_props(props),
                    children: vec![],
                    key: None,
                }
            }
            JSXElement::WithChildren { tag, props, children, .. } => {
                let mut child_nodes = Vec::new();
                let mut current_id = node_id + 1;

                for child in children {
                    match child {
                        JSXChild::Text(text, _) => {
                            child_nodes.push(VirtualNode {
                                id: rux_core::virtual_tree::NodeId(current_id),
                                node_type: NodeType::Text(text.clone()),
                                props: HashMap::new(),
                                children: vec![],
                                key: None,
                            });
                            current_id += 1;
                        }
                        JSXChild::Element(jsx) => {
                            let child_node = self.jsx_to_virtual_node(jsx, current_id);
                            current_id += self.count_nodes(&child_node);
                            child_nodes.push(child_node);
                        }
                        JSXChild::Expr(expr) => {
                            let child_node = self.expr_to_virtual_node(expr, current_id);
                            current_id += self.count_nodes(&child_node);
                            child_nodes.push(child_node);
                        }
                    }
                }

                VirtualNode {
                    id: rux_core::virtual_tree::NodeId(node_id),
                    node_type: NodeType::Element(tag.clone()),
                    props: self.jsx_props_to_props(props),
                    children: child_nodes,
                    key: None,
                }
            }
        }
    }

    /// Convert JSX props to PropValue map
    fn jsx_props_to_props(&self, props: &[JSXProp]) -> HashMap<String, PropValue> {
        let mut result = HashMap::new();
        for prop in props {
            let value = match &prop.value {
                JSXPropValue::Literal(lit) => match lit {
                    Literal::String(s) => PropValue::String(s.clone()),
                    Literal::Number(n) => PropValue::Number(*n),
                    Literal::Boolean(b) => PropValue::Boolean(*b),
                    _ => PropValue::String("".to_string()),
                },
                JSXPropValue::Bool(b) => PropValue::Boolean(*b),
                JSXPropValue::Expr(_) => {
                    // For expressions, we'd need to evaluate them
                    // For now, use a placeholder
                    PropValue::String("TODO".to_string())
                }
            };
            result.insert(prop.name.clone(), value);
        }
        result
    }

    /// Convert literal to string
    fn literal_to_string(&self, lit: &Literal) -> String {
        match lit {
            Literal::String(s) => s.clone(),
            Literal::Number(n) => n.to_string(),
            Literal::Boolean(b) => b.to_string(),
            Literal::Char(c) => c.to_string(),
            Literal::Unit => "()".to_string(),
        }
    }

    /// Count total nodes in a VirtualNode tree
    fn count_nodes(&self, node: &VirtualNode) -> usize {
        1 + node.children.iter().map(|c| self.count_nodes(c)).sum::<usize>()
    }
}

impl Default for ComponentExecutor {
    fn default() -> Self {
        Self::new()
    }
}
