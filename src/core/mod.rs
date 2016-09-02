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
mod events;

pub use self::collider::*;

use geom::*;
use geom_ext::*;
use noisy_float::prelude::*;

const HIGH_TIME: f64 = 1e50;

/// Type used as a handle for referencing hitboxes in a `Collider` instance.
pub type HitboxId = u64;

/// Represents a moving shape for continuous collision testing.
#[derive(PartialEq, Clone, Debug)]
pub struct Hitbox {
    /// The placed shape `shape` at the given point in time.
    pub shape: PlacedShape,
    
    /// A velocity that describes how the shape is changing over time.
    ///
    /// The `vel` may include the velocity of the width and height of the `shape` as
    /// well as the velocity of the position.
    pub vel: PlacedShape,
    
    /// An upper-bound on the time until this hitbox will be updated by the user.
    ///
    /// `N64::infinity()` may be used as a default, but using a lower value may
    /// reduce the number of collisions that need to be checked.
    /// E.g. if you are updating the velocities of a hitbox once every second,
    /// then use a duration of one second when you update the hitbox.
    ///
    /// `Collider` will panic if the duration is exceeded without update.
    pub duration: N64
}

impl Hitbox {
    /// Constructs a new hitbox with the given `shape` and a `vel` of zero and `duration` of `f64::INFINITY`.
    pub fn new(shape: PlacedShape) -> Hitbox {
        Hitbox {
            shape : shape,
            vel : PlacedShape::new(Vec2::zero(), Shape::new(shape.kind(), Vec2::zero())),
            duration : N64::infinity()
        }
    }
    
    fn advance(&mut self, orig_time: N64, new_time: N64) {
        assert!(orig_time <= new_time, "requires orig_time <= new_time");
        let delta = new_time - orig_time;
        if delta != 0.0 {
            self.shape = self.advanced_shape(delta);
            let end_time = orig_time + self.duration;
            assert!(new_time <= end_time, "tried to advance Hitbox beyond its duration");
            self.duration = end_time - new_time;
        }
    }
    
    fn advanced_shape(&self, time: N64) -> PlacedShape {
        assert!(time <= HIGH_TIME, "requires time <= {}", HIGH_TIME);
        self.shape + self.vel * r64(time.raw())
    }
    
    fn bounding_box(&self) -> PlacedShape {
        self.bounding_box_for(self.duration)
    }
    
    fn bounding_box_for(&self, duration: N64) -> PlacedShape {
        if self.vel.is_zero() {
            self.shape.as_rect()
        } else {
            let end_shape = self.advanced_shape(duration);
            self.shape.bounding_box(&end_shape)
        }
    }
    
    fn collide_time(&self, other: &Hitbox) -> N64 {
        solvers::collide_time(self, other)
    }
    
    fn separate_time(&self, other: &Hitbox, padding: R64) -> N64 {
        solvers::separate_time(self, other, padding)
    }
    
    fn validate(&self, min_size: R64) {
        assert!(!self.duration.is_nan() && self.duration >= 0.0, "duration must be non-negative");
        assert!(self.shape.kind() == self.vel.kind(), "shape and vel have different kinds");
        assert!(self.shape.dims().x >= min_size && self.shape.dims().y >= min_size, "shape width/height must be at least {}", min_size);
    }
    
    fn time_until_too_small(&self, min_size: R64) -> N64 {
        let min_size = min_size * 0.9;
        assert!(self.shape.dims().x > min_size && self.shape.dims().y > min_size, "illegal state");
        let mut time = N64::infinity();
        if self.vel.dims().x < 0.0 { time = time.min(N64::from(min_size - self.shape.dims().x) / N64::from(self.vel.dims().x)); }
        if self.vel.dims().y < 0.0 { time = time.min(N64::from(min_size - self.shape.dims().y) / N64::from(self.vel.dims().y)); }
        time
    }
}

/// Contains types that describe interactions between hitboxes.
pub mod inter {
    /// A group id that may be used as a first measure to efficiently filter out hitboxes that don't interact.
    ///
    /// The total number of groups used should in general be very small.
    /// Often 1 is enough, and 4 is excessive.
    /// As an example, in a [danmaku](https://en.wikipedia.org/wiki/Shoot_%27em_up#Bullet_hell_and_niche_appeal) game
    /// (which has many bullets on screen that do not interact with each other),
    /// we may use one group for bullets and one group for everything else,
    /// to avoid the quadratic cost of comparing all nearby bullets with each other.
    pub type Group = u32;

    static DEFAULT_GROUPS: [Group; 1] = [0];

    /// Used to determine which pairs of hitboxes should be checked for collisions
    /// and which pairs should be ignored.
    pub trait Interactivity {
        /// Returns the group id associated with the hitbox.
        /// Default is `Some(0)`.
        ///
        /// If `None` is returned, then no collisions will be reported
        /// for this hitbox at all.
        fn group(&self) -> Option<Group> { Some(0) }
        
        /// Returns a list of groups that this hitbox can interact with.
        /// Using large lists of groups may be inefficient.
        /// Default is `[0]`.
        fn interact_groups(&self) -> &'static [Group] { &DEFAULT_GROUPS }
        
        /// Returns true if the pair of hitboxes should be checked for collisions.
        /// This method should be commutative.
        /// This method should be consistent with `group` and `interact_groups`,
        /// although possibly more restrictive.
        fn can_interact(&self, other: &Self) -> bool;
    }

    /// The default implementation of `Interactivity`, in which
    /// every hitbox is allowed to interact with every other hitbox.
    #[derive(Default)]
    pub struct DefaultInteractivity;

    impl Interactivity for DefaultInteractivity {
        fn can_interact(&self, _other: &Self) -> bool { true }
    }
}

#[cfg(test)]
mod tests {
    use noisy_float::prelude::*;
    use geom::*;
    use core::*;
    use std::f64;

    #[test]
    fn test_rect_rect_collision() {
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(-11.0, 0.0), Shape::new_rect(vec2_f(2.0, 2.0))));
        a.vel.pos = vec2_f(2.0, 0.0);
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(12.0, 2.0), Shape::new_rect(vec2_f(2.0, 4.0))));
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
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(-0.1*sqrt2, 0.0), Shape::new_circle(r64(2.0))));
        a.vel.pos = vec2_f(0.1, 0.0);
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(3.0*sqrt2, 0.0), Shape::new_circle(r64(2.0 + sqrt2*0.1))));
        b.vel.pos = vec2_f(-2.0, 1.0);
        b.vel.shape = Shape::new_circle(r64(-0.1));
        b.duration = n64(100.0);
        assert!((a.collide_time(&b) - sqrt2).abs() < 1e-7);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);
    }

    #[test]
    fn test_rect_circle_collision() {
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(-11.0, 0.0), Shape::new_circle(r64(2.0))));
        a.vel.pos = vec2_f(2.0, 0.0);
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(12.0, 2.0), Shape::new_rect(vec2_f(2.0, 4.0))));
        b.vel.pos = vec2_f(-1.0, 0.0);
        b.duration = n64(100.0);
        assert!(a.collide_time(&b) == 7.0);
        assert!(b.collide_time(&a) == 7.0);
        assert!(a.separate_time(&b, r64(0.1)) == 0.0);
    }

    #[test]
    fn test_rect_rect_separation() {
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(0.0, 0.0), Shape::new_rect(vec2_f(6.0, 4.0))));
        a.vel.pos = vec2_f(1.0, 1.0);
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(1.0, 0.0), Shape::new_rect(vec2_f(4.0, 4.0))));
        b.vel.pos = vec2_f(0.5, 0.0);
        b.duration = n64(100.0);
        assert!(a.separate_time(&b, r64(0.1)) == 4.1);
        assert!(b.separate_time(&a, r64(0.1)) == 4.1);
        assert!(a.collide_time(&b) == 0.0);
    }
    
    #[test]
    fn test_circle_circle_separation() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(2.0, 5.0), Shape::new_circle(r64(2.0))));
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(3.0, 4.0), Shape::new_circle(r64(1.8))));
        b.vel.pos = vec2_f(-1.0, 1.0);
        b.duration = n64(100.0);
        assert!(a.separate_time(&b, r64(0.1)) == 1.0 + sqrt2);
        assert!(b.separate_time(&a, r64(0.1)) == 1.0 + sqrt2);
        assert!(a.collide_time(&b) == 0.0);
    }
    
    #[test]
    fn test_rect_circle_separation() {
        let sqrt2 = (2.0f64).sqrt();
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(4.0, 2.0), Shape::new_rect(vec2_f(4.0, 6.0))));
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(3.0, 4.0), Shape::new_circle(r64(3.8))));
        b.vel.pos = vec2_f(-1.0, 1.0);
        b.duration = n64(100.0);
        assert!(a.separate_time(&b, r64(0.1)) == 1.0 + sqrt2);
        assert!(b.separate_time(&a, r64(0.1)) == 1.0 + sqrt2);
        assert!(a.collide_time(&b) == 0.0);
    }
    
    #[test]
    fn test_no_collision() {
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(-11.0, 0.0), Shape::new_rect(vec2_f(2.0, 2.0))));
        a.vel.pos = vec2_f(2.0, 0.0);
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(12.0, 2.0), Shape::new_rect(vec2_f(2.0, 4.0))));
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
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(5.0, 1.0), Shape::new_rect(vec2_f(2.0, 2.0))));
        a.vel.pos = vec2_f(2.0, 1.0);
        a.duration = n64(100.0);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(5.0, 1.0), Shape::new_rect(vec2_f(2.0, 4.0))));
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
        let mut a = Hitbox::new(PlacedShape::new(vec2_f(0.0, 0.0), Shape::new_circle(r64(2.0))));
        a.duration = n64(4.0 - sqrt2 + 0.01);
        let mut b = Hitbox::new(PlacedShape::new(vec2_f(4.0, 4.0), Shape::new_circle(r64(2.0))));
        b.vel.pos = vec2_f(-1.0, -1.0);
        b.duration = n64(4.0 - sqrt2 + 0.01);
        assert!(a.collide_time(&b) == 4.0 - sqrt2);
        a.duration -= 0.02;
        assert!(a.collide_time(&b) == f64::INFINITY);
        b.duration -= 0.02;
        assert!(a.collide_time(&b) == f64::INFINITY);
    }
}
