use std::ops::Deref;

use cgmath::{
    num_traits::{Num, NumCast},
    Point2, Vector2,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Box2d<T: Num + Copy + NumCast + Ord> {
    pub origin: Point2<T>,
    pub size: Vector2<T>,
}

impl<T: Num + Copy + NumCast + Ord> Default for Box2d<T> {
    fn default() -> Self {
        Self::origin()
    }
}
impl<T: Num + Copy + NumCast + Ord> Box2d<T> {
    pub fn many_union<I: Iterator<Item = Box2d<T>>>(mut boxes: I) -> Box2d<T> {
        let first = boxes.next();
        if let Some(first) = first {
            boxes.fold(first, |acc, cur| acc.union(&cur))
        } else {
            Box2d::origin()
        }
    }
}

impl<T: Num + Copy + NumCast + Ord> Box2d<T> {
    pub fn origin() -> Self {
        Self {
            origin: Point2 {
                x: T::zero(),
                y: T::zero(),
            },
            size: Vector2 {
                x: T::zero(),
                y: T::zero(),
            },
        }
    }

    pub fn cast<U: Num + Copy + NumCast + Ord>(self) -> Option<Box2d<U>> {
        Some(Box2d {
            origin: self.origin.cast::<U>()?,
            size: self.size.cast::<U>()?,
        })
    }

    pub fn left(&self) -> T {
        self.origin.x
    }
    pub fn bottom(&self) -> T {
        self.origin.y + self.size.y
    }
    pub fn right(&self) -> T {
        self.origin.x + self.size.x
    }
    pub fn top(&self) -> T {
        self.origin.y
    }

    pub fn area(&self) -> T {
        self.size.x * self.size.y
    }

    pub fn contains(&self, other: &Box2d<T>) -> bool {
        (self.left() <= other.left() && self.right() >= other.right())
            && (self.top() <= other.top() && self.bottom() >= other.bottom())
    }

    pub fn intersect(&self, other: &Box2d<T>) -> Option<Self>
    where
        T: Ord,
    {
        if self.right() < other.left() || self.bottom() < self.top() {
            return None;
        }

        let intersection_x = self.left().max(other.left());
        let intersection_y = self.top().max(other.top());
        let width = (self.right() - intersection_x).min(other.right() - intersection_x);
        let height = (self.bottom() - intersection_y).min(other.bottom() - intersection_y);
        if width == T::zero() || height == T::zero() {
            return None;
        }
        return Some(Self {
            origin: Point2 {
                x: intersection_x,
                y: intersection_y,
            },
            size: Vector2 {
                x: width,
                y: height,
            },
        });
    }
    pub fn union(&self, other: &Box2d<T>) -> Self
    where
        T: Ord,
    {
        if self.contains(other) {
            return *self;
        }
        if other.contains(self) {
            return *other;
        }

        let (delta_x, size_x) = if other.left() > self.left() {
            (other.left() - self.left(), other.size.x)
        } else {
            (self.left() - other.left(), self.size.x)
        };
        let (delta_y, size_y) = if other.top() > self.top() {
            (other.top() - self.top(), other.size.y)
        } else {
            (self.top() - other.top(), self.size.y)
        };

        let x = self.left().min(other.left());
        let y = self.top().min(other.top());
        let width = self.right().max(other.right()) - x;
        let height = self.bottom().max(other.bottom()) - y;

        return Self {
            origin: Point2 { x, y },
            size: Vector2 {
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
        let b = Box2d::<u32>::origin();
        let o = Box2d::<u32> {
            size: Vector2 { x: 100, y: 100 },
            ..Default::default()
        };
        assert!(!b.contains(&o));
        assert!(o.contains(&b));
    }
    #[test]
    fn contains_not_intersect() {
        let b = Box2d::<u32> {
            size: Vector2 { x: 100, y: 100 },
            ..Default::default()
        };
        let o = Box2d::<u32> {
            size: Vector2 { x: 100, y: 100 },
            origin: Point2 { x: 110, y: 0 },
        };
        assert!(!b.contains(&o));
    }
    #[test]
    fn contains_intersect() {
        let b = Box2d::<u32> {
            size: Vector2 { x: 100, y: 100 },
            ..Default::default()
        };
        let o = Box2d::<u32> {
            size: Vector2 { x: 100, y: 100 },
            origin: Point2 { x: 50, y: 50 },
        };
        assert!(!b.contains(&o));
    }
    #[test]
    fn test_intersect_from_origin() {
        let b = Box2d::<u32>::origin();
        let o = Box2d::<u32>::origin();
        let i = b.intersect(&o);
        assert!(i.is_none());
    }
    #[test]
    fn test_intersect_from_point() {
        let b = Box2d::<u32> {
            origin: Point2 { x: 0, y: 0 },
            size: Vector2 { x: 100, y: 100 },
        };
        let o = Box2d::<u32> {
            origin: Point2 { x: 20, y: 20 },
            size: Vector2 { x: 10, y: 10 },
        };
        let i = b.intersect(&o).unwrap();
        assert_eq!(i, o);
    }
    #[test]
    fn test_intersect_edge() {
        let b = Box2d::<u32> {
            origin: Point2 { x: 0, y: 0 },
            size: Vector2 { x: 100, y: 100 },
        };
        let o = Box2d::<u32> {
            origin: Point2 { x: 95, y: 95 },
            size: Vector2 { x: 10, y: 10 },
        };
        let i = b.intersect(&o).unwrap();
        assert_eq!(
            i,
            Box2d::<u32> {
                origin: Point2 { x: 95, y: 95 },
                size: Vector2 { x: 5, y: 5 },
            }
        );
    }
    #[test]
    fn test_intersect_outside() {
        let b = Box2d::<u32> {
            origin: Point2 { x: 0, y: 0 },
            size: Vector2 { x: 100, y: 100 },
        };
        let o = Box2d::<u32> {
            origin: Point2 { x: 101, y: 100 },
            size: Vector2 { x: 10, y: 10 },
        };
        let i = b.intersect(&o);
        assert!(i.is_none());
    }
    #[test]
    fn test_union_from_origin() {
        let b = Box2d::<u32>::origin();
        let o = Box2d::<u32>::origin();
        let i = b.union(&o);
        assert_eq!(
            i,
            Box2d {
                origin: Point2 { x: 0, y: 0 },
                size: Vector2 { x: 0, y: 0 }
            }
        );
    }
    #[test]
    fn test_union_inside() {
        let b = Box2d::<u32> {
            origin: Point2 { x: 0, y: 0 },
            size: Vector2 { x: 100, y: 100 },
        };
        let o = Box2d::<u32> {
            origin: Point2 { x: 20, y: 20 },
            size: Vector2 { x: 10, y: 10 },
        };
        let i = b.union(&o);
        assert_eq!(i, b);
    }
    #[test]
    fn test_union_edge() {
        let b = Box2d::<u32> {
            origin: Point2 { x: 0, y: 0 },
            size: Vector2 { x: 100, y: 100 },
        };
        let o = Box2d::<u32> {
            origin: Point2 { x: 95, y: 95 },
            size: Vector2 { x: 10, y: 10 },
        };
        let i = b.union(&o);
        assert_eq!(
            i,
            Box2d::<u32> {
                origin: Point2 { x: 0, y: 0 },
                size: Vector2 { x: 105, y: 105 },
            }
        );
    }
    #[test]
    fn test_union_outside() {
        let b = Box2d::<u32> {
            origin: Point2 { x: 0, y: 0 },
            size: Vector2 { x: 100, y: 100 },
        };
        let o = Box2d::<u32> {
            origin: Point2 { x: 120, y: 120 },
            size: Vector2 { x: 10, y: 10 },
        };
        let i = b.union(&o);
        assert_eq!(
            i,
            Box2d::<u32> {
                origin: Point2 { x: 0, y: 0 },
                size: Vector2 { x: 130, y: 130 },
            }
        );
    }
}
