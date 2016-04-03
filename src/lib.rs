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

//! Collider is a library for continuous 2D collision detection,
//! for use with game developement.
//!
//! Most game engines follow the approach of periodically updating the
//! positions of all shapes and checking for collisions at a frozen snapshot in time.
//! [Continuous collision detection](https://en.wikipedia.org/wiki/Collision_detection#A_posteriori_.28discrete.29_versus_a_priori_.28continuous.29),
//! on the other hand, means that the time of collision is determined very precisely,
//! and the user is not restricted to a fixed time-stepping method.
//! There are currently two kinds of shapes supported by Collider: circles and rectangles.
//! The user specifies the positions and velocites of these shapes, which
//! they can update at any time, and Collider will solve for the precise times of
//! collision and separation.
//!
//! There are certain advantages that continuous collision detection
//! holds over the traditional approach.
//! In a game engine, the position of a sprite may be updated to overlap a wall,
//! and in a traditional collision system there would need to be a post-correction
//! to make sure the sprite does not appear inside of the wall.
//! This is not needed with continuous collision detection, since
//! the precise time and location at which the sprite touches the wall is known.
//! Traditional collision detection may have an issue with "tunneling," in which a
//! fast small object runs into a narrow wall and collision detection misses it,
//! or two fast small objects fly right through each other and collision detection misses it.
//! This is also not a problem for contiuous collision detection.
//! It is also debatable that continuous collision detection may be
//! more efficient in certain circumstances,
//! since the hitboxes may be updated less frequently and still maintain a
//! smooth appearance over time.
//!
//! #Example
//! ```
//! use collider::{Collider, Hitbox, Event};
//! use collider::geom::{PlacedShape, Shape, Vec2};
//! 
//! let mut collider: Collider = Collider::new(4.0, 0.01);
//!
//! let mut hitbox = Hitbox::new(PlacedShape::new(Vec2::new(-10.0, 0.0), Shape::new_rect(2.0, 2.0)));
//! hitbox.vel.pos = Vec2::new(1.0, 0.0);
//! collider.add_hitbox(0, hitbox);
//! 
//! let mut hitbox = Hitbox::new(PlacedShape::new(Vec2::new(10.0, 0.0), Shape::new_circle(2.0)));
//! hitbox.vel.pos = Vec2::new(-1.0, 0.0);
//! collider.add_hitbox(1, hitbox);
//!
//! let mut time = 0.0;
//! while time < 9.0 {
//!     assert!(collider.next() == None);
//!     let timestep = collider.time_until_next();
//!     collider.advance(timestep);
//!     time += timestep;
//! }
//! assert!(time == 9.0);
//! assert!(collider.next() == Some((Event::Collide, 0, 1)));
//! ```

pub mod geom;
mod geom_ext;
mod util;
mod core;
mod index_rect;

pub use core::*;

#[cfg(test)]
mod tests {
    use std::f64;
    use std::mem;
    use super::{Collider, Hitbox, Event};
    use geom::{PlacedShape, Shape, Vec2};
    
    fn advance_to_event(collider: &mut Collider, time: f64) {
        advance(collider, time);
        assert!(collider.time_until_next() == 0.0);
    }
    
    fn advance(collider: &mut Collider, mut time: f64) {
        while time > 0.0 {
            assert!(collider.next() == None);
            let timestep = collider.time_until_next().min(time);
            time -= timestep;
            collider.advance(timestep);
        }
        assert!(time == 0.0);
    }
    
    #[test]
    fn smoke_test() {
        let mut collider = Collider::new(4.0, 0.25);
        
        let mut hitbox = Hitbox::new(PlacedShape::new(Vec2::new(-10.0, 0.0), Shape::new_rect(2.0, 2.0)));
        hitbox.vel.pos = Vec2::new(1.0, 0.0);
        collider.add_hitbox(0, hitbox);
        
        let mut hitbox = Hitbox::new(PlacedShape::new(Vec2::new(10.0, 0.0), Shape::new_circle(2.0)));
        hitbox.vel.pos = Vec2::new(-1.0, 0.0);
        collider.add_hitbox(1, hitbox);
        
        advance_to_event(&mut collider, 9.0);
        assert!(collider.next() == Some((Event::Collide, 0, 1)));
        advance_to_event(&mut collider, 2.125);
        assert!(collider.next() == Some((Event::Separate, 0, 1)));
        advance(&mut collider, 11.0);
    }
    
    #[test]
    fn test_hitbox_updates() {
        let mut collider = Collider::new(4.0, 0.25);
        
        let mut hitbox = Hitbox::new(PlacedShape::new(Vec2::new(-10.0, 0.0), Shape::new_rect(2.0, 2.0)));
        hitbox.vel.pos = Vec2::new(1.0, 0.0);
        collider.add_hitbox(0, hitbox);
        
        let mut hitbox = Hitbox::new(PlacedShape::new(Vec2::new(10.0, 0.0), Shape::new_circle(2.0)));
        hitbox.vel.pos = Vec2::new(1.0, 0.0);
        collider.add_hitbox(1, hitbox);
        
        advance(&mut collider, 11.0);
        
        let mut hitbox = collider.get_hitbox(0);
        assert!(hitbox.shape == PlacedShape::new(Vec2::new(1.0, 0.0), Shape::new_rect(2.0, 2.0)));
        assert!(hitbox.vel == PlacedShape::new(Vec2::new(1.0, 0.0), Shape::new_rect(0.0, 0.0)));
        assert!(hitbox.duration == f64::INFINITY);
        hitbox.shape.pos = Vec2::new(0.0, 2.0);
        hitbox.vel.pos = Vec2::new(0.0, -1.0);
        collider.update_hitbox(0, hitbox);
        
        advance(&mut collider, 3.0);
        
        let mut hitbox = collider.get_hitbox(1);
        assert!(hitbox.shape == PlacedShape::new(Vec2::new(24.0, 0.0), Shape::new_circle(2.0)));
        assert!(hitbox.vel == PlacedShape::new(Vec2::new(1.0, 0.0), Shape::new_circle(0.0)));
        assert!(hitbox.duration == f64::INFINITY);
        hitbox.shape.pos = Vec2::new(0.0, -8.0);
        hitbox.vel.pos = Vec2::new(0.0, 0.0);
        collider.update_hitbox(1, hitbox);
        
        advance_to_event(&mut collider, 5.0);
        
        assert!(collider.next() == Some((Event::Collide, 0, 1)));
        let mut hitbox = collider.get_hitbox(0);
        assert!(hitbox.shape == PlacedShape::new(Vec2::new(0.0, -6.0), Shape::new_rect(2.0, 2.0)));
        assert!(hitbox.vel == PlacedShape::new(Vec2::new(0.0, -1.0), Shape::new_rect(0.0, 0.0)));
        assert!(hitbox.duration == f64::INFINITY);
        hitbox.vel.pos = Vec2::new(0.0, 0.0);
        collider.update_hitbox(0, hitbox);
        
        let mut hitbox = collider.get_hitbox(1);
        assert!(hitbox.shape == PlacedShape::new(Vec2::new(0.0, -8.0), Shape::new_circle(2.0)));
        assert!(hitbox.vel == PlacedShape::new(Vec2::new(0.0, 0.0), Shape::new_circle(0.0)));
        assert!(hitbox.duration == f64::INFINITY);
        hitbox.vel.pos = Vec2::new(0.0, 2.0);
        collider.update_hitbox(1, hitbox);
        
        let hitbox = Hitbox::new(PlacedShape::new(Vec2::new(0.0, 0.0), Shape::new_rect(2.0, 20.0)));
        collider.add_hitbox(2, hitbox);
        
        assert!(collider.time_until_next() == 0.0);
        let (mut event_1, mut event_2) = (collider.next(), collider.next());
        if event_1.unwrap().1 == 1 { mem::swap(&mut event_1, &mut event_2); }
        assert!(event_1 == Some((Event::Collide, 0, 2)));
        assert!(event_2 == Some((Event::Collide, 1, 2)));
        
        advance_to_event(&mut collider, 2.125);
        
        assert!(collider.next() == Some((Event::Separate, 0, 1)));
        
        advance(&mut collider, 5.0);
        
        collider.remove_hitbox(1);

        advance(&mut collider, 11.0);
    }
    
    //TODO test custom interactivities and interactivity changes...
}