use crate::virtual_tree::{VirtualNode, NodeId, Patch};

pub trait Renderer {
    fn create_element(&mut self, node: &VirtualNode) -> ElementId;
    fn update_element(&mut self, element_id: ElementId, patches: &[Patch]);
    fn remove_element(&mut self, element_id: ElementId);
    fn mount(&mut self, root: ElementId, node: &VirtualNode);
    fn unmount(&mut self, root: ElementId);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(pub usize);

pub struct RenderContext {
    pub element_id_counter: usize,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            element_id_counter: 0,
        }
    }
    
    pub fn next_element_id(&mut self) -> ElementId {
        let id = ElementId(self.element_id_counter);
        self.element_id_counter += 1;
        id
    }
}

pub fn apply_patches_to_renderer<R: Renderer>(
    renderer: &mut R,
    patches: &[Patch],
    root_id: ElementId,
) {
    for patch in patches {
        match patch {
            Patch::Replace { node_id: _, new_node } => {
                // Remove old, create new
                let _new_element_id = renderer.create_element(new_node);
                renderer.mount(root_id, new_node);
            }
            Patch::UpdateProps { node_id, props: _ } => {
                // Update element properties
                // Would need to map node_id to element_id
                let _element_id = ElementId(node_id.0);
                renderer.update_element(ElementId(0), &[patch.clone()]);
            }
            Patch::Insert { parent_id, index: _, node } => {
                let _element_id = renderer.create_element(node);
                // Would need to map NodeId to ElementId
                renderer.mount(root_id, node);
            }
            Patch::Remove { node_id: _ } => {
                // Would need element_id mapping
            }
            Patch::Move { node_id: _, new_parent: _, new_index: _ } => {
                // Move element to new position
                // (Simplified)
            }
        }
    }
}
