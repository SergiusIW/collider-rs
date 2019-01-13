// Copyright 2016-2018 Matthew D. Michelotti
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

use super::{Collider, HbEvent, HbId, HbProfile, HbVel};
use geom::{v2, Shape};
use std::f64;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct TestHbProfile {
    id: HbId,
}

impl From<HbId> for TestHbProfile {
    fn from(id: HbId) -> TestHbProfile {
        TestHbProfile { id }
    }
}

impl HbProfile for TestHbProfile {
    fn id(&self) -> HbId {
        self.id
    }
    fn can_interact(&self, _other: &TestHbProfile) -> bool {
        true
    }
}

fn advance_to_event(collider: &mut Collider<TestHbProfile>, time: f64) {
    advance(collider, time);
    assert_eq!(collider.next_time(), collider.time());
}

fn advance(collider: &mut Collider<TestHbProfile>, time: f64) {
    while collider.time() < time {
        assert!(collider.next().is_none());
        let new_time = collider.next_time().min(time);
        collider.set_time(new_time);
    }
    assert_eq!(collider.time(), time);
}

fn advance_through_events(collider: &mut Collider<TestHbProfile>, time: f64) {
    while collider.time() < time {
        collider.next();
        let new_time = collider.next_time().min(time);
        collider.set_time(new_time);
    }
    assert_eq!(collider.time(), time);
}

fn sort(mut vector: Vec<TestHbProfile>) -> Vec<TestHbProfile> {
    vector.sort();
    vector
}

#[test]
fn smoke_test() {
    let mut collider = Collider::<TestHbProfile>::new(4.0, 0.25);

    let mut hitbox = Shape::square(2.0).place(v2(-10.0, 0.0)).still();
    hitbox.vel.value = v2(1.0, 0.0);
    let overlaps = collider.add_hitbox(0.into(), hitbox);
    assert_eq!(overlaps, vec![]);

    let mut hitbox = Shape::circle(2.0).place(v2(10.0, 0.0)).still();
    hitbox.vel.value = v2(-1.0, 0.0);
    let overlaps = collider.add_hitbox(1.into(), hitbox);
    assert_eq!(overlaps, vec![]);

    advance_to_event(&mut collider, 9.0);
    assert_eq!(
        collider.next(),
        Some((HbEvent::Collide, 0.into(), 1.into()))
    );
    advance_to_event(&mut collider, 11.125);
    assert_eq!(
        collider.next(),
        Some((HbEvent::Separate, 0.into(), 1.into()))
    );
    advance(&mut collider, 23.0);
}

#[test]
fn test_hitbox_updates() {
    let mut collider = Collider::<TestHbProfile>::new(4.0, 0.25);

    let mut hitbox = Shape::square(2.0).place(v2(-10.0, 0.0)).still();
    hitbox.vel.value = v2(1.0, 0.0);
    let overlaps = collider.add_hitbox(0.into(), hitbox);
    assert!(overlaps.is_empty());

    let mut hitbox = Shape::circle(2.0).place(v2(10.0, 0.0)).still();
    hitbox.vel.value = v2(1.0, 0.0);
    let overlaps = collider.add_hitbox(1.into(), hitbox);
    assert!(overlaps.is_empty());

    advance(&mut collider, 11.0);

    let mut hitbox = collider.get_hitbox(0);
    assert_eq!(hitbox.value, Shape::square(2.0).place(v2(1.0, 0.0)));
    assert_eq!(hitbox.vel.value, v2(1.0, 0.0));
    assert_eq!(hitbox.vel.resize, v2(0.0, 0.0));
    assert_eq!(hitbox.vel.end_time, f64::INFINITY);
    hitbox.value.pos = v2(0.0, 2.0);
    hitbox.vel.value = v2(0.0, -1.0);
    let overlaps = collider.remove_hitbox(0);
    assert_eq!(overlaps, vec![]);
    let overlaps = collider.add_hitbox(0.into(), hitbox);
    assert_eq!(overlaps, vec![]);

    advance(&mut collider, 14.0);

    let mut hitbox = collider.get_hitbox(1);
    assert_eq!(hitbox.value, Shape::circle(2.0).place(v2(24.0, 0.0)));
    assert_eq!(hitbox.vel.value, v2(1.0, 0.0));
    assert_eq!(hitbox.vel.resize, v2(0.0, 0.0));
    assert_eq!(hitbox.vel.end_time, f64::INFINITY);
    hitbox.value.pos = v2(0.0, -8.0);
    hitbox.vel.value = v2(0.0, 0.0);
    let overlaps = collider.remove_hitbox(1);
    assert_eq!(overlaps, vec![]);
    let overlaps = collider.add_hitbox(1.into(), hitbox);
    assert_eq!(overlaps, vec![]);

    advance_to_event(&mut collider, 19.0);

    assert_eq!(
        collider.next(),
        Some((HbEvent::Collide, 0.into(), 1.into()))
    );
    let mut hitbox = collider.get_hitbox(0);
    assert_eq!(hitbox.value, Shape::square(2.0).place(v2(0.0, -6.0)));
    assert_eq!(hitbox.vel.value, v2(0.0, -1.0));
    assert_eq!(hitbox.vel.resize, v2(0.0, 0.0));
    assert_eq!(hitbox.vel.end_time, f64::INFINITY);
    hitbox.vel.value = v2(0.0, 0.0);
    collider.set_hitbox_vel(0, hitbox.vel);

    let mut hitbox = collider.get_hitbox(1);
    assert_eq!(hitbox.value, Shape::circle(2.0).place(v2(0.0, -8.0)));
    assert_eq!(hitbox.vel.value, v2(0.0, 0.0));
    assert_eq!(hitbox.vel.resize, v2(0.0, 0.0));
    assert_eq!(hitbox.vel.end_time, f64::INFINITY);
    hitbox.vel.value = v2(0.0, 2.0);
    collider.set_hitbox_vel(1, hitbox.vel);

    let hitbox = Shape::rect(v2(2.0, 20.0)).place(v2(0.0, 0.0)).still();
    assert_eq!(
        sort(collider.add_hitbox(2.into(), hitbox)),
        vec![0.into(), 1.into()]
    );

    advance_to_event(&mut collider, 21.125);

    assert_eq!(
        collider.next(),
        Some((HbEvent::Separate, 0.into(), 1.into()))
    );

    advance(&mut collider, 26.125);

    let overlaps = collider.remove_hitbox(1);
    assert_eq!(overlaps, vec![2.into()]);

    advance(&mut collider, 37.125);
}

#[test]
fn test_get_overlaps() {
    let mut collider = Collider::<TestHbProfile>::new(4.0, 0.25);

    collider.add_hitbox(
        0.into(),
        Shape::square(2.0)
            .place(v2(-10.0, 0.0))
            .moving(v2(1.0, 0.0)),
    );
    collider.add_hitbox(
        1.into(),
        Shape::circle(2.0)
            .place(v2(10.0, 0.0))
            .moving(v2(-1.0, 0.0)),
    );
    collider.add_hitbox(2.into(), Shape::square(2.0).place(v2(0.0, 0.0)).still());

    assert_eq!(collider.get_overlaps(0), vec![]);
    assert_eq!(collider.get_overlaps(1), vec![]);
    assert_eq!(collider.get_overlaps(2), vec![]);
    assert!(!collider.is_overlapping(0, 1));
    assert!(!collider.is_overlapping(0, 2));
    assert!(!collider.is_overlapping(1, 2));
    assert!(!collider.is_overlapping(1, 0));

    advance_through_events(&mut collider, 10.0);

    assert_eq!(sort(collider.get_overlaps(0)), vec![1.into(), 2.into()]);
    assert_eq!(sort(collider.get_overlaps(1)), vec![0.into(), 2.into()]);
    assert_eq!(sort(collider.get_overlaps(2)), vec![0.into(), 1.into()]);
    assert!(collider.is_overlapping(0, 1));
    assert!(collider.is_overlapping(0, 2));
    assert!(collider.is_overlapping(1, 2));
    assert!(collider.is_overlapping(1, 0));

    collider.set_hitbox_vel(1, HbVel::moving(v2(1.0, 0.0)));
    advance_through_events(&mut collider, 20.0);

    assert_eq!(collider.get_overlaps(0), vec![1.into()]);
    assert_eq!(collider.get_overlaps(1), vec![0.into()]);
    assert_eq!(collider.get_overlaps(2), vec![]);
    assert!(collider.is_overlapping(0, 1));
    assert!(!collider.is_overlapping(0, 2));
    assert!(!collider.is_overlapping(1, 2));

    collider.remove_hitbox(2);
    assert_eq!(collider.get_overlaps(0), vec![1.into()]);
    assert_eq!(collider.get_overlaps(1), vec![0.into()]);
    assert!(collider.is_overlapping(0, 1));

    collider.remove_hitbox(1);
    assert_eq!(collider.get_overlaps(0), vec![]);
}

#[test]
fn test_query_overlaps() {
    let mut collider = Collider::<TestHbProfile>::new(4.0, 0.25);

    collider.add_hitbox(
        0.into(),
        Shape::square(2.0).place(v2(-5.0, 0.0)).moving(v2(1.0, 0.0)),
    );
    collider.add_hitbox(1.into(), Shape::circle(2.0).place(v2(0.0, 0.0)).still());
    collider.add_hitbox(
        2.into(),
        Shape::circle(2.0)
            .place(v2(10.0, 0.0))
            .moving(v2(-1.0, 0.0)),
    );

    let test_shape = Shape::circle(2.0).place(v2(-1.0, 0.5));
    assert_eq!(
        collider.query_overlaps(&test_shape, &5.into()),
        vec![1.into()]
    );

    advance(&mut collider, 3.0);
    assert_eq!(
        sort(collider.query_overlaps(&test_shape, &5.into())),
        vec![0.into(), 1.into()]
    );
}

#[test]
fn test_separate_initial_overlap() {
    let mut collider = Collider::<TestHbProfile>::new(4.0, 0.25);

    let overlaps = collider.add_hitbox(
        0.into(),
        Shape::square(1.).place(v2(0., 0.)).moving(v2(0.0, 1.)),
    );
    assert_eq!(overlaps, vec![]);
    let overlaps = collider.add_hitbox(1.into(), Shape::square(1.).place(v2(0., 0.)).still());
    assert_eq!(overlaps, vec![0.into()]);

    advance_to_event(&mut collider, 1.25);
    assert_eq!(
        collider.next(),
        Some((HbEvent::Separate, 0.into(), 1.into()))
    );

    advance(&mut collider, 1.5);
}

//TODO test custom interactivities...
