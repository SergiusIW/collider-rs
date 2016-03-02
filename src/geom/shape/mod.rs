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
use geom::{Vec2, DirVec2};

mod normals;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ShapeKind {
    Circle,
    Rect
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Shape {
    kind: ShapeKind,
    width: f64,
    height: f64
}

impl Shape {
    pub fn new(kind: ShapeKind, width: f64, height: f64) -> Shape {
        assert!(kind == ShapeKind::Rect || width == height, "circle width must equal height");
        Shape { kind : kind, width : width, height : height }
    }

    pub fn new_circle(diam: f64) -> Shape {
        Shape::new(ShapeKind::Circle, diam, diam)
    }
    
    pub fn new_rect(width: f64, height: f64) -> Shape {
        Shape::new(ShapeKind::Rect, width, height)
    }
    
    #[inline]
    pub fn kind(&self) -> ShapeKind {
        self.kind
    }
    
    #[inline]
    pub fn width(&self) -> f64 {
        self.width
    }
    
    #[inline]
    pub fn height(&self) -> f64 {
        self.height
    }
}

impl Mul<f64> for Shape {
    type Output = Shape;
    fn mul(self, rhs: f64) -> Shape {
        Shape::new(self.kind, self.width*rhs, self.height*rhs)
    }
}

impl Add for Shape {
    type Output = Shape;
    fn add(self, rhs: Shape) -> Shape {
        assert!(self.kind == rhs.kind, "cannot add circle with rect");
        Shape::new(self.kind, self.width + rhs.width, self.height + rhs.height)
    }
}

impl Sub for Shape {
    type Output = Shape;
    fn sub(self, rhs: Shape) -> Shape {
        assert!(self.kind == rhs.kind, "cannot add circle with rect");
        Shape::new(self.kind, self.width - rhs.width, self.height - rhs.height)
    }
}

impl Neg for Shape {
    type Output = Shape;
    fn neg(self) -> Shape {
        Shape::new(self.kind, -self.width, -self.height)
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct PlacedShape {
    pub pos: Vec2,
    pub shape: Shape
}

impl PlacedShape {
    #[inline]
    pub fn new(pos: Vec2, shape: Shape) -> PlacedShape {
        PlacedShape { pos : pos, shape : shape }
    }
    
    #[inline]
    pub fn kind(&self) -> ShapeKind {
        self.shape.kind()
    }
    
    #[inline]
    pub fn width(&self) -> f64 {
        self.shape.width()
    }
    
    #[inline]
    pub fn height(&self) -> f64 {
        self.shape.height()
    }
    
    pub fn bottom(&self) -> f64 {
        self.pos.y() - 0.5*self.shape.height()
    }
    
    pub fn left(&self) -> f64 {
        self.pos.x() - 0.5*self.shape.width()
    }
    
    pub fn top(&self) -> f64 {
        self.pos.y() + 0.5*self.shape.height()
    }
    
    pub fn right(&self) -> f64 {
        self.pos.x() + 0.5*self.shape.width()
    }
    
    pub fn overlaps(&self, other: &PlacedShape) -> bool {
        self.normal_from(other).len() >= 0.0
    }
    
    pub fn normal_from(&self, other: &PlacedShape) -> DirVec2 {
        match (self.kind(), other.kind()) {
            (ShapeKind::Rect, ShapeKind::Rect) => normals::rect_rect_normal(self, other),
            (ShapeKind::Rect, ShapeKind::Circle) => normals::rect_circle_normal(self, other),
            (ShapeKind::Circle, ShapeKind::Rect) => normals::rect_circle_normal(other, self).flip(),
            (ShapeKind::Circle, ShapeKind::Circle) => normals::circle_circle_normal(self, other)
        }
    }
}

impl Mul<f64> for PlacedShape {
    type Output = PlacedShape;
    fn mul(self, rhs: f64) -> PlacedShape {
        PlacedShape::new(self.pos*rhs, self.shape*rhs)
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
        let shape_1 = PlacedShape::new(Vec2::new(3.0, 5.0), Shape::new_circle(2.0));
        let shape_2 = PlacedShape::new(Vec2::new(1.0, 2.0), Shape::new_circle(-1.0));
        let shape_3 = PlacedShape::new(Vec2::new(-1.0, 3.0), Shape::new_circle(4.0));
        let result = PlacedShape::new(Vec2::new(9.0, 10.0), Shape::new_circle(3.0));
        assert!(shape_1*3.0 + -shape_2 - shape_3 == result);
    }

    #[test]
    fn test_rect_ops() {
        let shape_1 = PlacedShape::new(Vec2::new(3.0, 5.0), Shape::new_rect(2.0, 5.0));
        let shape_2 = PlacedShape::new(Vec2::new(1.0, 2.0), Shape::new_rect(-1.0, 2.0));
        let shape_3 = PlacedShape::new(Vec2::new(-1.0, 3.0), Shape::new_rect(4.0, 1.0));
        let result = PlacedShape::new(Vec2::new(9.0, 10.0), Shape::new_rect(3.0, 12.0));
        assert!(shape_1*3.0 + -shape_2 - shape_3 == result);
    }

    #[test]
    #[should_panic]
    fn test_circle_rect_add() {
        Shape::new_rect(1.0, 2.0) + Shape::new_circle(3.0);
    }

    #[test]
    #[should_panic]
    fn test_circle_rect_sub() {
        Shape::new_rect(1.0, 2.0) - Shape::new_circle(3.0);
    }
    
    #[test]
    fn test_edges() {
        let shape = PlacedShape::new(Vec2::new(3.0, 5.0), Shape::new_rect(4.0, 6.0));
        assert!(shape.left() == 1.0);
        assert!(shape.bottom() == 2.0);
        assert!(shape.right() == 5.0);
        assert!(shape.top() == 8.0);
    }
    
    #[test]
    fn test_rect_rect_normal() {
        let src = PlacedShape::new(Vec2::new(1.0, 1.0), Shape::new_rect(4.0, 4.0));
        let dst = PlacedShape::new(Vec2::new(2.0, 1.5), Shape::new_rect(8.0, 8.0));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(1.0, 0.0), 5.0));
        let dst = PlacedShape::new(Vec2::new(0.0, 0.5), Shape::new_rect(8.0, 8.0));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(-1.0, 0.0), 5.0));
        let dst = PlacedShape::new(Vec2::new(3.8, 4.0), Shape::new_rect(4.0, 2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(0.0, 1.0), 0.0));
        let dst = PlacedShape::new(Vec2::new(-2.0, -3.0), Shape::new_rect(8.0, 2.0));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(0.0, -1.0), -1.0));
    }
    
    #[test]
    fn test_circle_circle_normal() {
        let src = PlacedShape::new(Vec2::new(1.0, 1.0), Shape::new_circle(2.0));
        let dst = PlacedShape::new(Vec2::new(2.0, 0.0), Shape::new_circle(3.0));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(1.0, -1.0), 2.5 - (2.0f64).sqrt()));
    }
    
    #[test]
    fn test_rect_circle_normal() {
        let src = PlacedShape::new(Vec2::new(0.0, 0.0), Shape::new_rect(2.0, 2.0));
        
        let dst = PlacedShape::new(Vec2::new(-2.0, 0.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(-1.0, 0.0), 0.25));
        let dst = PlacedShape::new(Vec2::new(0.0, -2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(0.0, -1.0), 0.25));
        let dst = PlacedShape::new(Vec2::new(2.0, 0.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(1.0, 0.0), 0.25));
        let dst = PlacedShape::new(Vec2::new(0.0, 2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(0.0, 1.0), 0.25));
        
        let dst = PlacedShape::new(Vec2::new(-2.0, -2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(-1.0, -1.0), 1.25 - (2.0f64).sqrt()));
        let dst = PlacedShape::new(Vec2::new(2.0, -2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(1.0, -1.0), 1.25 - (2.0f64).sqrt()));
        let dst = PlacedShape::new(Vec2::new(-2.0, 2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(-1.0, 1.0), 1.25 - (2.0f64).sqrt()));
        let dst = PlacedShape::new(Vec2::new(2.0, 2.0), Shape::new_circle(2.5));
        assert!(dst.normal_from(&src) == DirVec2::new(Vec2::new(1.0, 1.0), 1.25 - (2.0f64).sqrt()));
    }
}
