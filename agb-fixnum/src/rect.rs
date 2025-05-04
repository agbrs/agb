use num_traits::Signed;

use crate::{FixedWidthUnsignedInteger, Number, Vector2D, vec2};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A rectangle with a position in 2d space and a 2d size
pub struct Rect<T: Number> {
    /// The position of the rectangle
    pub position: Vector2D<T>,
    /// The size of the rectangle
    pub size: Vector2D<T>,
}

impl<T: Number> Rect<T> {
    #[must_use]
    /// Creates a rectangle from it's position and size
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(2,3));
    /// assert_eq!(r.position, Vector2D::new(1,1));
    /// assert_eq!(r.size, Vector2D::new(2,3));
    /// ```
    pub fn new(position: Vector2D<T>, size: Vector2D<T>) -> Self {
        Rect { position, size }
    }

    /// Returns true if the rectangle contains the point given, note that the boundary counts as containing the rectangle.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    /// assert!(r.contains_point(Vector2D::new(1,1)));
    /// assert!(r.contains_point(Vector2D::new(2,2)));
    /// assert!(r.contains_point(Vector2D::new(3,3)));
    /// assert!(r.contains_point(Vector2D::new(4,4)));
    ///
    /// assert!(!r.contains_point(Vector2D::new(0,2)));
    /// assert!(!r.contains_point(Vector2D::new(5,2)));
    /// assert!(!r.contains_point(Vector2D::new(2,0)));
    /// assert!(!r.contains_point(Vector2D::new(2,5)));
    /// ```
    pub fn contains_point(&self, point: Vector2D<T>) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y
    }

    /// Returns true if the other rectangle touches or overlaps the first.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    ///
    /// assert!(r.touches(r));
    ///
    /// let r1 = Rect::new(Vector2D::new(2,2), Vector2D::new(3,3));
    /// assert!(r.touches(r1));
    ///
    /// let r2 = Rect::new(Vector2D::new(-10,-10), Vector2D::new(3,3));
    /// assert!(!r.touches(r2));
    /// ```
    pub fn touches(&self, other: Rect<T>) -> bool {
        self.position.x < other.position.x + other.size.x
            && self.position.x + self.size.x > other.position.x
            && self.position.y < other.position.y + other.size.y
            && self.position.y + self.size.y > other.position.y
    }

    #[must_use]
    /// Returns the rectangle that is the region that the two rectangles have in
    /// common, or [None] if they don't overlap
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    /// let r2 = Rect::new(Vector2D::new(2,2), Vector2D::new(3,3));
    ///
    /// assert_eq!(r.overlapping_rect(r2), Some(Rect::new(Vector2D::new(2,2), Vector2D::new(2,2))));
    /// ```
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(Vector2D::new(1,1), Vector2D::new(3,3));
    /// let r2 = Rect::new(Vector2D::new(-10,-10), Vector2D::new(3,3));
    ///
    /// assert_eq!(r.overlapping_rect(r2), None);
    /// ```
    pub fn overlapping_rect(&self, other: Rect<T>) -> Option<Self> {
        if !self.touches(other) {
            return None;
        }

        fn max<E: Number>(x: E, y: E) -> E {
            if x > y { x } else { y }
        }
        fn min<E: Number>(x: E, y: E) -> E {
            if x > y { y } else { x }
        }

        let top_left: Vector2D<T> = (
            max(self.position.x, other.position.x),
            max(self.position.y, other.position.y),
        )
            .into();
        let bottom_right: Vector2D<T> = (
            min(
                self.position.x + self.size.x,
                other.position.x + other.size.x,
            ),
            min(
                self.position.y + self.size.y,
                other.position.y + other.size.y,
            ),
        )
            .into();

        Some(Rect::new(top_left, bottom_right - top_left))
    }

    /// Clamps the given point to be within the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let bounding_rect = Rect::new(vec2(10, 10), vec2(10, 10));
    ///
    /// assert_eq!(bounding_rect.clamp_point(vec2(15, 15)), vec2(15, 15));
    /// assert_eq!(bounding_rect.clamp_point(vec2(0, 15)), vec2(10, 15));
    /// assert_eq!(bounding_rect.clamp_point(vec2(100, 30)), vec2(20, 20));
    /// ```
    #[inline(always)]
    pub fn clamp_point(self, point: impl Into<Vector2D<T>>) -> Vector2D<T> {
        let point = point.into();
        let top_left = self.top_left();
        let bottom_right = self.bottom_right();

        let x = point.x.clamp(top_left.x, bottom_right.x);
        let y = point.y.clamp(top_left.y, bottom_right.y);

        vec2(x, y)
    }

    /// Returns the top left point of the rectangle.
    ///
    /// Is the same as `.position`.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.top_left(), vec2(10, 10));
    /// assert_eq!(r.top_left(), r.position);
    /// ```
    #[inline(always)]
    pub fn top_left(self) -> Vector2D<T> {
        self.position
    }

    /// Returns the top right point of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.top_right(), vec2(20, 10));
    /// ```
    #[inline(always)]
    pub fn top_right(self) -> Vector2D<T> {
        self.position + vec2(self.size.x, T::zero())
    }

    /// Returns the bottom left point of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.bottom_left(), vec2(10, 20));
    /// ```
    #[inline(always)]
    pub fn bottom_left(self) -> Vector2D<T> {
        self.position + vec2(T::zero(), self.size.y)
    }

    /// Returns the bottom right point of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.bottom_right(), vec2(20, 20));
    /// ```
    #[inline(always)]
    pub fn bottom_right(self) -> Vector2D<T> {
        self.position + self.size
    }
}

impl<T: FixedWidthUnsignedInteger> Rect<T> {
    /// Iterate over the points in a rectangle in row major order.
    /// ```
    /// use agb_fixnum::{Rect, vec2};
    /// let r = Rect::new(vec2(1,1), vec2(1,2));
    ///
    /// let expected_points = vec![vec2(1,1), vec2(2,1), vec2(1,2), vec2(2,2), vec2(1,3), vec2(2,3)];
    /// let rect_points: Vec<_> = r.iter().collect();
    ///
    /// assert_eq!(rect_points, expected_points);
    /// ```
    pub fn iter(self) -> impl Iterator<Item = Vector2D<T>> {
        let mut x = self.position.x;
        let mut y = self.position.y;
        core::iter::from_fn(move || {
            if x > self.position.x + self.size.x {
                x = self.position.x;
                y = y + T::one();
                if y > self.position.y + self.size.y {
                    return None;
                }
            }

            let ret_x = x;
            x = x + T::one();

            Some(vec2(ret_x, y))
        })
    }
}

impl<T: Number + Signed> Rect<T> {
    /// Makes a rectangle that represents the equivalent location in space but with a positive size
    pub fn abs(self) -> Self {
        Self {
            position: (
                self.position.x + self.size.x.min(T::zero()),
                self.position.y + self.size.y.min(T::zero()),
            )
                .into(),
            size: self.size.abs(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    extern crate alloc;

    #[test]
    fn test_rect_iter() {
        let rect: Rect<i32> = Rect::new((5_i32, 5_i32).into(), (2_i32, 2_i32).into());
        assert_eq!(
            rect.iter().collect::<alloc::vec::Vec<_>>(),
            &[
                vec2(5, 5),
                vec2(6, 5),
                vec2(7, 5),
                vec2(5, 6),
                vec2(6, 6),
                vec2(7, 6),
                vec2(5, 7),
                vec2(6, 7),
                vec2(7, 7),
            ]
        );
    }
}
