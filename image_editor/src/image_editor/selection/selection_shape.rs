use framework::Box2d;

use super::SelectionAddition;

#[derive(Debug, Clone)]
pub enum Shape {
    Rectangle(Box2d),
}

#[derive(Debug, Clone)]
pub struct SelectionShape {
    pub mode: SelectionAddition,
    pub shape: Shape,
}
