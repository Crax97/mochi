mod selection_shape;

use cgmath::{ElementWise, Point2, Vector2};
pub use selection_shape::SelectionShape;

pub use self::selection_shape::Shape;

#[derive(Default, Debug, Clone)]
pub struct Selection {
    pub(crate) shapes: Vec<SelectionShape>,
    pub(crate) inverted: bool,
}

#[derive(Clone, Copy, strum_macros::EnumIter, strum_macros::Display, Eq, PartialEq, Debug)]
pub enum SelectionAddition {
    Add = 0,
    Subtract = 1,
}

impl From<usize> for SelectionAddition {
    fn from(n: usize) -> Self {
        match n {
            0 => Self::Add,
            1 => Self::Subtract,
            _ => unreachable!(),
        }
    }
}

impl From<SelectionAddition> for usize {
    fn from(v: SelectionAddition) -> Self {
        v as usize
    }
}

impl Selection {
    pub fn set(&mut self, new_selection: SelectionShape) {
        self.shapes = vec![new_selection];
        self.inverted = false;
    }

    pub fn extend(&mut self, new_selection: SelectionShape) {
        self.shapes.push(new_selection);
    }

    pub fn clear(&mut self) {
        self.shapes.clear();
        self.inverted = false;
    }

    pub fn invert(&mut self) {
        self.inverted = !self.inverted
    }

    pub fn translate(&mut self, delta: Vector2<f32>) {
        for shape in self.shapes.iter_mut() {
            match shape.shape {
                Shape::Rectangle(ref mut rect) => rect.center += delta,
            }
        }
    }

    pub fn expand(&mut self, amount_px: i32) {
        for shape in self.shapes.iter_mut() {
            match shape.shape {
                Shape::Rectangle(ref mut rect) => {
                    rect.extents.add_assign_element_wise(amount_px as f32)
                }
            }
        }
    }

    pub fn contains(&self, point: Point2<f32>) -> bool {
        let inside_selection = self.shapes.iter().any(|shape| match shape.shape {
            Shape::Rectangle(area) => area.contains_point(point.clone()),
        });

        if self.inverted {
            return !inside_selection;
        } else {
            return inside_selection;
        }
    }
}

#[cfg(test)]
mod test {
    use cgmath::{point2, vec2};
    use framework::Box2d;

    use crate::selection::{selection_shape::Shape, SelectionAddition::*};

    use super::{Selection, SelectionShape};

    #[test]
    pub fn assert_two_rect_contains_point() {
        let mut selection = Selection::default();
        selection.extend(SelectionShape {
            shape: Shape::Rectangle(Box2d {
                center: point2(10.0, 10.0),
                extents: vec2(5.0, 5.0),
            }),
            mode: Add,
        });
        selection.extend(SelectionShape {
            shape: Shape::Rectangle(Box2d {
                center: point2(10.0, -10.0),
                extents: vec2(5.0, 5.0),
            }),
            mode: Add,
        });

        assert!(selection.contains(point2(12.5, 12.5)));
        assert!(selection.contains(point2(12.5, -12.5)));
    }
    #[test]
    pub fn assert_inverted_two_rect_contains_point() {
        let mut selection = Selection::default();
        selection.invert();
        selection.extend(SelectionShape {
            shape: Shape::Rectangle(Box2d {
                center: point2(10.0, 10.0),
                extents: vec2(5.0, 5.0),
            }),
            mode: Add,
        });
        selection.extend(SelectionShape {
            shape: Shape::Rectangle(Box2d {
                center: point2(10.0, -10.0),
                extents: vec2(5.0, 5.0),
            }),
            mode: Add,
        });

        assert!(!selection.contains(point2(12.5, 12.5)));
        assert!(selection.contains(point2(17.5, 12.5)));
    }
}
