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

use std::cmp::Ordering;

use geom::{Vec2, DirVec2, v2, Card};
use core::{Hitbox, HbVel};
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
    /// Dimensions must be non-negative.
    /// If `kind` is `Circle`, then the width and height must match.
    pub fn new(kind: ShapeKind, dims: Vec2) -> Shape {
        assert!(dims.x >= 0.0 && dims.y >= 0.0, "dims must be non-negative");
        Shape::with_any_dims(kind, dims)
    }

    // allows negative dims
    fn with_any_dims(kind: ShapeKind, dims: Vec2) -> Shape {
        if kind == ShapeKind::Circle {
            assert!(dims.x == dims.y, "circle width must equal height");
        }
        Shape { kind: kind, dims: dims }
    }

    /// Constructs a new circle shape, using `diam` as the width and height.
    #[inline]
    pub fn circle(diam: f64) -> Shape {
        Shape::new(ShapeKind::Circle, v2(diam, diam))
    }

    /// Constructs a new axis-aligned rectangle shape with the given `dims` (width and height dimensions).
    #[inline]
    pub fn rect(dims: Vec2) -> Shape {
        Shape::new(ShapeKind::Rect, dims)
    }

    /// Constructs a new axis-aligned square shape with the given `width`.
    #[inline]
    pub fn square(width: f64) -> Shape {
        Shape::new(ShapeKind::Rect, v2(width, width))
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

    /// Shorthand for `PlacedShape::new(pos, self)`.
    pub fn place(self, pos: Vec2) -> PlacedShape {
        PlacedShape::new(pos, self)
    }

    pub(crate) fn advance(&self, resize_vel: Vec2, elapsed: f64) -> Shape {
        Shape::with_any_dims(self.kind, self.dims + resize_vel * elapsed)
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
        PlacedShape { pos: pos, shape: shape }
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
    pub fn bottom(&self) -> f64 { self.bounds_bottom() }

    /// Returns the lowest x coordinate of the `PlacedShape`.
    pub fn left(&self) -> f64 { self.bounds_left() }

    /// Returns the highest y coordinate of the `PlacedShape`.
    pub fn top(&self) -> f64 { self.bounds_top() }

    /// Returns the highest x coordinate of the `PlacedShape`.
    pub fn right(&self) -> f64 { self.bounds_right() }

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

    /// Shorthand for `Hitbox::new(self, HbVel::moving(vel))`.
    pub fn moving(self, vel: Vec2) -> Hitbox {
        Hitbox::new(self, HbVel::moving(vel))
    }

    /// Shorthand for `Hitbox::new(self, HbVel::moving_until(vel, end_time))`.
    pub fn moving_until(self, vel: Vec2, end_time: f64) -> Hitbox {
        Hitbox::new(self, HbVel::moving_until(vel, end_time))
    }

    /// Shorthand for `Hitbox::new(self, HbVel::still())`.
    pub fn still(self) -> Hitbox {
        Hitbox::new(self, HbVel::still())
    }

    /// Shorthand for `Hitbox::new(self, HbVel::still_until(end_time))`.
    pub fn still_until(self, end_time: f64) -> Hitbox {
        Hitbox::new(self, HbVel::still_until(end_time))
    }

    pub(crate) fn sector(&self, point: Vec2) -> Sector {
        let x = interval_sector(self.left(), self.right(), point.x);
        let y = interval_sector(self.bottom(), self.top(), point.y);
        Sector::new(x, y)
    }

    pub(crate) fn as_rect(&self) -> PlacedShape {
        PlacedShape::new(self.pos, Shape::rect(self.shape.dims()))
    }

    pub(crate) fn bounding_box(&self, other: &PlacedShape) -> PlacedShape {
        let right = self.right().max(other.right());
        let top = self.top().max(other.top());
        let left = self.left().min(other.left());
        let bottom = self.bottom().min(other.bottom());

        let shape = Shape::rect(v2(right - left, top - bottom));
        let pos = v2(left + shape.dims().x * 0.5, bottom + shape.dims().y * 0.5);
        PlacedShape::new(pos, shape)
    }

    pub(crate) fn advance(&self, vel: Vec2, resize_vel: Vec2, elapsed: f64) -> PlacedShape {
        PlacedShape::new(self.pos + vel * elapsed, self.shape.advance(resize_vel, elapsed))
    }
}

pub(crate) trait PlacedBounds {
    fn bounds_center(&self) -> &Vec2;
    fn bounds_dims(&self) -> &Vec2;

    fn bounds_bottom(&self) -> f64 { self.bounds_center().y - self.bounds_dims().y * 0.5 }
    fn bounds_left(&self) -> f64 { self.bounds_center().x - self.bounds_dims().x * 0.5 }
    fn bounds_top(&self) -> f64 { self.bounds_center().y + self.bounds_dims().y * 0.5 }
    fn bounds_right(&self) -> f64 { self.bounds_center().x + self.bounds_dims().x * 0.5 }

    fn edge(&self, card: Card) -> f64 {
        match card {
            Card::Bottom => -self.bounds_bottom(),
            Card::Left => -self.bounds_left(),
            Card::Top => self.bounds_top(),
            Card::Right => self.bounds_right(),
        }
    }

    fn max_edge(&self) -> f64 {
        Card::vals().iter()
                    .map(|&card| self.edge(card).abs())
                    .max_by_key(|&edge| n64(edge))
                    .unwrap()
    }

    fn card_overlap(&self, src: &Self, card: Card) -> f64 {
        src.edge(card) + self.edge(card.flip())
    }

    fn corner(&self, sector: Sector) -> Vec2 {
        let x = match sector.x {
            Ordering::Less => self.bounds_left(),
            Ordering::Greater => self.bounds_right(),
            Ordering::Equal => panic!("expected corner sector")
        };
        let y = match sector.y {
            Ordering::Less => self.bounds_bottom(),
            Ordering::Greater => self.bounds_top(),
            Ordering::Equal => panic!("expected corner sector")
        };
        v2(x, y)
    }
}

impl PlacedBounds for PlacedShape {
    fn bounds_center(&self) -> &Vec2 { &self.pos }
    fn bounds_dims(&self) -> &Vec2 { &self.shape.dims }
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

#[cfg(test)]
mod tests {
    use geom::*;

    #[test]
    fn test_circle_advance() {
        let shape_1 = Shape::circle(2.0).place(v2(3.0, 5.0));
        assert!(shape_1.advance(v2(1.0, 2.0), v2(-0.25, -0.25), 2.0) == Shape::circle(1.5).place(v2(5.0, 9.0)));
    }

    #[test]
    fn test_rect_advance() {
        let shape_1 = Shape::rect(v2(2.0, 5.0)).place(v2(3.0, 5.0));
        assert!(shape_1.advance(v2(1.0, 2.0), v2(-0.25, 1.0), 2.0) == Shape::rect(v2(1.5, 7.0)).place(v2(5.0, 9.0)));
    }

    #[test]
    #[should_panic]
    fn test_illegal_circle_advance() {
        let shape = Shape::circle(2.0).place(v2(3.0, 5.0));
        shape.advance(v2(1.0, 2.0), v2(-0.25, -0.24), 2.0);
    }

    #[test]
    fn test_edges() {
        let shape = Shape::rect(v2(4.0, 6.0)).place(v2(3.0, 5.0));
        assert!(shape.left() == 1.0);
        assert!(shape.bottom() == 2.0);
        assert!(shape.right() == 5.0);
        assert!(shape.top() == 8.0);
    }

    #[test]
    fn test_rect_rect_normal() {
        let src = Shape::rect(v2(4.0, 4.0)).place(v2(1.0, 1.0));
        let dst = Shape::rect(v2(8.0, 8.0)).place(v2(2.0, 1.5));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(1.0, 0.0), 5.0));
        let dst = Shape::rect(v2(8.0, 8.0)).place(v2(0.0, 0.5));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(-1.0, 0.0), 5.0));
        let dst = Shape::rect(v2(4.0, 2.0)).place(v2(3.8, 4.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(0.0, 1.0), 0.0));
        let dst = Shape::rect(v2(8.0, 2.0)).place(v2(-2.0, -3.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(0.0, -1.0), -1.0));
    }

    #[test]
    fn test_circle_circle_normal() {
        let src = Shape::circle(2.0).place(v2(1.0, 1.0));
        let dst = Shape::circle(3.0).place(v2(2.0, 0.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(1.0, -1.0), 2.5 - (2.0f64).sqrt()));
    }

    #[test]
    fn test_rect_circle_normal() {
        let src = Shape::rect(v2(2.0, 2.0)).place(v2(0.0, 0.0));

        let dst = Shape::circle(2.5).place(v2(-2.0, 0.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(-1.0, 0.0), 0.25));
        let dst = Shape::circle(2.5).place(v2(0.0, -2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(0.0, -1.0), 0.25));
        let dst = Shape::circle(2.5).place(v2(2.0, 0.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(1.0, 0.0), 0.25));
        let dst = Shape::circle(2.5).place(v2(0.0, 2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(0.0, 1.0), 0.25));

        let dst = Shape::circle(2.5).place(v2(-2.0, -2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(-1.0, -1.0), 1.25 - (2.0f64).sqrt()));
        let dst = Shape::circle(2.5).place(v2(2.0, -2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(1.0, -1.0), 1.25 - (2.0f64).sqrt()));
        let dst = Shape::circle(2.5).place(v2(-2.0, 2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(-1.0, 1.0), 1.25 - (2.0f64).sqrt()));
        let dst = Shape::circle(2.5).place(v2(2.0, 2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(v2(1.0, 1.0), 1.25 - (2.0f64).sqrt()));
    }
}
