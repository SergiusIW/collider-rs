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

mod event_manager;

use std::collections::HashMap;
use std::mem;
use self::event_manager::{EventManager, EventKey};
use core::inter::{Interactivity, DefaultInteractivity};
use core::{Hitbox, HitboxId, HIGH_TIME};
use core::grid::Grid;
use util::{TightSet, OneOrTwo};

pub struct Collider<I: Interactivity = DefaultInteractivity> {
    hitboxes: HashMap<HitboxId, HitboxInfo<I>>,
    time: f64,
    grid: Grid,
    padding: f64,
    events: EventManager
}

impl <I: Interactivity> Collider<I> {
    pub fn new(cell_width: f64, padding: f64) -> Collider {
        Collider {
            hitboxes : HashMap::new(),
            time : 0.0,
            grid : Grid::new(cell_width),
            padding : padding,
            events : EventManager::new()
        }
    }
    
    pub fn time_until_next(&self) -> f64 {
        self.events.peek_time() - self.time
    }
    
    pub fn advance(&mut self, time: f64) {
        assert!(time >= 0.0, "time must be non-negative");
        self.time += time;
        assert!(self.time <= self.events.peek_time(), "time must not exceed time_until_next()");
        assert!(self.time < HIGH_TIME, "time must not exceed {}", HIGH_TIME);
    }
    
    pub fn next(&mut self) -> Option<Event> {
        while let Some(event) = self.events.next(self.time, &mut self.hitboxes) {
            if let Some(event) = self.process_event(event) {
                return Some(event);
            }
        }
        None
    }
    
    fn process_event(&mut self, event: InternalEvent) -> Option<Event> {
        match event {
            InternalEvent::Collide(id_1, id_2) => {
                let mut hitbox_info_1 = self.hitboxes.remove(&id_1).unwrap();
                let hitbox_info_2 = self.hitboxes.get_mut(&id_2).unwrap();
                assert!(hitbox_info_1.overlaps.insert(id_2), "illegal state");
                assert!(hitbox_info_2.overlaps.insert(id_1), "illegal state");
                let delay = hitbox_info_1.hitbox_at_time(self.time).separate_time(&hitbox_info_2.hitbox_at_time(self.time), self.padding);
                self.events.add_pair_event(self.time + delay, InternalEvent::Separate(id_1, id_2),
                    &mut hitbox_info_1.event_keys, &mut hitbox_info_2.event_keys);
                assert!(self.hitboxes.insert(id_1, hitbox_info_1).is_none(), "illegal state");
                Some(Event::new_collide(id_1, id_2))
            },
            InternalEvent::Separate(id_1, id_2) => {
                let mut hitbox_info_1 = self.hitboxes.remove(&id_1).unwrap();
                let hitbox_info_2 = self.hitboxes.get_mut(&id_2).unwrap();
                assert!(hitbox_info_1.overlaps.remove(&id_2), "illegal state");
                assert!(hitbox_info_2.overlaps.remove(&id_1), "illegal state");
                let delay = hitbox_info_1.hitbox_at_time(self.time).collide_time(&hitbox_info_2.hitbox_at_time(self.time));
                self.events.add_pair_event(self.time + delay, InternalEvent::Collide(id_1, id_2),
                    &mut hitbox_info_1.event_keys, &mut hitbox_info_2.event_keys);
                assert!(self.hitboxes.insert(id_1, hitbox_info_1).is_none(), "illegal state");
                Some(Event::new_separate(id_1, id_2))
            },
            InternalEvent::Reiterate(id) => {
                let new_hitbox = self.hitboxes[&id].pub_hitbox_at_time(self.time);
                self.update_hitbox(id, new_hitbox);
                None
            },
            InternalEvent::PanicSmallHitbox(id) => {
                panic!("hitbox {} became too small", id)
            },
            InternalEvent::PanicDurationPassed(id) => {
                panic!("hitbox {} was not updated before duration passed", id)
            }
        }
    }
    
    pub fn add_hitbox(&mut self, id: HitboxId, hitbox: Hitbox, interactivity: I) {
        let hitbox_info = HitboxInfo::new(hitbox, interactivity, self.time);
        assert!(self.hitboxes.insert(id, hitbox_info).is_none(), "hitbox id {} already in use", id);
        self.internal_update_hitbox(id, None, None, Phase::Add);
    }
    
    pub fn remove_hitbox(&mut self, id: HitboxId, hitbox: Hitbox, interactivity: I) {
        self.internal_update_hitbox(id, None, None, Phase::Remove);
        self.hitboxes.remove(id);
    }
    
    pub fn get_hitbox(&self, id: HitboxId) -> Hitbox {
        self.hitboxes[id].hitbox_at_time(self.time)
    }
    
    pub fn update_hitbox(&mut self, id: HitboxId, hitbox: Hitbox) {
        self.internal_update_hitbox(id, Some(hitbox), None, Phase::Update);
    }
    
    pub fn update_interactivity(&mut self, id: HitboxId, interactivity: I) {
        self.internal_update_hitbox(id, None, Some(interactivity), Phase::Update);
    }
    
    pub fn update_hitbox_and_interactivity(&mut self, id: HitboxId, hitbox: Hitbox, interactivity: I) {
        self.internal_update_hitbox(id, Some(hitbox), Some(interactivity), Phase::Update);
    }
    
    fn internal_update_hitbox(&mut self, id: HitboxId, hitbox: Option<Hitbox>, interactivity: Option<I>, phase: Phase) {
        let mut hitbox_info = self.hitboxes.remove(id).unwrap_or_else(panic!("hitbox id {} not found", id));
        let mut hitbox = hitbox.or_else(|| hitbox_info.pub_hitbox(self.time));
        hitbox.validate(self.padding);
        self.events.clear_related_events(id, &mut hitbox_info.key_set, self.hitboxes);
        
        mem::swap(&mut hitbox, &mut hitbox_info.hitbox);
        let old_hitbox = hitbox;
        self.solitaire_event_check(id, &mut hitbox_info);
        
        let old_group = hitbox_info.interactivity.group();
        let (old_group, new_group) = match (phase, interactivity) {
            (Phase::Add, None) => (None, old_group),
            (Phase::Remove, None) => {
                self.clear_overlaps(id, &mut hitbox_info);
                (old_group, None)
            },
            (Phase::Update, None) => (old_group, old_group),
            (Phase::Update, Some(interactivity)) => {
                hitbox_info.interactivity = interactivity;
                let new_group = hitbox_info.interactivity.group();
                if new_group.is_none() {
                    self.clear_overlaps(id, &mut hitbox_info);
                } else {
                    self.recheck_overlap_interactivity(id, &mut hitbox_info);
                }
                (old_group, new_group)
            },
            _ => unreachable!()
        };
        let test_ids = self.grid.update_hitbox(
            id, (&old_hitbox, old_group), (&hitbox_info.hitbox, new_group), hitbox.interactivity.interact_groups()
        );
        hitbox_info.start_time = self.time;
        
        for other_id in test_ids {
            if !hitbox_info.overlaps.contains(other_id) {
                let other_hitbox_info = self.hitboxes.get(other_id).unwrap();
                if hitbox_info.interactivity.can_interact(other_hitbox_info.interactivity) {
                    let delay = hitbox_info.hitbox.collide_time(&other_hitbox_info.hitbox_at_time(self.time));
                    self.events.add_pair_event(self.time + delay, InternalEvent::Collide(id, other_id), hitbox_info.event_keys, other_hitbox_info.event_keys);
                }
            }
        }
        for other_id in hitbox_info.overlaps.clone() {
            let other_hitbox_info = self.hitboxes.get(other_id).unwrap();
            let delay = hitbox_info.hitbox.separate_time(&other_hitbox_info.hitbox_at_time(self.time), self.padding);
            self.events.add_pair_event(self.time + delay, InternalEvent::Separate(id, other_id), hitbox_info.event_keys, other_hitbox_info.event_keys);
        }
        
        assert!(self.hitboxes.insert(id, hitbox).is_none(), "illegal state");
    }
    
    fn recheck_overlap_interactivity(&mut self, id: HitboxId, hitbox_info: &mut HitboxInfo<I>) {
        for other_id in hitbox_info.overlaps.clone() {
            let other_hitbox_info = self.hitboxes.get_mut(other_id).unwrap();
            if !hitbox_info.interactivity.can_interact(other_hitbox_info.interactivity) {
                assert!(hitbox_info.overlaps.remove(other_id), "illegal state");
                assert!(other_hitbox_info.overlaps.remove(id), "illegal state");
            }
        }
    }
    
    fn clear_overlaps(&mut self, id: HitboxId, hitbox_info: &mut HitboxInfo<I>) {
        for other_id in hitbox_info.overlaps {
            let other_hitbox_info = self.hitboxes.get_mut(other_id).unwrap();
            assert!(other_hitbox_info.overlaps.remove(id), "illegal state");
        }
        hitbox_info.overlaps.clear();
    }
    
    fn solitaire_event_check(&mut self, id: HitboxId, hitbox_info: &mut HitboxInfo<I>) {
        hitbox_info.pub_duration = hitbox_info.hitbox.duration;
        let mut result = (self.grid.cell_period(hitbox_info.hitbox), InternalEvent::Reiterate(id));
        let delay = hitbox_info.hitbox.duration;
        if delay < result.0 { result = (delay, InternalEvent::PanicDurationPassed(id)); }
        let delay = hitbox_info.hitbox.time_until_too_small(self.padding);
        if delay < result.0 { result = (delay, InternalEvent::PanicSmallHitbox(id)); }
        hitbox_info.hitbox.duration = result.0;
        self.events.add_solitaire_event(self.time + result.0, result.1, hitbox_info.key_set);
    }
}

enum Phase {
    Add, Remove, Update
}

//TODO split HitboxInfo into two structs, one with interactivity and one without, so as to avoid unnecessary generic code...
struct HitboxInfo<I: Interactivity> {
    interactivity: I,
    hitbox: Hitbox,
    start_time: f64,
    pub_duration: f64,
    event_keys: TightSet<EventKey>,
    overlaps: TightSet<HitboxId>
}

impl <I: Interactivity> HitboxInfo<I> {
    fn new(hitbox: Hitbox, interactivity: Interactivity, start_time: f64) {
        HitboxInfo {
            interactivity: interactivity,
            hitbox: hitbox,
            start_time: start_time,
            pub_duration: hitbox.duration,
            event_keys: TightSet::new(),
            overlaps: TightSet::new()
        }
    }

    fn hitbox_at_time(&self, time: f64) -> Hitbox {
        let mut result = self.hitbox.clone();
        result.advance(self.start_time, time);
        result
    }
    
    fn pub_hitbox_at_time(&self, time: f64) -> Hitbox {
        let mut result = self.hitbox.clone();
        result.duration = self.pub_duration;
        result.advance(self.start_time, time);
        result
    }
}

pub enum EventKind {
    Collide, Separate
}

pub struct Event {
    id_1: HitboxId,
    id_2: HitboxId,
    kind: EventKind
}

impl Event {
    fn new_collide(id_1: HitboxId, id_2: HitboxId) -> Event {
        Event { id_1 : id_1, id_2 : id_2, kind : EventKind::Collide }
    }
    
    fn new_separate(id_1: HitboxId, id_2: HitboxId) -> Event {
        Event { id_1 : id_1, id_2 : id_2, kind : EventKind::Separate }
    }
}

#[derive(Copy, Clone)]
enum InternalEvent {
    PanicSmallHitbox(HitboxId),
    PanicDurationPassed(HitboxId),
    Reiterate(HitboxId),
    Collide(HitboxId, HitboxId),
    Separate(HitboxId, HitboxId)
}

impl InternalEvent {
    fn other_id(self, id: HitboxId) -> Option<HitboxId> {
        self.involved_hitbox_ids().other_id(id)
    }
}

impl InternalEvent {
    fn involved_hitbox_ids(self) -> OneOrTwo<HitboxId> {
        match self {
            InternalEvent::PanicSmallHitbox(id) | InternalEvent::PanicDurationPassed(id) | InternalEvent::Reiterate(id) => OneOrTwo::One(id),
            InternalEvent::Collide(a, b) | InternalEvent::Separate(a, b) => OneOrTwo::Two(a, b)
        }
    }
}