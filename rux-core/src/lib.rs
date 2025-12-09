// Core runtime library for RUX

pub mod signals;
pub mod virtual_tree;
pub mod scheduler;
pub mod renderer;
pub mod layout;

pub use signals::{Signal, SignalRegistry, create_signal, create_derived, create_computed};
pub use virtual_tree::{VirtualNode, NodeId, NodeType, Patch, diff, apply_patches};
pub use scheduler::{Scheduler, Priority, Fiber, FiberId, schedule_work, should_yield};
pub use renderer::{Renderer, ElementId, RenderContext, apply_patches_to_renderer};
pub use layout::{
    Rect, Size, Constraints, FlexLayout, StackLayout, GridLayout,
    LayoutDirection, MainAxisAlignment, CrossAxisAlignment, StackAlignment,
    LayoutChild,
};
