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
    use super::{Collider, Hitbox, Event};
    use geom::{PlacedShape, Shape, Vec2};
    
    fn advance_to_next_event(collider: &mut Collider, mut time: f64) {
        while time > 0.0 {
            assert!(collider.next() == None);
            let timestep = collider.time_until_next();
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
        advance_to_next_event(&mut collider, 9.0);
        assert!(collider.next() == Some(Event::new_collide(0, 1)));
        assert!(collider.next() == None);
        advance_to_next_event(&mut collider, 2.125);
        assert!(collider.next() == Some(Event::new_separate(0, 1)));
        assert!(collider.next() == None);
    }
    
    //TODO add some more thorough tests
}