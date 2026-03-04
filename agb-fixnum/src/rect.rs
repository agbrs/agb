use core::ops::{Add, AddAssign, Div, Mul, Sub};

use num_traits::{One, Signed, Zero};

use crate::{FixedWidthUnsignedInteger, Vector2D, vec2};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A rectangle with a position in 2d space and a 2d size
pub struct Rect<T> {
    /// The position of the rectangle
    pub position: Vector2D<T>,
    /// The size of the rectangle
    pub size: Vector2D<T>,
}

impl<T> Rect<T> {
    #[inline(always)]
    #[must_use]
    /// Creates a rectangle from it's position and size
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(1, 1), vec2(2, 3));
    /// assert_eq!(r.position, vec2(1, 1));
    /// assert_eq!(r.size, vec2(2, 3));
    /// ```
    pub fn new(position: Vector2D<T>, size: Vector2D<T>) -> Self {
        Rect { position, size }
    }
}

impl<T> Rect<T> {
    /// Scales up the rectangle by the given amount about the origin.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(5, 10), vec2(15, 20));
    /// assert_eq!(r.scale(2), Rect::new(vec2(10, 20), vec2(30, 40)));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn scale<Rhs, O>(self, rhs: Rhs) -> Rect<O>
    where
        T: Mul<Rhs, Output = O>,
        Rhs: Copy,
    {
        Rect::new(self.position * rhs, self.size * rhs)
    }

    /// Scales down the rectangle by the given amount about the origin.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 20), vec2(30, 40));
    /// assert_eq!(r.scale_down(2), Rect::new(vec2(5, 10), vec2(15, 20)));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn scale_down<Rhs, O>(self, rhs: Rhs) -> Rect<O>
    where
        T: Div<Rhs, Output = O>,
        Rhs: Copy,
    {
        Rect::new(self.position / rhs, self.size / rhs)
    }

    /// Translate the rectangle by the given amount
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 20), vec2(30, 40));
    /// assert_eq!(r.translate(vec2(3, 5)), Rect::new(vec2(13, 25), vec2(30, 40)));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn translate(self, amount: Vector2D<T>) -> Self
    where
        T: Add<Output = T> + Copy,
    {
        Rect::new(self.position + amount, self.size)
    }
}

impl<T> Rect<T>
where
    T: Add<T, Output = T> + Ord + Copy,
{
    /// Returns true if the rectangle contains the point given, note that the boundary counts part of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(1, 1), vec2(3, 3));
    /// assert!(r.contains_point(vec2(1, 1)));
    /// assert!(r.contains_point(vec2(2, 2)));
    /// assert!(r.contains_point(vec2(3, 3)));
    /// assert!(r.contains_point(vec2(4, 4)));
    ///
    /// assert!(!r.contains_point(vec2(0, 2)));
    /// assert!(!r.contains_point(vec2(5, 2)));
    /// assert!(!r.contains_point(vec2(2, 0)));
    /// assert!(!r.contains_point(vec2(2, 5)));
    /// ```
    #[must_use]
    pub fn contains_point(&self, point: Vector2D<T>) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y
    }

    /// Returns true if the other rectangle touches or overlaps the first.
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(1, 1), vec2(3, 3));
    ///
    /// assert!(r.touches(r));
    ///
    /// let r1 = Rect::new(vec2(2, 3), vec2(3, 3));
    /// assert!(r.touches(r1));
    ///
    /// let r2 = Rect::new(vec2(-10, -10), vec2(3, 3));
    /// assert!(!r.touches(r2));
    /// ```
    #[must_use]
    pub fn touches(&self, other: Rect<T>) -> bool {
        self.position.x < other.position.x + other.size.x
            && self.position.x + self.size.x > other.position.x
            && self.position.y < other.position.y + other.size.y
            && self.position.y + self.size.y > other.position.y
    }
}

impl<T> Rect<T>
where
    T: Add<T, Output = T> + Ord + Copy + AddAssign<T> + Sub<Output = T>,
{
    #[must_use]
    /// Returns the rectangle that is the region that the two rectangles have in
    /// common, or [None] if they don't overlap
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(1, 1), vec2(3, 3));
    /// let r2 = Rect::new(vec2(2, 2), vec2(3, 3));
    ///
    /// assert_eq!(r.overlapping_rect(r2), Some(Rect::new(vec2(2, 2), vec2(2, 2))));
    /// ```
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(1, 1), vec2(3, 3));
    /// let r2 = Rect::new(vec2(-10, -10), vec2(3, 3));
    ///
    /// assert_eq!(r.overlapping_rect(r2), None);
    /// ```
    pub fn overlapping_rect(&self, other: Rect<T>) -> Option<Self> {
        if !self.touches(other) {
            return None;
        }

        let top_left = self.position.max(other.position);
        let bottom_right = self.bottom_right().min(other.bottom_right());

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
    #[must_use]
    pub fn clamp_point(self, point: impl Into<Vector2D<T>>) -> Vector2D<T> {
        point.into().clamp(self.top_left(), self.bottom_right())
    }
}

impl<T> Rect<T> {
    /// Returns the width of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(5, 10), vec2(20, 30));
    /// assert_eq!(r.width(), 20);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn width(self) -> T {
        self.size.x
    }

    /// Returns the height of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(5, 10), vec2(20, 30));
    /// assert_eq!(r.height(), 30);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn height(self) -> T {
        self.size.y
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
    #[must_use]
    pub fn top_left(self) -> Vector2D<T> {
        self.position
    }

    /// Returns the `x` coordinate of the left hand side of the rectangle
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 20), vec2(20, 25));
    /// assert_eq!(r.left(), 10);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn left(self) -> T {
        self.position.x
    }

    /// Returns the `y` coordinate of the top of the rectangle
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 20), vec2(20, 25));
    /// assert_eq!(r.top(), 20);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn top(self) -> T {
        self.position.y
    }
}

impl<T> Rect<T>
where
    T: AddAssign<T> + Add<T, Output = T>,
{
    /// Returns the `x` coordinate of the right hand side of the rectangle
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 20), vec2(20, 25));
    /// assert_eq!(r.right(), 30);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn right(self) -> T {
        self.position.x + self.size.x
    }

    /// Returns the `y` coordinate of the top of the rectangle
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 20), vec2(20, 25));
    /// assert_eq!(r.bottom(), 45);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn bottom(self) -> T {
        self.position.y + self.size.y
    }

    /// Returns the top right point of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.top_right(), vec2(20, 10));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn top_right(self) -> Vector2D<T> {
        let mut pos = self.position;
        pos.x += self.size.x;
        pos
    }

    /// Returns the bottom left point of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.bottom_left(), vec2(10, 20));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn bottom_left(self) -> Vector2D<T> {
        let mut pos = self.position;
        pos.y += self.size.y;
        pos
    }

    /// Returns the bottom right point of the rectangle.
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.bottom_right(), vec2(20, 20));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn bottom_right(self) -> Vector2D<T> {
        self.position + self.size
    }
}

impl<T> Rect<T>
where
    T: Add<T, Output = T> + Div<T, Output = T> + One + Copy,
{
    /// Returns the centre point of the rectangle
    ///
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.centre(), vec2(15, 15));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn centre(self) -> Vector2D<T> {
        self.position + self.size / (T::one() + T::one())
    }

    /// Returns the centre point of the left edge of the rectangle
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.left_centre(), vec2(10, 15));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn left_centre(self) -> Vector2D<T> {
        let pos = self.position;
        vec2(pos.x, pos.y + self.size.y / (T::one() + T::one()))
    }

    /// Returns the centre point of the right edge of the rectangle
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.right_centre(), vec2(20, 15));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn right_centre(self) -> Vector2D<T> {
        let pos = self.position;
        vec2(
            pos.x + self.size.x,
            pos.y + self.size.y / (T::one() + T::one()),
        )
    }

    /// Returns the centre point of the top edge of the rectangle
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.top_centre(), vec2(15, 10));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn top_centre(self) -> Vector2D<T> {
        let pos = self.position;
        vec2(pos.x + self.size.x / (T::one() + T::one()), pos.y)
    }

    /// Returns the centre point of the bottom edge of the rectangle
    /// ```
    /// # use agb_fixnum::*;
    /// let r = Rect::new(vec2(10, 10), vec2(10, 10));
    /// assert_eq!(r.bottom_centre(), vec2(15, 20));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn bottom_centre(self) -> Vector2D<T> {
        let pos = self.position;
        vec2(
            pos.x + self.size.x / (T::one() + T::one()),
            pos.y + self.size.y,
        )
    }
}

impl<T: FixedWidthUnsignedInteger> Rect<T> {
    /// Iterate over the points in a rectangle in row major order.
    /// ```
    /// use agb_fixnum::{Rect, vec2};
    /// let r = Rect::new(vec2(1, 1), vec2(1, 2));
    ///
    /// let expected_points = vec![vec2(1, 1), vec2(2, 1), vec2(1, 2), vec2(2, 2), vec2(1, 3), vec2(2, 3)];
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

impl<T: Ord + Zero + Signed + Copy> Rect<T> {
    /// Makes a rectangle that represents the equivalent location in space but with a positive size
    ///
    /// ```
    /// use agb_fixnum::{Rect, vec2};
    ///
    /// let r: Rect<i32> = Rect::new(vec2(5, 5), vec2(-3, -2));
    ///
    /// let normalized_rect = Rect::new(vec2(2, 3), vec2(3, 2));
    ///
    /// // even though they represent the same area, they are not consider equivalent
    /// assert_ne!(r, normalized_rect);
    /// // unless you normalize the one with negative area
    /// assert_eq!(r.abs(), normalized_rect);
    /// ```
    #[must_use]
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

    #[test]
    fn test_rect_contains_point() {
        let rect1: Rect<i32> = Rect::new(Vector2D::new(-1, -1), Vector2D::new(2, 2));
        assert!(rect1.contains_point(Vector2D::default()));
        let rect2: Rect<i32> = Rect::new(Vector2D::new(1, 1), Vector2D::new(2, 2));
        assert!(!rect2.contains_point(Vector2D::default()));
    }

    #[test]
    fn test_rect_touches() {
        let a: Rect<i32> = Rect::new(Vector2D::new(0, 0), Vector2D::new(2, 2));
        let b: Rect<i32> = Rect::new(Vector2D::new(1, 1), Vector2D::new(2, 2));
        let c: Rect<i32> = Rect::new(Vector2D::new(3, 3), Vector2D::new(1, 1));
        assert!(a.touches(b));
        assert!(!a.touches(c));
    }

    #[test]
    fn test_rect_overlapping() {
        let a: Rect<i32> = Rect::new(Vector2D::new(0, 0), Vector2D::new(2, 2));
        let b: Rect<i32> = Rect::new(Vector2D::new(3, 3), Vector2D::new(1, 1));
        assert_eq!(a.overlapping_rect(b), None);
        let d: Rect<i32> = Rect::new(Vector2D::new(1, 1), Vector2D::new(2, 2));
        assert_eq!(
            a.overlapping_rect(d),
            Some(Rect::new(Vector2D::new(1, 1), Vector2D::new(1, 1)))
        );
    }

    #[test]
    fn test_rect_clamp_point() {
        let rect: Rect<i32> = Rect::new(Vector2D::new(0, 0), Vector2D::new(10, 10));
        assert_eq!(rect.clamp_point(Vector2D::new(5, 5)), Vector2D::new(5, 5));
        assert_eq!(
            rect.clamp_point(Vector2D::new(-5, 15)),
            Vector2D::new(0, 10)
        );
    }

    #[test]
    fn test_rect_top_left() {
        let rect: Rect<i32> = Rect::new(Vector2D::new(3, 4), Vector2D::new(1, 1));
        assert_eq!(rect.top_left(), Vector2D::new(3, 4));
    }

    #[test]
    fn test_rect_top_right() {
        let rect: Rect<i32> = Rect::new(Vector2D::new(1, 2), Vector2D::new(3, 4));
        assert_eq!(rect.top_right(), Vector2D::new(4, 2));
    }

    #[test]
    fn test_rect_bottom_left() {
        let rect: Rect<i32> = Rect::new(Vector2D::new(1, 2), Vector2D::new(3, 4));
        assert_eq!(rect.bottom_left(), Vector2D::new(1, 6));
    }

    #[test]
    fn test_rect_bottom_right() {
        let rect: Rect<i32> = Rect::new(Vector2D::new(1, 2), Vector2D::new(3, 4));
        assert_eq!(rect.bottom_right(), Vector2D::new(4, 6));
    }

    #[test]
    fn test_rect_centre() {
        let rect: Rect<i32> = Rect::new(Vector2D::new(0, 0), Vector2D::new(4, 6));
        assert_eq!(rect.centre(), Vector2D::new(2, 3));
    }

    #[test]
    fn test_rect_abs() {
        let rect = Rect::new(Vector2D::new(1_i32, 2_i32), Vector2D::new(3_i32, 4_i32));
        let result = rect.abs();
        assert_eq!(result.position, Vector2D::new(1_i32, 2_i32));
        assert_eq!(result.size, Vector2D::new(3_i32, 4_i32));
    }
}
