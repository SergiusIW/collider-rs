// Copyright 2016-2017 Matthew D. Michelotti
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ops::{Add, Sub, Mul, Neg};
use std::cmp::Ordering;
use geom::{Vec2, DirVec2, vec2, Card};
use float::n64;

mod normals;

/// Enumeration of kinds of shapes used by Collider.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum ShapeKind {
    /// Circle.  Requires width and height to match.
    Circle,
    /// Axis-aligned rectangle.
    Rect
}

/// Represents a shape, without any position.
///
/// Each shape has a `width` and `height`, which are allowed to be negative.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Shape {
    kind: ShapeKind,
    dims: Vec2
}

impl Shape {
    /// Constructs a new shape with the given `kind` and `dims` (width and height dimensions).
    ///
    /// If `kind` is `Circle`, then the width and height must match.
    pub fn new(kind: ShapeKind, dims: Vec2) -> Shape {
        assert!(kind == ShapeKind::Rect || dims.x == dims.y, "circle width must equal height");
        Shape { kind : kind, dims : dims }
    }

    /// Constructs a new circle shape, using `diam` as the width and height.
    #[inline]
    pub fn new_circle(diam: f64) -> Shape {
        Shape::new(ShapeKind::Circle, vec2(diam, diam))
    }

    /// Constructs a new axis-aligned rectangle shape with the given `dims` (width and height dimensions).
    #[inline]
    pub fn new_rect(dims: Vec2) -> Shape {
        Shape::new(ShapeKind::Rect, dims)
    }

    /// Constructs a new axis-aligned square shape with the given `width`.
    #[inline]
    pub fn new_square(width: f64) -> Shape {
        Shape::new(ShapeKind::Rect, vec2(width, width))
    }

    /// Returns the kind of shape.
    #[inline]
    pub fn kind(&self) -> ShapeKind {
        self.kind
    }

    /// Returns the dims of the shape.
    #[inline]
    pub fn dims(&self) -> Vec2 {
        self.dims
    }

    /// Returns a shape with width and height swapped.
    #[inline]
    pub fn turn(&self) -> Shape {
        Shape { kind : self.kind, dims : vec2(self.dims.y, self.dims.x) }
    }
}

impl Mul<f64> for Shape {
    type Output = Shape;
    fn mul(self, rhs: f64) -> Shape {
        Shape::new(self.kind, self.dims * rhs)
    }
}

impl Add for Shape {
    type Output = Shape;
    fn add(self, rhs: Shape) -> Shape {
        assert!(self.kind == rhs.kind, "cannot add circle with rect");
        Shape::new(self.kind, self.dims + rhs.dims)
    }
}

impl Sub for Shape {
    type Output = Shape;
    fn sub(self, rhs: Shape) -> Shape {
        assert!(self.kind == rhs.kind, "cannot add circle with rect");
        Shape::new(self.kind, self.dims - rhs.dims)
    }
}

impl Neg for Shape {
    type Output = Shape;
    fn neg(self) -> Shape {
        Shape::new(self.kind, -self.dims)
    }
}

/// Represents a shape with a position.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct PlacedShape {
    /// The position of the center of the shape.
    pub pos: Vec2,
    /// The shape.
    pub shape: Shape
}

impl PlacedShape {
    /// Constructs a new `PlacedShape` with the given `pos` and `shape`.
    #[inline]
    pub fn new(pos: Vec2, shape: Shape) -> PlacedShape {
        PlacedShape { pos : pos, shape : shape }
    }

    /// Shorthand for `self.shape.kind()`
    #[inline]
    pub fn kind(&self) -> ShapeKind {
        self.shape.kind()
    }

    /// Shorthand for `self.shape.dims()`
    #[inline]
    pub fn dims(&self) -> Vec2 {
        self.shape.dims()
    }

    /// Returns the lowest y coordinate of the `PlacedShape`.
    pub fn bottom(&self) -> f64 {
        self.pos.y - self.shape.dims().y * 0.5
    }

    /// Returns the lowest x coordinate of the `PlacedShape`.
    pub fn left(&self) -> f64 {
        self.pos.x - self.shape.dims().x * 0.5
    }

    /// Returns the highest y coordinate of the `PlacedShape`.
    pub fn top(&self) -> f64 {
        self.pos.y + self.shape.dims().y * 0.5
    }

    /// Returns the highest x coordinate of the `PlacedShape`.
    pub fn right(&self) -> f64 {
        self.pos.x + self.shape.dims.x * 0.5
    }

    /// Returns `true` if the two shapes overlap, subject to negligible numerical error.
    pub fn overlaps(&self, other: &PlacedShape) -> bool {
        self.normal_from(other).len() >= 0.0
    }

    /// Returns a normal vector that points in the direction from `other` to `self`.
    ///
    /// The length of this vector is the minimum distance that `self` would need to
    /// be moved along this direction so that it is no longer overlapping `other`.
    /// If the shapes are not overlappingt to begin with, then the length of this vector
    /// is negative, and describes the minimum distance that `self` would need to
    /// be moved so that it is just overlapping `other`.
    ///
    /// (As a minor caveat,
    /// when computing the normal between two `Rect` shapes,
    /// the direction will always be axis-aligned.)
    pub fn normal_from(&self, other: &PlacedShape) -> DirVec2 {
        match (self.kind(), other.kind()) {
            (ShapeKind::Rect, ShapeKind::Rect) => normals::rect_rect_normal(self, other),
            (ShapeKind::Rect, ShapeKind::Circle) => normals::rect_circle_normal(self, other),
            (ShapeKind::Circle, ShapeKind::Rect) => normals::rect_circle_normal(other, self).flip(),
            (ShapeKind::Circle, ShapeKind::Circle) => normals::circle_circle_normal(self, other)
        }
    }

    pub(crate) fn sector(&self, point: Vec2) -> Sector {
        let x = interval_sector(self.left(), self.right(), point.x);
        let y = interval_sector(self.bottom(), self.top(), point.y);
        Sector::new(x, y)
    }

    pub(crate) fn corner(&self, sector: Sector) -> Vec2 {
        let x = match sector.x {
            Ordering::Less => self.left(),
            Ordering::Greater => self.right(),
            Ordering::Equal => panic!("expected corner sector")
        };
        let y = match sector.y {
            Ordering::Less => self.bottom(),
            Ordering::Greater => self.top(),
            Ordering::Equal => panic!("expected corner sector")
        };
        vec2(x, y)
    }

    pub(crate) fn card_overlap(&self, src: &PlacedShape, card: Card) -> f64 {
        edge(src, card) + edge(self, card.flip())
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.pos == Vec2::zero() && self.shape.dims() == Vec2::zero()
    }

    pub(crate) fn as_rect(&self) -> PlacedShape {
        PlacedShape::new(self.pos, Shape::new_rect(self.shape.dims()))
    }

    pub(crate) fn bounding_box(&self, other: &PlacedShape) -> PlacedShape {
        let right = self.right().max(other.right());
        let top = self.top().max(other.top());
        let left = self.left().min(other.left());
        let bottom = self.bottom().min(other.bottom());

        let shape = Shape::new_rect(vec2(right - left, top - bottom));
        let pos = vec2(left + shape.dims().x * 0.5, bottom + shape.dims().y * 0.5);
        PlacedShape::new(pos, shape)
    }

    pub(crate) fn max_edge(&self) -> f64 {
        Card::vals().iter()
                    .map(|&card| edge(self, card).abs())
                    .max_by_key(|&edge| n64(edge))
                    .unwrap()
    }
}

fn edge(shape: &PlacedShape, card: Card) -> f64 {
    match card {
        Card::Bottom => -shape.bottom(),
        Card::Left => -shape.left(),
        Card::Top => shape.top(),
        Card::Right => shape.right()
    }
}

fn interval_sector(left: f64, right: f64, val: f64) -> Ordering {
    if val < left {
        Ordering::Less
    } else if val > right {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub(crate) struct Sector {
    x: Ordering,
    y: Ordering
}

impl Sector {
    pub fn new(x: Ordering, y: Ordering) -> Sector {
        Sector { x: x, y: y }
    }

    pub fn is_corner(&self) -> bool {
        self.x != Ordering::Equal && self.y != Ordering::Equal
    }
}


impl Mul<f64> for PlacedShape {
    type Output = PlacedShape;
    fn mul(self, rhs: f64) -> PlacedShape {
        PlacedShape::new(self.pos * rhs, self.shape * rhs)
    }
}

impl Add for PlacedShape {
    type Output = PlacedShape;
    fn add(self, rhs: PlacedShape) -> PlacedShape {
        PlacedShape::new(self.pos + rhs.pos, self.shape + rhs.shape)
    }
}

impl Sub for PlacedShape {
    type Output = PlacedShape;
    fn sub(self, rhs: PlacedShape) -> PlacedShape {
        PlacedShape::new(self.pos - rhs.pos, self.shape - rhs.shape)
    }
}

impl Neg for PlacedShape {
    type Output = PlacedShape;
    fn neg(self) -> PlacedShape {
        PlacedShape::new(-self.pos, -self.shape)
    }
}

#[cfg(test)]
mod tests {
    use geom::*;

    #[test]
    fn test_circle_ops() {
        let shape_1 = PlacedShape::new(vec2(3.0, 5.0), Shape::new_circle(2.0));
        let shape_2 = PlacedShape::new(vec2(1.0, 2.0), Shape::new_circle(-1.0));
        let shape_3 = PlacedShape::new(vec2(-1.0, 3.0), Shape::new_circle(4.0));
        let expected = PlacedShape::new(vec2(9.0, 10.0), Shape::new_circle(3.0));
        assert!(shape_1 * 3.0 + -shape_2 - shape_3 == expected);
    }

    #[test]
    fn test_rect_ops() {
        let shape_1 = PlacedShape::new(vec2(3.0, 5.0), Shape::new_rect(vec2(2.0, 5.0)));
        let shape_2 = PlacedShape::new(vec2(1.0, 2.0), Shape::new_rect(vec2(-1.0, 2.0)));
        let shape_3 = PlacedShape::new(vec2(-1.0, 3.0), Shape::new_rect(vec2(4.0, 1.0)));
        let expected = PlacedShape::new(vec2(9.0, 10.0), Shape::new_rect(vec2(3.0, 12.0)));
        assert!(shape_1*3.0 + -shape_2 - shape_3 == expected);
    }

    #[test]
    #[should_panic]
    fn test_circle_rect_add() {
        Shape::new_rect(vec2(1.0, 2.0)) + Shape::new_circle(3.0);
    }

    #[test]
    #[should_panic]
    fn test_circle_rect_sub() {
        Shape::new_rect(vec2(1.0, 2.0)) - Shape::new_circle(3.0);
    }

    #[test]
    fn test_edges() {
        let shape = PlacedShape::new(vec2(3.0, 5.0), Shape::new_rect(vec2(4.0, 6.0)));
        assert!(shape.left() == 1.0);
        assert!(shape.bottom() == 2.0);
        assert!(shape.right() == 5.0);
        assert!(shape.top() == 8.0);
    }

    #[test]
    fn test_rect_rect_normal() {
        let src = PlacedShape::new(vec2(1.0, 1.0), Shape::new_rect(vec2(4.0, 4.0)));
        let dst = PlacedShape::new(vec2(2.0, 1.5), Shape::new_rect(vec2(8.0, 8.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(1.0, 0.0), 5.0));
        let dst = PlacedShape::new(vec2(0.0, 0.5), Shape::new_rect(vec2(8.0, 8.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(-1.0, 0.0), 5.0));
        let dst = PlacedShape::new(vec2(3.8, 4.0), Shape::new_rect(vec2(4.0, 2.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(0.0, 1.0), 0.0));
        let dst = PlacedShape::new(vec2(-2.0, -3.0), Shape::new_rect(vec2(8.0, 2.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(0.0, -1.0), -1.0));
    }

    #[test]
    fn test_circle_circle_normal() {
        let src = PlacedShape::new(vec2(1.0, 1.0), Shape::new_circle(2.0));
        let dst = PlacedShape::new(vec2(2.0, 0.0), Shape::new_circle(3.0));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(1.0, -1.0), 2.5 - (2.0f64).sqrt()));
    }

    #[test]
    fn test_rect_circle_normal() {
        let src = PlacedShape::new(vec2(0.0, 0.0), Shape::new_rect(vec2(2.0, 2.0)));

        let dst = PlacedShape::new(vec2(-2.0, 0.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(-1.0, 0.0), 0.25));
        let dst = PlacedShape::new(vec2(0.0, -2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(0.0, -1.0), 0.25));
        let dst = PlacedShape::new(vec2(2.0, 0.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(1.0, 0.0), 0.25));
        let dst = PlacedShape::new(vec2(0.0, 2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(0.0, 1.0), 0.25));

        let dst = PlacedShape::new(vec2(-2.0, -2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(-1.0, -1.0), 1.25 - (2.0f64).sqrt()));
        let dst = PlacedShape::new(vec2(2.0, -2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(1.0, -1.0), 1.25 - (2.0f64).sqrt()));
        let dst = PlacedShape::new(vec2(-2.0, 2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(-1.0, 1.0), 1.25 - (2.0f64).sqrt()));
        let dst = PlacedShape::new(vec2(2.0, 2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2(1.0, 1.0), 1.25 - (2.0f64).sqrt()));
    }
}
