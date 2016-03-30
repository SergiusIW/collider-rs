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
        let mut collider: Collider = Collider::new(4.0, 0.25);
        
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
        let mut collider: Collider = Collider::new(4.0, 0.25);
        
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