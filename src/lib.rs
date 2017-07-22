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
//! use collider::{Collider, HbEvent, HbId, HbProfile};
//! use collider::geom::{Shape, v2};
//!
//! #[derive(Copy, Clone, Debug)]
//! struct DemoHbProfile { id: HbId } // add any additional identfying data to this struct
//!
//! impl HbProfile for DemoHbProfile {
//!     fn id(&self) -> HbId { self.id }
//!     fn can_interact(&self, _other: &DemoHbProfile) -> bool { true }
//!     fn cell_width() -> f64 { 4.0 }
//!     fn padding() -> f64 { 0.01 }
//! }
//!
//! let mut collider: Collider<DemoHbProfile> = Collider::new();
//!
//! let hitbox = Shape::square(2.0).place(v2(-10.0, 0.0)).moving(v2(1.0, 0.0));
//! let overlaps = collider.add_hitbox(DemoHbProfile { id: 0 }, hitbox);
//! assert!(overlaps.is_empty());
//!
//! let hitbox = Shape::square(2.0).place(v2(10.0, 0.0)).moving(v2(-1.0, 0.0));
//! let overlaps = collider.add_hitbox(DemoHbProfile { id: 1 }, hitbox);
//! assert!(overlaps.is_empty());
//!
//! while collider.time() < 20.0 {
//!     let time = collider.next_time().min(20.0);
//!     collider.set_time(time);
//!     if let Some((event, pr_1, pr_2)) = collider.next() {
//!         println!("{:?} between {:?} and {:?} at time {}.",
//!                  event, pr_1, pr_2, collider.time());
//!         if event == HbEvent::Collide {
//!             println!("Speed of collided hitboxes is halved.");
//!             for pr in [pr_1, pr_2].iter() {
//!                 let mut hb_vel = collider.get_hitbox(pr.id()).vel;
//!                 hb_vel.value *= 0.5;
//!                 collider.set_hitbox_vel(pr.id(), hb_vel);
//!             }
//!         }
//!     }
//! }
//!
//! // the above loop prints the following events:
//! //   Collide between DemoHbProfile { id: 0 } and DemoHbProfile { id: 1 } at time 9.
//! //   Speed of collided hitboxes is halved.
//! //   Separate between DemoHbProfile { id: 0 } and DemoHbProfile { id: 1 } at time 13.01.
//! ```

extern crate fnv;

mod float;
pub mod geom;
mod util;
mod core;
mod index_rect;

pub use core::*;

#[cfg(test)]
mod tests {
    use std::f64;
    use std::collections::HashSet;
    use super::{Collider, HbEvent, HbId, HbProfile};
    use geom::{PlacedShape, Shape, v2};

    #[derive(Copy, Clone, PartialEq, Eq)]
    struct TestHbProfile { id: HbId }

    impl From<HbId> for TestHbProfile {
        fn from(id: HbId) -> TestHbProfile {
            TestHbProfile { id }
        }
    }

    impl HbProfile for TestHbProfile {
        fn id(&self) -> HbId { self.id }
        fn can_interact(&self, _other: &TestHbProfile) -> bool { true }
        fn cell_width() -> f64 { 4.0 }
        fn padding() -> f64 { 0.25 }
    }

    fn advance_to_event(collider: &mut Collider<TestHbProfile>, time: f64) {
        advance(collider, time);
        assert!(collider.next_time() == collider.time());
    }

    fn advance(collider: &mut Collider<TestHbProfile>, time: f64) {
        while collider.time() < time {
            assert!(collider.next().is_none());
            let new_time = collider.next_time().min(time);
            collider.set_time(new_time);
        }
        assert!(collider.time() == time);
    }

    #[test]
    fn smoke_test() {
        let mut collider = Collider::<TestHbProfile>::new();

        let mut hitbox = PlacedShape::new(v2(-10.0, 0.0), Shape::square(2.0)).still();
        hitbox.vel.value = v2(1.0, 0.0);
        let overlaps = collider.add_hitbox(0.into(), hitbox);
        assert!(overlaps.is_empty());

        let mut hitbox = PlacedShape::new(v2(10.0, 0.0), Shape::circle(2.0)).still();
        hitbox.vel.value = v2(-1.0, 0.0);
        let overlaps = collider.add_hitbox(1.into(), hitbox);
        assert!(overlaps.is_empty());

        advance_to_event(&mut collider, 9.0);
        assert!(collider.next() == Some((HbEvent::Collide, 0.into(), 1.into())));
        advance_to_event(&mut collider, 11.125);
        assert!(collider.next() == Some((HbEvent::Separate, 0.into(), 1.into())));
        advance(&mut collider, 23.0);
    }

    #[test]
    fn test_hitbox_updates() {
        let mut collider = Collider::<TestHbProfile>::new();

        let mut hitbox = PlacedShape::new(v2(-10.0, 0.0), Shape::square(2.0)).still();
        hitbox.vel.value = v2(1.0, 0.0);
        let overlaps = collider.add_hitbox(0.into(), hitbox);
        assert!(overlaps.is_empty());

        let mut hitbox = PlacedShape::new(v2(10.0, 0.0), Shape::circle(2.0)).still();
        hitbox.vel.value = v2(1.0, 0.0);
        let overlaps = collider.add_hitbox(1.into(), hitbox);
        assert!(overlaps.is_empty());

        advance(&mut collider, 11.0);

        let mut hitbox = collider.get_hitbox(0);
        assert!(hitbox.value == PlacedShape::new(v2(1.0, 0.0), Shape::square(2.0)));
        assert!(hitbox.vel.value == v2(1.0, 0.0));
        assert!(hitbox.vel.resize == v2(0.0, 0.0));
        assert!(hitbox.vel.end_time == f64::INFINITY);
        hitbox.value.pos = v2(0.0, 2.0);
        hitbox.vel.value = v2(0.0, -1.0);
        let overlaps = collider.remove_hitbox(0);
        assert!(overlaps.is_empty());
        let overlaps = collider.add_hitbox(0.into(), hitbox);
        assert!(overlaps.is_empty());

        advance(&mut collider, 14.0);

        let mut hitbox = collider.get_hitbox(1);
        assert!(hitbox.value == PlacedShape::new(v2(24.0, 0.0), Shape::circle(2.0)));
        assert!(hitbox.vel.value == v2(1.0, 0.0));
        assert!(hitbox.vel.resize == v2(0.0, 0.0));
        assert!(hitbox.vel.end_time == f64::INFINITY);
        hitbox.value.pos = v2(0.0, -8.0);
        hitbox.vel.value = v2(0.0, 0.0);
        let overlaps = collider.remove_hitbox(1);
        assert!(overlaps.is_empty());
        let overlaps = collider.add_hitbox(1.into(), hitbox);
        assert!(overlaps.is_empty());

        advance_to_event(&mut collider, 19.0);

        assert!(collider.next() == Some((HbEvent::Collide, 0.into(), 1.into())));
        let mut hitbox = collider.get_hitbox(0);
        assert!(hitbox.value == PlacedShape::new(v2(0.0, -6.0), Shape::square(2.0)));
        assert!(hitbox.vel.value == v2(0.0, -1.0));
        assert!(hitbox.vel.resize == v2(0.0, 0.0));
        assert!(hitbox.vel.end_time == f64::INFINITY);
        hitbox.vel.value = v2(0.0, 0.0);
        collider.set_hitbox_vel(0, hitbox.vel);

        let mut hitbox = collider.get_hitbox(1);
        assert!(hitbox.value == PlacedShape::new(v2(0.0, -8.0), Shape::circle(2.0)));
        assert!(hitbox.vel.value == v2(0.0, 0.0));
        assert!(hitbox.vel.resize == v2(0.0, 0.0));
        assert!(hitbox.vel.end_time == f64::INFINITY);
        hitbox.vel.value = v2(0.0, 2.0);
        collider.set_hitbox_vel(1, hitbox.vel);

        let hitbox = PlacedShape::new(v2(0.0, 0.0), Shape::rect(v2(2.0, 20.0))).still();
        let overlaps = collider.add_hitbox(2.into(), hitbox);
        let overlaps_set: HashSet<_> = overlaps.iter().map(|pr| pr.id()).collect();
        assert!(overlaps_set.contains(&0));
        assert!(overlaps_set.contains(&1));
        assert!(overlaps_set.len() == 2);

        advance_to_event(&mut collider, 21.125);

        assert!(collider.next() == Some((HbEvent::Separate, 0.into(), 1.into())));

        advance(&mut collider, 26.125);

        let overlaps = collider.remove_hitbox(1);
        assert!(overlaps == vec![2.into()]);

        advance(&mut collider, 37.125);
    }

    //TODO test custom interactivities...
}
