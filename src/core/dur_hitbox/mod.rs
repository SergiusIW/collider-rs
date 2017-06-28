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

mod solvers;

use std::f64;
use geom::*;
use geom_ext::*;
use core;

#[derive(Clone)]
pub struct DurHitbox {
    pub shape: PlacedShape,
    pub vel: PlacedShape,
    pub duration: f64
}

impl DurHitbox {
    pub fn new(shape: PlacedShape) -> DurHitbox {
        DurHitbox {
            shape : shape,
            vel : PlacedShape::new(Vec2::zero(), Shape::new(shape.kind(), Vec2::zero())),
            duration : f64::INFINITY,
        }
    }

    pub fn advanced_shape(&self, time: f64) -> PlacedShape {
        assert!(time < core::HIGH_TIME, "requires time < {}", core::HIGH_TIME);
        self.shape + self.vel * time
    }

    pub fn bounding_box(&self) -> PlacedShape {
        self.bounding_box_for(self.duration)
    }

    pub fn bounding_box_for(&self, duration: f64) -> PlacedShape {
        if self.vel.is_zero() {
            self.shape.as_rect()
        } else {
            let end_shape = self.advanced_shape(duration);
            self.shape.bounding_box(&end_shape)
        }
    }

    pub fn collide_time(&self, other: &DurHitbox) -> f64 {
        solvers::collide_time(self, other)
    }

    pub fn separate_time(&self, other: &DurHitbox, padding: f64) -> f64 {
        solvers::separate_time(self, other, padding)
    }
}

#[cfg(test)]
mod tests {
    use float::*;
    use geom::*;
    use core::dur_hitbox::DurHitbox;
    use std::f64;

    #[test]
    fn test_rect_rect_collision() {
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(-11.0, 0.0), Shape::new_rect(vec2_f(2.0, 2.0))));
        a.vel.pos = vec2_f(2.0, 0.0);
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(12.0, 2.0), Shape::new_rect(vec2_f(2.0, 4.0))));
        b.vel.pos = vec2_f(-0.5, 0.0);
        b.vel.shape = Shape::new_rect(vec2_f(1.0, 0.0));
        b.duration = n64(100.0);
        assert!(a.collide_time(&b) == 7.0);
        assert!(b.collide_time(&a) == 7.0);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);
    }

    #[test]
    fn test_circle_circle_collision() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(-0.1*sqrt2, 0.0), Shape::new_circle(r64(2.0))));
        a.vel.pos = vec2_f(0.1, 0.0);
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(3.0*sqrt2, 0.0), Shape::new_circle(r64(2.0 + sqrt2*0.1))));
        b.vel.pos = vec2_f(-2.0, 1.0);
        b.vel.shape = Shape::new_circle(r64(-0.1));
        b.duration = n64(100.0);
        assert!((a.collide_time(&b) - sqrt2).abs() < 1e-7);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);
    }

    #[test]
    fn test_rect_circle_collision() {
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(-11.0, 0.0), Shape::new_circle(r64(2.0))));
        a.vel.pos = vec2_f(2.0, 0.0);
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(12.0, 2.0), Shape::new_rect(vec2_f(2.0, 4.0))));
        b.vel.pos = vec2_f(-1.0, 0.0);
        b.duration = n64(100.0);
        assert!(a.collide_time(&b) == 7.0);
        assert!(b.collide_time(&a) == 7.0);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);
    }

    #[test]
    fn test_rect_rect_separation() {
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(0.0, 0.0), Shape::new_rect(vec2_f(6.0, 4.0))));
        a.vel.pos = vec2_f(1.0, 1.0);
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(1.0, 0.0), Shape::new_rect(vec2_f(4.0, 4.0))));
        b.vel.pos = vec2_f(0.5, 0.0);
        b.duration = n64(100.0);
        assert!(a.separate_time(&b, r64(0.1)) == 4.1);
        assert!(b.separate_time(&a, r64(0.1)) == 4.1);
        assert!(a.collide_time(&b) == 0.0);
    }

    #[test]
    fn test_circle_circle_separation() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(2.0, 5.0), Shape::new_circle(r64(2.0))));
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(3.0, 4.0), Shape::new_circle(r64(1.8))));
        b.vel.pos = vec2_f(-1.0, 1.0);
        b.duration = n64(100.0);
        assert!(a.separate_time(&b, r64(0.1)) == 1.0 + sqrt2);
        assert!(b.separate_time(&a, r64(0.1)) == 1.0 + sqrt2);
        assert!(a.collide_time(&b) == 0.0);
    }

    #[test]
    fn test_rect_circle_separation() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(4.0, 2.0), Shape::new_rect(vec2_f(4.0, 6.0))));
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(3.0, 4.0), Shape::new_circle(r64(3.8))));
        b.vel.pos = vec2_f(-1.0, 1.0);
        b.duration = n64(100.0);
        assert!(a.separate_time(&b, r64(0.1)) == 1.0 + sqrt2);
        assert!(b.separate_time(&a, r64(0.1)) == 1.0 + sqrt2);
        assert!(a.collide_time(&b) == 0.0);
    }

    #[test]
    fn test_no_collision() {
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(-11.0, 0.0), Shape::new_rect(vec2_f(2.0, 2.0))));
        a.vel.pos = vec2_f(2.0, 0.0);
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(12.0, 2.0), Shape::new_rect(vec2_f(2.0, 4.0))));
        b.vel.pos = vec2_f(-1.0, 1.0);
        b.duration = n64(100.0);
        assert!(a.collide_time(&b) == f64::INFINITY);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);

        b.shape.shape == Shape::new_circle(r64(2.0));
        b.vel.shape == Shape::new_circle(r64(0.0));
        assert!(a.collide_time(&b) == f64::INFINITY);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);

        a.shape.shape == Shape::new_circle(r64(2.0));
        a.vel.shape == Shape::new_circle(r64(0.0));
        assert!(a.collide_time(&b) == f64::INFINITY);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);
    }

    #[test]
    fn test_no_separation() {
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(5.0, 1.0), Shape::new_rect(vec2_f(2.0, 2.0))));
        a.vel.pos = vec2_f(2.0, 1.0);
        a.duration = n64(100.0);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(5.0, 1.0), Shape::new_rect(vec2_f(2.0, 4.0))));
        b.vel.pos = vec2_f(2.0, 1.0);
        b.duration = n64(100.0);
        assert!(a.separate_time(&b, r64(0.1)) == f64::INFINITY);
        assert!(a.collide_time(&b) == 0.0);

        b.shape.shape == Shape::new_circle(r64(2.0));
        b.vel.shape == Shape::new_circle(r64(0.0));
        assert!(a.separate_time(&b, r64(0.1)) == f64::INFINITY);
        assert!(a.collide_time(&b) == 0.0);

        a.shape.shape == Shape::new_circle(r64(2.0));
        a.vel.shape == Shape::new_circle(r64(0.0));
        assert!(a.separate_time(&b, r64(0.1)) == f64::INFINITY);
        assert!(a.collide_time(&b) == 0.0);
    }

    #[test]
    fn test_low_duration() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = DurHitbox::new(PlacedShape::new(vec2_f(0.0, 0.0), Shape::new_circle(r64(2.0))));
        a.duration = n64(4.0 - sqrt2 + 0.01);
        let mut b = DurHitbox::new(PlacedShape::new(vec2_f(4.0, 4.0), Shape::new_circle(r64(2.0))));
        b.vel.pos = vec2_f(-1.0, -1.0);
        b.duration = n64(4.0 - sqrt2 + 0.01);
        assert!(a.collide_time(&b) == 4.0 - sqrt2);
        a.duration -= 0.02;
        assert!(a.collide_time(&b) == f64::INFINITY);
        b.duration -= 0.02;
        assert!(a.collide_time(&b) == f64::INFINITY);
    }
}
