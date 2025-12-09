use rux_core::renderer::{Renderer, ElementId};
use rux_core::virtual_tree::{VirtualNode, NodeId, Patch, PropValue, NodeType};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Text};
use std::collections::HashMap;

pub struct WebRenderer {
    document: Document,
    node_to_element: HashMap<NodeId, ElementId>,
    element_to_node: HashMap<ElementId, NodeId>,
    element_map: HashMap<ElementId, Element>,
    root_element: Option<Element>,
    next_element_id: usize,
}

impl WebRenderer {
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("No document"))?;
        
        Ok(Self {
            document,
            node_to_element: HashMap::new(),
            element_to_node: HashMap::new(),
            element_map: HashMap::new(),
            root_element: None,
            next_element_id: 1,
        })
    }
    
    pub fn mount_to_element_id(&mut self, element_id: &str, node: &VirtualNode) -> Result<(), JsValue> {
        let container = self.document
            .get_element_by_id(element_id)
            .ok_or_else(|| JsValue::from_str("Element not found"))?;
        
        self.root_element = Some(container.clone());
        
        // Clear container
        container.set_inner_html("");
        
        // Mount the virtual tree
        self.mount(ElementId(0), node);
        Ok(())
    }
    
    fn create_element_from_node(&mut self, node: &VirtualNode) -> Result<Element, JsValue> {
        match &node.node_type {
            NodeType::Element(tag) => {
                let element = self.document.create_element(tag)?;
                
                // Set attributes/props
                for (key, value) in &node.props {
                    self.set_prop(&element, key, value)?;
                }
                
                Ok(element)
            }
            NodeType::Text(text) => {
                // Text nodes are handled differently - they're appended directly
                // Return a placeholder element for now
                let div = self.document.create_element("span")?;
                div.set_text_content(Some(text));
                Ok(div)
            }
            NodeType::Component(_) => {
                // Component rendering would go here
                let div = self.document.create_element("div")?;
                Ok(div)
            }
            NodeType::Fragment => {
                // Fragments don't create elements
                let div = self.document.create_element("div")?;
                Ok(div)
            }
        }
    }
    
    fn set_prop(&self, element: &Element, key: &str, value: &PropValue) -> Result<(), JsValue> {
        match value {
            PropValue::String(s) => {
                if key.starts_with("on") {
                    // Event handler - simplified
                    // In a real implementation, we'd set up event listeners
                } else {
                    element.set_attribute(key, s)?;
                }
            }
            PropValue::Boolean(b) => {
                if *b {
                    element.set_attribute(key, "")?;
                } else {
                    element.remove_attribute(key);
                }
            }
            PropValue::Number(n) => {
                element.set_attribute(key, &n.to_string())?;
            }
            PropValue::Function(_) => {
                // Function props (event handlers) would be handled here
            }
        }
        Ok(())
    }
    
    fn apply_patches(&mut self, patches: &[Patch]) -> Result<(), JsValue> {
        for patch in patches {
            match patch {
                Patch::Replace { node_id, new_node } => {
                    if let Some(&element_id) = self.node_to_element.get(node_id) {
                        let old_element_opt = self.element_map.get(&element_id).cloned();
                        if let Some(old_element) = old_element_opt {
                            let new_element = self.create_element_from_node(new_node)?;
                            if let Some(parent) = old_element.parent_element() {
                                parent.replace_child(&new_element, &old_element)?;
                                self.element_map.insert(element_id, new_element);
                            }
                        }
                    }
                }
                Patch::UpdateProps { node_id, props } => {
                    if let Some(&element_id) = self.node_to_element.get(node_id) {
                        if let Some(element) = self.element_map.get(&element_id) {
                            for (key, value) in props {
                                self.set_prop(element, key, value)?;
                            }
                        }
                    }
                }
                Patch::Insert { parent_id, index: _, node } => {
                    if let Some(&parent_element_id) = self.node_to_element.get(parent_id) {
                        let parent_element_opt = self.element_map.get(&parent_element_id).cloned();
                        if let Some(parent_element) = parent_element_opt {
                            let new_element = self.create_element_from_node(node)?;
                            let element_id = ElementId(self.next_element_id);
                            self.next_element_id += 1;
                            
                            self.node_to_element.insert(node.id, element_id);
                            self.element_to_node.insert(element_id, node.id);
                            self.element_map.insert(element_id, new_element.clone());
                            
                            // Insert at index
                            // For now, just append - proper index insertion would need more complex logic
                            parent_element.append_child(&new_element)?;
                        }
                    }
                }
                Patch::Remove { node_id } => {
                    if let Some(&element_id) = self.node_to_element.get(node_id) {
                        if let Some(element) = self.element_map.remove(&element_id) {
                            if let Some(parent) = element.parent_element() {
                                parent.remove_child(&element)?;
                            }
                            self.node_to_element.remove(node_id);
                            self.element_to_node.remove(&element_id);
                        }
                    }
                }
                Patch::Move { node_id, new_parent, new_index: _ } => {
                    if let Some(&element_id) = self.node_to_element.get(node_id) {
                        let element_opt = self.element_map.get(&element_id).cloned();
                        if let Some(element) = element_opt {
                            if let Some(&new_parent_element_id) = self.node_to_element.get(new_parent) {
                                let new_parent_element_opt = self.element_map.get(&new_parent_element_id).cloned();
                                if let Some(new_parent_element) = new_parent_element_opt {
                                    // Remove from old position
                                    if let Some(old_parent) = element.parent_element() {
                                        old_parent.remove_child(&element)?;
                                    }
                                    // Insert at new position
                                    // For now, just append - proper index insertion would need more complex logic
                                    new_parent_element.append_child(&element)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Renderer for WebRenderer {
    fn create_element(&mut self, node: &VirtualNode) -> ElementId {
        let element_id = ElementId(self.next_element_id);
        self.next_element_id += 1;
        
        // Create DOM element
        if let Ok(element) = self.create_element_from_node(node) {
            self.node_to_element.insert(node.id, element_id);
            self.element_to_node.insert(element_id, node.id);
            self.element_map.insert(element_id, element);
        }
        
        element_id
    }
    
    fn update_element(&mut self, element_id: ElementId, patches: &[Patch]) {
        // Apply patches to update DOM
        if let Err(e) = self.apply_patches(patches) {
            // Log error - in WASM, we'd use wasm_bindgen::console::error
            let _ = e;
        }
    }
    
    fn remove_element(&mut self, element_id: ElementId) {
        if let Some(element) = self.element_map.remove(&element_id) {
            if let Some(node_id) = self.element_to_node.remove(&element_id) {
                self.node_to_element.remove(&node_id);
            }
            if let Some(parent) = element.parent_element() {
                let _ = parent.remove_child(&element);
            }
        }
    }
    
    fn mount(&mut self, root: ElementId, node: &VirtualNode) {
        let parent_opt = if root.0 == 0 {
            self.root_element.clone()
        } else {
            self.element_map.get(&root).cloned()
        };
        
        if let Some(parent) = parent_opt {
            self.mount_recursive(&parent, node, 0);
        }
    }
    
    fn unmount(&mut self, root: ElementId) {
        if let Some(element) = self.element_map.get(&root) {
            if let Some(parent) = element.parent_element() {
                let _ = parent.remove_child(element);
            }
        }
        self.remove_element(root);
    }
    
}

impl WebRenderer {
    fn mount_recursive(&mut self, parent: &Element, node: &VirtualNode, depth: usize) {
        match &node.node_type {
            NodeType::Element(_) | 
            NodeType::Component(_) => {
                if let Ok(element) = self.create_element_from_node(node) {
                    let element_id = ElementId(self.next_element_id);
                    self.next_element_id += 1;
                    
                    self.node_to_element.insert(node.id, element_id);
                    self.element_to_node.insert(element_id, node.id);
                    self.element_map.insert(element_id, element.clone());
                    
                    // Mount children
                    for child in &node.children {
                        self.mount_recursive(&element, child, depth + 1);
                    }
                    
                    parent.append_child(&element).ok();
                }
            }
            NodeType::Text(text) => {
                let text_node = self.document.create_text_node(text);
                parent.append_child(&text_node).ok();
            }
            NodeType::Fragment => {
                for child in &node.children {
                    self.mount_recursive(parent, child, depth);
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn init_rux_web() {
    // Initialize RUX web runtime
}

#[wasm_bindgen]
pub fn render_to_element(element_id: &str) {
    // Render component to DOM element
    let _ = element_id;
}
