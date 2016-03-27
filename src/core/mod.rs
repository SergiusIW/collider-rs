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

mod solvers;
mod grid;
mod collider;

use geom::*;
use geom_ext::*;
use std::f64;

const HIGH_TIME: f64 = 1e50;

//TODO check Hitbox consistency when submitting to Collider for a change (e.g. make sure shape width/height is at least padding, duration is non-negative and non-NaN, ...)

pub type HitboxId = u64;

#[derive(PartialEq, Clone, Debug)]
pub struct Hitbox {
    pub shape: PlacedShape,
    pub vel: PlacedShape,
    pub duration: f64
}

impl Hitbox {
    pub fn new(shape: PlacedShape) -> Hitbox {
        Hitbox {
            shape : shape,
            vel : PlacedShape::new(Vec2::zero(), Shape::new(shape.kind(), 0.0, 0.0)),
            duration : f64::INFINITY
        }
    }
    
    fn advance(&mut self, orig_time: f64, new_time: f64) {
        assert!(orig_time <= new_time, "requires orig_time <= new_time");
        let delta = new_time - orig_time;
        if delta != 0.0 {
            self.shape = self.advanced_shape(delta);
            let end_time = orig_time + self.duration;
            assert!(new_time <= end_time, "tried to advance Hitbox beyond its duration");
            self.duration = end_time - new_time;
        }
    }
    
    fn advanced_shape(&self, time: f64) -> PlacedShape {
        assert!(time <= HIGH_TIME, "requires time <= {}", HIGH_TIME);
        self.shape + self.vel*time
    }
    
    fn bounding_box(&self) -> PlacedShape {
        self.bounding_box_for(self.duration)
    }
    
    fn bounding_box_for(&self, duration: f64) -> PlacedShape {
        if self.vel.is_zero() {
            self.shape.as_rect()
        } else {
            let end_shape = self.advanced_shape(duration);
            self.shape.bounding_box(&end_shape)
        }
    }
    
    fn collide_time(&self, other: &Hitbox) -> f64 {
        solvers::collide_time(self, other)
    }
    
    fn separate_time(&self, other: &Hitbox, padding: f64) -> f64 {
        solvers::separate_time(self, other, padding)
    }
    
    fn validate(&self, min_size: f64) {
        assert!(!self.duration.is_nan() && self.duration >= 0.0, "duration must be non-negative");
        assert!(self.shape.kind() == vel.kind(), "shape and vel have different kinds");
        assert!(self.shape.width() >= min_size && self.shape.height() >= min_size, "shape width/height must be at least {}", min_size);
    }
    
    fn time_until_too_small(&self, min_size: f64) -> f64 {
        let min_size = min_size*0.9;
        assert!(self.shape.width() > min_size && self.shape.height() > min_size, "illegal state");
        let mut time = f64::INFINITY;
        if self.vel.width() < 0.0 { time = time.min((min_size - self.shape.width())/self.vel.width()); }
        if self.vel.height() < 0.0 { time = time.min((min_size - self.shape.height())/self.vel.height()); }
        time
    }
}

pub mod inter {
    pub type Group = u32;

    static DEFAULT_GROUPS: [Group; 1] = [0];

    pub trait Interactivity {
        fn group(&self) -> Option<Group> { Some(0) }
        fn interact_groups(&self) -> &'static [Group] { &DEFAULT_GROUPS }
        fn can_interact(&self, other: &Self) -> bool;
    }

    pub struct DefaultInteractivity;

    impl Interactivity for DefaultInteractivity {
        fn can_interact(&self, other: &Self) -> bool { true }
    }
}

#[cfg(test)]
mod tests {
    use geom::*;
    use core::*;
    use std::f64;

    #[test]
    fn test_rect_rect_collision() {
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(-11.0, 0.0), Shape::new_rect(2.0, 2.0)));
        a.vel.pos = Vec2::new(2.0, 0.0);
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(12.0, 2.0), Shape::new_rect(2.0, 4.0)));
        b.vel.pos = Vec2::new(-0.5, 0.0);
        b.vel.shape = Shape::new_rect(1.0, 0.0);
        b.duration = 100.0;
        assert!(a.collide_time(&b) == 7.0);
        assert!(b.collide_time(&a) == 7.0);
        assert!(a.separate_time(&b, 0.1) == 0.0);
    }
    
    #[test]
    fn test_circle_circle_collision() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(-0.1*sqrt2, 0.0), Shape::new_circle(2.0)));
        a.vel.pos = Vec2::new(0.1, 0.0);
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(3.0*sqrt2, 0.0), Shape::new_circle(2.0 + sqrt2*0.1)));
        b.vel.pos = Vec2::new(-2.0, 1.0);
        b.vel.shape = Shape::new_circle(-0.1);
        b.duration = 100.0;
        assert!((a.collide_time(&b) - sqrt2).abs() < 1e-7);
        assert!(a.separate_time(&b, 0.1) == 0.0);
    }

    #[test]
    fn test_rect_circle_collision() {
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(-11.0, 0.0), Shape::new_circle(2.0)));
        a.vel.pos = Vec2::new(2.0, 0.0);
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(12.0, 2.0), Shape::new_rect(2.0, 4.0)));
        b.vel.pos = Vec2::new(-1.0, 0.0);
        b.duration = 100.0;
        assert!(a.collide_time(&b) == 7.0);
        assert!(b.collide_time(&a) == 7.0);
        assert!(a.separate_time(&b, 0.1) == 0.0);
    }

    #[test]
    fn test_rect_rect_separation() {
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(0.0, 0.0), Shape::new_rect(6.0, 4.0)));
        a.vel.pos = Vec2::new(1.0, 1.0);
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(1.0, 0.0), Shape::new_rect(4.0, 4.0)));
        b.vel.pos = Vec2::new(0.5, 0.0);
        b.duration = 100.0;
        assert!(a.separate_time(&b, 0.1) == 4.1);
        assert!(b.separate_time(&a, 0.1) == 4.1);
        assert!(a.collide_time(&b) == 0.0);
    }
    
    #[test]
    fn test_circle_circle_separation() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(2.0, 5.0), Shape::new_circle(2.0)));
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(3.0, 4.0), Shape::new_circle(1.8)));
        b.vel.pos = Vec2::new(-1.0, 1.0);
        b.duration = 100.0;
        assert!(a.separate_time(&b, 0.1) == 1.0 + sqrt2);
        assert!(b.separate_time(&a, 0.1) == 1.0 + sqrt2);
        assert!(a.collide_time(&b) == 0.0);
    }
    
    #[test]
    fn test_rect_circle_separation() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(4.0, 2.0), Shape::new_rect(4.0, 6.0)));
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(3.0, 4.0), Shape::new_circle(3.8)));
        b.vel.pos = Vec2::new(-1.0, 1.0);
        b.duration = 100.0;
        assert!(a.separate_time(&b, 0.1) == 1.0 + sqrt2);
        assert!(b.separate_time(&a, 0.1) == 1.0 + sqrt2);
        assert!(a.collide_time(&b) == 0.0);
    }
    
    #[test]
    fn test_no_collision() {
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(-11.0, 0.0), Shape::new_rect(2.0, 2.0)));
        a.vel.pos = Vec2::new(2.0, 0.0);
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(12.0, 2.0), Shape::new_rect(2.0, 4.0)));
        b.vel.pos = Vec2::new(-1.0, 1.0);
        b.duration = 100.0;
        assert!(a.collide_time(&b) == f64::INFINITY);
        assert!(a.separate_time(&b, 0.1) == 0.0);
        
        b.shape.shape == Shape::new_circle(2.0);
        b.vel.shape == Shape::new_circle(0.0);
        assert!(a.collide_time(&b) == f64::INFINITY);
        assert!(a.separate_time(&b, 0.1) == 0.0);
        
        a.shape.shape == Shape::new_circle(2.0);
        a.vel.shape == Shape::new_circle(0.0);
        assert!(a.collide_time(&b) == f64::INFINITY);
        assert!(a.separate_time(&b, 0.1) == 0.0);
    }
    
    #[test]
    fn test_no_separation() {
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(5.0, 1.0), Shape::new_rect(2.0, 2.0)));
        a.vel.pos = Vec2::new(2.0, 1.0);
        a.duration = 100.0;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(5.0, 1.0), Shape::new_rect(2.0, 4.0)));
        b.vel.pos = Vec2::new(2.0, 1.0);
        b.duration = 100.0;
        assert!(a.separate_time(&b, 0.1) == f64::INFINITY);
        assert!(a.collide_time(&b) == 0.0);
        
        b.shape.shape == Shape::new_circle(2.0);
        b.vel.shape == Shape::new_circle(0.0);
        assert!(a.separate_time(&b, 0.1) == f64::INFINITY);
        assert!(a.collide_time(&b) == 0.0);
        
        a.shape.shape == Shape::new_circle(2.0);
        a.vel.shape == Shape::new_circle(0.0);
        assert!(a.separate_time(&b, 0.1) == f64::INFINITY);
        assert!(a.collide_time(&b) == 0.0);
    }
    
    #[test]
    fn test_low_duration() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = Hitbox::new(PlacedShape::new(Vec2::new(0.0, 0.0), Shape::new_circle(2.0)));
        a.duration = 4.0 - sqrt2 + 0.01;
        let mut b = Hitbox::new(PlacedShape::new(Vec2::new(4.0, 4.0), Shape::new_circle(2.0)));
        b.vel.pos = Vec2::new(-1.0, -1.0);
        b.duration = 4.0 - sqrt2 + 0.01;
        assert!(a.collide_time(&b) == 4.0 - sqrt2);
        a.duration -= 0.02;
        assert!(a.collide_time(&b) == f64::INFINITY);
        b.duration -= 0.02;
        assert!(a.collide_time(&b) == f64::INFINITY);
    }
}
