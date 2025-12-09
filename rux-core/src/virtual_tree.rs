use std::collections::HashMap;

// Add Clone to Patch for renderer usage

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub props: HashMap<String, PropValue>,
    pub children: Vec<VirtualNode>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Element(String),
    Text(String),
    Component(String),
    Fragment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Function(String), // Simplified
}

#[derive(Debug, Clone)]
pub enum Patch {
    Replace {
        node_id: NodeId,
        new_node: VirtualNode,
    },
    UpdateProps {
        node_id: NodeId,
        props: HashMap<String, PropValue>,
    },
    Insert {
        parent_id: NodeId,
        index: usize,
        node: VirtualNode,
    },
    Remove {
        node_id: NodeId,
    },
    Move {
        node_id: NodeId,
        new_parent: NodeId,
        new_index: usize,
    },
}

pub fn diff(old: &VirtualNode, new: &VirtualNode) -> Vec<Patch> {
    let mut patches = Vec::new();
    
    if old.node_type != new.node_type {
        patches.push(Patch::Replace {
            node_id: old.id,
            new_node: new.clone(),
        });
        return patches;
    }
    
    // Diff props
    let prop_patches = diff_props(&old.props, &new.props);
    if !prop_patches.is_empty() {
        patches.push(Patch::UpdateProps {
            node_id: old.id,
            props: new.props.clone(),
        });
    }
    
    // Diff children
    patches.extend(diff_children_with_keys(&old.children, &new.children));
    
    patches
}

fn diff_props(old: &HashMap<String, PropValue>, new: &HashMap<String, PropValue>) -> Vec<(String, PropValue)> {
    let mut changes = Vec::new();
    
    for (key, new_value) in new {
        if let Some(old_value) = old.get(key) {
            if old_value != new_value {
                changes.push((key.clone(), new_value.clone()));
            }
        } else {
            changes.push((key.clone(), new_value.clone()));
        }
    }
    
    for key in old.keys() {
        if !new.contains_key(key) {
            // Prop removed - would need to track this
        }
    }
    
    changes
}

fn diff_children_with_keys(old: &[VirtualNode], new: &[VirtualNode]) -> Vec<Patch> {
    let mut patches = Vec::new();
    
    // Build key maps
    let old_key_map: HashMap<Option<&String>, usize> = old
        .iter()
        .enumerate()
        .map(|(i, node)| (node.key.as_ref(), i))
        .collect();
    
    // Track which old nodes have been matched
    let mut old_matched = vec![false; old.len()];
    
    // First pass: match nodes by key
    for new_node in new.iter() {
        if let Some(key) = &new_node.key {
            if let Some(&old_idx) = old_key_map.get(&Some(key)) {
                if !old_matched[old_idx] {
                    // Nodes match by key - diff them
                    let node_patches = diff(&old[old_idx], new_node);
                    patches.extend(node_patches);
                    old_matched[old_idx] = true;
                    continue;
                }
            }
        }
        
        // No match found - insert new node
        // (Simplified - would need parent_id)
    }
    
    // Second pass: remove unmatched old nodes
    for (old_idx, matched) in old_matched.iter().enumerate() {
        if !matched {
            patches.push(Patch::Remove {
                node_id: old[old_idx].id,
            });
        }
    }
    
    patches
}

pub fn apply_patches(patches: &[Patch], tree: &mut VirtualNode) {
    for patch in patches {
        apply_patch(patch, tree);
    }
}

fn apply_patch(patch: &Patch, tree: &mut VirtualNode) {
    match patch {
        Patch::Replace { node_id, new_node } => {
            if tree.id == *node_id {
                *tree = new_node.clone();
            } else {
                // Recursively find and replace
                find_and_replace(tree, *node_id, new_node);
            }
        }
        Patch::UpdateProps { node_id, props } => {
            if tree.id == *node_id {
                tree.props = props.clone();
            } else {
                find_and_update_props(tree, *node_id, props);
            }
        }
        Patch::Insert { parent_id, index, node } => {
            if tree.id == *parent_id {
                tree.children.insert(*index, node.clone());
            } else {
                find_and_insert(tree, *parent_id, *index, node);
            }
        }
        Patch::Remove { node_id } => {
            if tree.id == *node_id {
                // Would need parent reference
            } else {
                find_and_remove(tree, *node_id);
            }
        }
        Patch::Move { node_id, new_parent, new_index } => {
            // Find node, remove from old position, insert at new position
            // (Simplified implementation)
        }
    }
}

fn find_and_replace(tree: &mut VirtualNode, id: NodeId, new_node: &VirtualNode) {
    for child in &mut tree.children {
        if child.id == id {
            *child = new_node.clone();
            return;
        }
        find_and_replace(child, id, new_node);
    }
}

fn find_and_update_props(tree: &mut VirtualNode, id: NodeId, props: &HashMap<String, PropValue>) {
    for child in &mut tree.children {
        if child.id == id {
            child.props = props.clone();
            return;
        }
        find_and_update_props(child, id, props);
    }
}

fn find_and_insert(tree: &mut VirtualNode, parent_id: NodeId, index: usize, node: &VirtualNode) {
    if tree.id == parent_id {
        tree.children.insert(index, node.clone());
        return;
    }
    for child in &mut tree.children {
        find_and_insert(child, parent_id, index, node);
    }
}

fn find_and_remove(tree: &mut VirtualNode, id: NodeId) {
    tree.children.retain_mut(|child| {
        if child.id == id {
            false
        } else {
            find_and_remove(child, id);
            true
        }
    });
}
