// Copyright 2016 Matthew D. Michelotti
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
use noisy_float::prelude::*;
use geom::{Vec2, DirVec2, vec2};

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
    pub fn new_circle(diam: R64) -> Shape {
        Shape::new(ShapeKind::Circle, vec2(diam, diam))
    }
    
    /// Constructs a new axis-aligned rectangle shape with the given `dims` (width and height dimensions).
    #[inline]
    pub fn new_rect(dims: Vec2) -> Shape {
        Shape::new(ShapeKind::Rect, dims)
    }
    
    /// Constructs a new axis-aligned square shape with the given `width`.
    #[inline]
    pub fn new_square(width: R64) -> Shape {
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

impl Mul<R64> for Shape {
    type Output = Shape;
    fn mul(self, rhs: R64) -> Shape {
        Shape::new(self.kind, self.dims * rhs)
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
    pub fn bottom(&self) -> R64 {
        self.pos.y - self.shape.dims().y * 0.5
    }
    
    /// Returns the lowest x coordinate of the `PlacedShape`.
    pub fn left(&self) -> R64 {
        self.pos.x - self.shape.dims().x * 0.5
    }
    
    /// Returns the highest y coordinate of the `PlacedShape`.
    pub fn top(&self) -> R64 {
        self.pos.y + self.shape.dims().y * 0.5
    }
    
    /// Returns the highest x coordinate of the `PlacedShape`.
    pub fn right(&self) -> R64 {
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
}

impl Mul<R64> for PlacedShape {
    type Output = PlacedShape;
    fn mul(self, rhs: R64) -> PlacedShape {
        PlacedShape::new(self.pos * rhs, self.shape * rhs)
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
    use noisy_float::prelude::*;
    use geom::*;

    #[test]
    fn test_circle_ops() {
        let shape_1 = PlacedShape::new(vec2_f(3.0, 5.0), Shape::new_circle(r64(2.0)));
        let shape_2 = PlacedShape::new(vec2_f(1.0, 2.0), Shape::new_circle(r64(-1.0)));
        let shape_3 = PlacedShape::new(vec2_f(-1.0, 3.0), Shape::new_circle(r64(4.0)));
        let expected = PlacedShape::new(vec2_f(9.0, 10.0), Shape::new_circle(r64(3.0)));
        assert!(shape_1 * 3.0 + -shape_2 - shape_3 == expected);
    }

    #[test]
    fn test_rect_ops() {
        let shape_1 = PlacedShape::new(vec2_f(3.0, 5.0), Shape::new_rect(vec2_f(2.0, 5.0)));
        let shape_2 = PlacedShape::new(vec2_f(1.0, 2.0), Shape::new_rect(vec2_f(-1.0, 2.0)));
        let shape_3 = PlacedShape::new(vec2_f(-1.0, 3.0), Shape::new_rect(vec2_f(4.0, 1.0)));
        let expected = PlacedShape::new(vec2_f(9.0, 10.0), Shape::new_rect(vec2_f(3.0, 12.0)));
        assert!(shape_1*3.0 + -shape_2 - shape_3 == expected);
    }

    #[test]
    #[should_panic]
    fn test_circle_rect_add() {
        Shape::new_rect(vec2_f(1.0, 2.0)) + Shape::new_circle(r64(3.0));
    }

    #[test]
    #[should_panic]
    fn test_circle_rect_sub() {
        Shape::new_rect(vec2_f(1.0, 2.0)) - Shape::new_circle(r64(3.0));
    }
    
    #[test]
    fn test_edges() {
        let shape = PlacedShape::new(vec2_f(3.0, 5.0), Shape::new_rect(vec2_f(4.0, 6.0)));
        assert!(shape.left() == 1.0);
        assert!(shape.bottom() == 2.0);
        assert!(shape.right() == 5.0);
        assert!(shape.top() == 8.0);
    }
    
    #[test]
    fn test_rect_rect_normal() {
        let src = PlacedShape::new(vec2_f(1.0, 1.0), Shape::new_rect(vec2_f(4.0, 4.0)));
        let dst = PlacedShape::new(vec2_f(2.0, 1.5), Shape::new_rect(vec2_f(8.0, 8.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(1.0, 0.0), r64(5.0)));
        let dst = PlacedShape::new(vec2_f(0.0, 0.5), Shape::new_rect(vec2_f(8.0, 8.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(-1.0, 0.0), r64(5.0)));
        let dst = PlacedShape::new(vec2_f(3.8, 4.0), Shape::new_rect(vec2_f(4.0, 2.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(0.0, 1.0), r64(0.0)));
        let dst = PlacedShape::new(vec2_f(-2.0, -3.0), Shape::new_rect(vec2_f(8.0, 2.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(0.0, -1.0), r64(-1.0)));
    }
    
    #[test]
    fn test_circle_circle_normal() {
        let src = PlacedShape::new(vec2_f(1.0, 1.0), Shape::new_circle(r64(2.0)));
        let dst = PlacedShape::new(vec2_f(2.0, 0.0), Shape::new_circle(r64(3.0)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(1.0, -1.0), r64(2.5 - (2.0f64).sqrt())));
    }
    
    #[test]
    fn test_rect_circle_normal() {
        let src = PlacedShape::new(vec2_f(0.0, 0.0), Shape::new_rect(vec2_f(2.0, 2.0)));
        
        let dst = PlacedShape::new(vec2_f(-2.0, 0.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(-1.0, 0.0), r64(0.25)));
        let dst = PlacedShape::new(vec2_f(0.0, -2.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(0.0, -1.0), r64(0.25)));
        let dst = PlacedShape::new(vec2_f(2.0, 0.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(1.0, 0.0), r64(0.25)));
        let dst = PlacedShape::new(vec2_f(0.0, 2.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(0.0, 1.0), r64(0.25)));
        
        let dst = PlacedShape::new(vec2_f(-2.0, -2.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(-1.0, -1.0), r64(1.25 - (2.0f64).sqrt())));
        let dst = PlacedShape::new(vec2_f(2.0, -2.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(1.0, -1.0), r64(1.25 - (2.0f64).sqrt())));
        let dst = PlacedShape::new(vec2_f(-2.0, 2.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(-1.0, 1.0), r64(1.25 - (2.0f64).sqrt())));
        let dst = PlacedShape::new(vec2_f(2.0, 2.0), Shape::new_circle(r64(2.5)));
        assert!(dst.normal_from(&src) == DirVec2::new(vec2_f(1.0, 1.0), r64(1.25 - (2.0f64).sqrt())));
    }
}
