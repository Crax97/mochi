use cgmath::{num_traits::clamp_min, Point2, Vector2};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Box2d {
    pub origin: Point2<f32>,
    pub extents: Vector2<f32>,
}

impl Default for Box2d {
    fn default() -> Self {
        Self::origin()
    }
}
impl Box2d {
    pub fn many_union<I: Iterator<Item = Box2d>>(mut boxes: I) -> Box2d {
        let first = boxes.next();
        if let Some(first) = first {
            boxes.fold(first, |acc, cur| acc.union(&cur))
        } else {
            Box2d::origin()
        }
    }
}

impl Box2d {
    pub fn origin() -> Self {
        Self {
            origin: Point2 { x: 0.0, y: 0.0 },
            extents: Vector2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn center(&self) -> Point2<f32> {
        Point2 {
            x: (self.right() - self.left()) * 0.5,
            y: (self.bottom() - self.top()) * 0.5,
        }
    }

    pub fn left(&self) -> f32 {
        self.origin.x - self.extents.x
    }
    pub fn bottom(&self) -> f32 {
        self.origin.y + self.extents.y
    }
    pub fn right(&self) -> f32 {
        self.origin.x + self.extents.x
    }
    pub fn top(&self) -> f32 {
        self.origin.y - self.extents.y
    }

    pub fn area(&self) -> f32 {
        self.extents.x * self.extents.y * 4.0
    }

    pub fn contains(&self, other: &Box2d) -> bool {
        (self.left() <= other.left() && self.right() >= other.right())
            && (self.top() <= other.top() && self.bottom() >= other.bottom())
    }

    pub fn intersect(&self, other: &Box2d) -> Option<Self> {
        if self.right() < other.left() || self.bottom() < self.top() {
            return None;
        }

        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let width = (self.right() - left).min(other.right() - left) * 0.5;
        let height = (self.bottom() - top).min(other.bottom() - top) * 0.5;
        if width == 0.0 || height == 0.0 {
            return None;
        }
        return Some(Self {
            origin: Point2 {
                x: left + width,
                y: top + height,
            },
            extents: Vector2 {
                x: width,
                y: height,
            },
        });
    }
    pub fn union(&self, other: &Box2d) -> Self {
        if self.contains(other) {
            return *self;
        }
        if other.contains(self) {
            return *other;
        }

        let x = (self.origin.x + other.origin.x) * 0.5;
        let y = (self.origin.y + other.origin.y) * 0.5;
        let left = other.left().min(self.left());
        let right = other.right().max(self.right());
        let top = other.top().min(self.top());
        let bottom = other.bottom().max(self.bottom());
        let width = (right - left) * 0.5;
        let height = (bottom - top) * 0.5;

        return Self {
            origin: Point2 { x, y },
            extents: Vector2 {
                x: width,
                y: height,
            },
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn contains() {
        let b = Box2d::origin();
        let o = Box2d {
            extents: Vector2 { x: 100.0, y: 100.0 },
            ..Default::default()
        };
        // assert!(!b.contains(&o));
        assert!(o.contains(&b));
    }
    #[test]
    fn contains_not_intersect() {
        let b = Box2d {
            extents: Vector2 { x: 100.0, y: 100.0 },
            ..Default::default()
        };
        let o = Box2d {
            extents: Vector2 { x: 100.0, y: 100.0 },
            origin: Point2 { x: 110.0, y: 0.0 },
        };
        assert!(!b.contains(&o));
    }
    #[test]
    fn contains_intersect() {
        let b = Box2d {
            extents: Vector2 { x: 100.0, y: 100.0 },
            ..Default::default()
        };
        let o = Box2d {
            extents: Vector2 { x: 100.0, y: 100.0 },
            origin: Point2 { x: 50.0, y: 50.0 },
        };
        assert!(!b.contains(&o));
    }
    #[test]
    fn test_intersect_from_origin() {
        let b = Box2d::origin();
        let o = Box2d::origin();
        let i = b.intersect(&o);
        assert!(i.is_none());
    }
    #[test]
    fn test_intersect_from_point() {
        let b = Box2d {
            origin: Point2 { x: 0.0, y: 0.0 },
            extents: Vector2 { x: 100.0, y: 100.0 },
        };
        let o = Box2d {
            origin: Point2 { x: 20.0, y: 20.0 },
            extents: Vector2 { x: 10.0, y: 10.0 },
        };
        let i = b.intersect(&o).unwrap();
        assert_eq!(i, o);
    }
    #[test]
    fn test_intersect_edge() {
        let b = Box2d {
            origin: Point2 { x: 0.0, y: 0.0 },
            extents: Vector2 { x: 100.0, y: 100.0 },
        };
        let o = Box2d {
            origin: Point2 { x: 95.0, y: 95.0 },
            extents: Vector2 { x: 5.0, y: 5.0 },
        };
        let i = b.intersect(&o).unwrap();
        assert_eq!(i, o);
    }
    #[test]
    fn test_intersect_outside() {
        let b = Box2d {
            origin: Point2 { x: 0.0, y: 0.0 },
            extents: Vector2 { x: 100.0, y: 100.0 },
        };
        let o = Box2d {
            origin: Point2 { x: 111.0, y: 111.0 },
            extents: Vector2 { x: 10.0, y: 10.0 },
        };
        let i = b.intersect(&o);
        assert!(i.is_none());
    }
    #[test]
    fn test_union_from_origin() {
        let b = Box2d::origin();
        let o = Box2d::origin();
        let i = b.union(&o);
        assert_eq!(
            i,
            Box2d {
                origin: Point2 { x: 0.0, y: 0.0 },
                extents: Vector2 { x: 0.0, y: 0.0 }
            }
        );
    }
    #[test]
    fn test_union_inside() {
        let b = Box2d {
            origin: Point2 { x: 0.0, y: 0.0 },
            extents: Vector2 { x: 100.0, y: 100.0 },
        };
        let o = Box2d {
            origin: Point2 { x: 20.0, y: 20.0 },
            extents: Vector2 { x: 10.0, y: 10.0 },
        };
        let i = b.union(&o);
        assert_eq!(i, b);
    }
    #[test]
    fn test_union_edge() {
        let b = Box2d {
            origin: Point2 { x: 0.0, y: 0.0 },
            extents: Vector2 { x: 100.0, y: 100.0 },
        };
        let o = Box2d {
            origin: Point2 { x: 95.0, y: 95.0 },
            extents: Vector2 { x: 5.0, y: 5.0 },
        };
        let i = b.union(&o);
        assert_eq!(i, b);
    }
    #[test]
    fn test_union_outside() {
        let b = Box2d {
            origin: Point2 { x: 0.0, y: 0.0 },
            extents: Vector2 { x: 100.0, y: 100.0 },
        };
        let o = Box2d {
            origin: Point2 { x: 120.0, y: 120.0 },
            extents: Vector2 { x: 10.0, y: 10.0 },
        };
        let i = b.union(&o);
        assert_eq!(
            i,
            Box2d {
                origin: Point2 { x: 60.0, y: 60.0 },
                extents: Vector2 { x: 115.0, y: 115.0 },
            }
        );
    }
}
