use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }
    
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }
    
    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }
    
    pub fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.max(self.min_width).min(self.max_width),
            height: size.height.max(self.min_height).min(self.max_height),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum MainAxisAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Center,
    Stretch,
}

#[derive(Debug, Clone)]
pub struct FlexLayout {
    pub direction: LayoutDirection,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub spacing: f32,
}

impl FlexLayout {
    pub fn new(direction: LayoutDirection) -> Self {
        Self {
            direction,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            spacing: 0.0,
        }
    }
    
    pub fn layout(&self, constraints: Constraints, children: &[Box<dyn LayoutChild>]) -> Vec<Rect> {
        match self.direction {
            LayoutDirection::Horizontal => self.layout_horizontal(constraints, children),
            LayoutDirection::Vertical => self.layout_vertical(constraints, children),
        }
    }
    
    fn layout_horizontal(&self, constraints: Constraints, children: &[Box<dyn LayoutChild>]) -> Vec<Rect> {
        let mut positions = Vec::new();
        let mut current_x: f32 = 0.0;
        let mut max_height: f32 = 0.0;
        
        // First pass: calculate sizes
        let child_constraints = Constraints::new(
            0.0,
            constraints.max_width / children.len() as f32,
            constraints.min_height,
            constraints.max_height,
        );
        
        let mut child_sizes: Vec<Size> = children
            .iter()
            .map(|child| child.layout(child_constraints))
            .collect();
        
        // Calculate total width
        let total_width: f32 = child_sizes.iter().map(|s| s.width).sum::<f32>()
            + self.spacing * (children.len().saturating_sub(1)) as f32;
        
        // Adjust if needed
        if total_width > constraints.max_width {
            let scale = constraints.max_width / total_width;
            for size in &mut child_sizes {
                size.width *= scale;
            }
        }
        
        // Second pass: position children
        for (i, size) in child_sizes.iter().enumerate() {
            if i > 0 {
                current_x += self.spacing;
            }
            
            let y = match self.cross_axis_alignment {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::End => constraints.max_height - size.height,
                CrossAxisAlignment::Center => (constraints.max_height - size.height) / 2.0,
                CrossAxisAlignment::Stretch => 0.0,
            };
            
            positions.push(Rect {
                x: current_x,
                y,
                width: size.width,
                height: size.height,
            });
            
            current_x += size.width;
            max_height = max_height.max(size.height);
        }
        
        positions
    }
    
    fn layout_vertical(&self, constraints: Constraints, children: &[Box<dyn LayoutChild>]) -> Vec<Rect> {
        let mut positions = Vec::new();
        let mut current_y: f32 = 0.0;
        let mut max_width: f32 = 0.0;
        
        // First pass: calculate sizes
        let child_constraints = Constraints::new(
            constraints.min_width,
            constraints.max_width,
            0.0,
            constraints.max_height / children.len() as f32,
        );
        
        let mut child_sizes: Vec<Size> = children
            .iter()
            .map(|child| child.layout(child_constraints))
            .collect();
        
        // Calculate total height
        let total_height: f32 = child_sizes.iter().map(|s| s.height).sum::<f32>()
            + self.spacing * (children.len().saturating_sub(1)) as f32;
        
        // Adjust if needed
        if total_height > constraints.max_height {
            let scale = constraints.max_height / total_height;
            for size in &mut child_sizes {
                size.height *= scale;
            }
        }
        
        // Second pass: position children
        for (i, size) in child_sizes.iter().enumerate() {
            if i > 0 {
                current_y += self.spacing;
            }
            
            let x = match self.cross_axis_alignment {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::End => constraints.max_width - size.width,
                CrossAxisAlignment::Center => (constraints.max_width - size.width) / 2.0,
                CrossAxisAlignment::Stretch => 0.0,
            };
            
            positions.push(Rect {
                x,
                y: current_y,
                width: size.width,
                height: size.height,
            });
            
            current_y += size.height;
            max_width = max_width.max(size.width);
        }
        
        positions
    }
}

pub trait LayoutChild: std::fmt::Debug {
    fn layout(&self, constraints: Constraints) -> Size;
}

// Simple implementation for testing
#[derive(Debug, Clone)]
pub struct SimpleLayoutChild {
    pub size: Size,
}

impl LayoutChild for SimpleLayoutChild {
    fn layout(&self, constraints: Constraints) -> Size {
        constraints.constrain(self.size)
    }
}

#[derive(Debug, Clone)]
pub struct StackLayout {
    pub alignment: StackAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackAlignment {
    TopStart,
    TopCenter,
    TopEnd,
    CenterStart,
    Center,
    CenterEnd,
    BottomStart,
    BottomCenter,
    BottomEnd,
}

impl StackLayout {
    pub fn new() -> Self {
        Self {
            alignment: StackAlignment::Center,
        }
    }
    
    pub fn layout(&self, constraints: Constraints, children: &[Box<dyn LayoutChild>]) -> Vec<Rect> {
        children
            .iter()
            .map(|child| {
                let size = child.layout(constraints);
                let (x, y) = self.align_position(size, constraints);
                Rect {
                    x,
                    y,
                    width: size.width,
                    height: size.height,
                }
            })
            .collect()
    }
    
    fn align_position(&self, size: Size, constraints: Constraints) -> (f32, f32) {
        match self.alignment {
            StackAlignment::TopStart => (0.0, 0.0),
            StackAlignment::TopCenter => ((constraints.max_width - size.width) / 2.0, 0.0),
            StackAlignment::TopEnd => (constraints.max_width - size.width, 0.0),
            StackAlignment::CenterStart => (0.0, (constraints.max_height - size.height) / 2.0),
            StackAlignment::Center => (
                (constraints.max_width - size.width) / 2.0,
                (constraints.max_height - size.height) / 2.0,
            ),
            StackAlignment::CenterEnd => (
                constraints.max_width - size.width,
                (constraints.max_height - size.height) / 2.0,
            ),
            StackAlignment::BottomStart => (0.0, constraints.max_height - size.height),
            StackAlignment::BottomCenter => (
                (constraints.max_width - size.width) / 2.0,
                constraints.max_height - size.height,
            ),
            StackAlignment::BottomEnd => (
                constraints.max_width - size.width,
                constraints.max_height - size.height,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridLayout {
    pub columns: usize,
    pub rows: usize,
    pub spacing: f32,
}

impl GridLayout {
    pub fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns,
            rows,
            spacing: 0.0,
        }
    }
    
    pub fn layout(&self, constraints: Constraints, children: &[Box<dyn LayoutChild>]) -> Vec<Rect> {
        let cell_width = (constraints.max_width - self.spacing * (self.columns - 1) as f32) / self.columns as f32;
        let cell_height = (constraints.max_height - self.spacing * (self.rows - 1) as f32) / self.rows as f32;
        
        let cell_constraints = Constraints::tight(Size {
            width: cell_width,
            height: cell_height,
        });
        
        let mut positions = Vec::new();
        
        for (i, child) in children.iter().enumerate() {
            let col = i % self.columns;
            let row = i / self.columns;
            
            let size = child.layout(cell_constraints);
            
            positions.push(Rect {
                x: col as f32 * (cell_width + self.spacing),
                y: row as f32 * (cell_height + self.spacing),
                width: size.width,
                height: size.height,
            });
        }
        
        positions
    }
}
