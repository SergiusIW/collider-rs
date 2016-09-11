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

use std::collections::HashMap;
use std::mem;
use float::*;
use core::events::{EventManager, EventKey, EventKeysMap, InternalEvent};
use core::inter::{Interactivity, DefaultInteractivity, Group};
use core::{Hitbox, HitboxId, HIGH_TIME};
use core::grid::Grid;
use util::TightSet;

/// A structure that tracks hitboxes and returns collide/separate events.
pub struct Collider<I: Interactivity = DefaultInteractivity> {
    hitboxes: HashMap<HitboxId, HitboxInfo<I>>,
    time: N64,
    grid: Grid,
    padding: R64,
    events: EventManager
}

impl <I: Interactivity> Collider<I> {
    /// Constructs a new `Collider` instance.
    ///
    /// To reduce the number of overlaps that are tested,
    /// hitboxes are placed in a fixed grid structure behind the scenes.
    /// `cell_width` is the width of the cells in this grid.
    /// If your projct uses a similar grid, then it is usually a good choice
    /// to use the same cell width as that grid.
    /// Otherwise, a good choice is to use a width that is slightly larger
    /// than most of the hitboxes.
    ///
    /// Collider generates both `Collide` and `Separate` events.
    /// However, due to numerical error, it is important that two hitboxes
    /// be a certain small distance apart from each other after a collision
    /// before they are considered separated.
    /// Otherwise false separation events may occur if, for example,
    /// a sprite runs into a wall and stops, still touching the wall.
    /// `padding` is used to describe what this minimum separation distance is.
    /// This should typically be something that is not visible to the
    /// user, perhaps a fraction of a "pixel."
    /// Another restriction introduced by `padding` is that hitboxes are not
    /// allowed to have a width or height smaller than `padding`.
    pub fn new(cell_width: R64, padding: R64) -> Collider<I> {
        assert!(cell_width > padding, "requires cell_width > padding");
        assert!(padding > 0.0, "requires padding > 0.0");
        Collider {
            hitboxes : HashMap::new(),
            time : n64(0.0),
            grid : Grid::new(cell_width),
            padding : padding,
            events : EventManager::new()
        }
    }
    
    /// Returns the time until `self.next()` needs to be called again.
    ///
    /// Even if `self.time_until_next() == 0.0`, there is a chance that
    /// calling `self.next()` will return `None`, having processed an internal event.
    /// Regardless, after `self.next()` has been called repeatedly until it
    /// returns `None`, then `self.time_until_next()` will be greater than `0.0` again.
    ///
    /// This is a fast constant-time operation.  The result may be infinity.
    pub fn time_until_next(&self) -> N64 {
        self.events.peek_time() - self.time
    }
    
    /// Advances the positions of all hitboxes, based on the velocities of the hitboxes,
    /// by the given amount of `time`.
    /// Will panic if `time` exceeds `self.time_until_next()`.
    ///
    /// The hitboxes are updated implicitly, and this is actually a
    /// fast constant-time operation.
    pub fn advance(&mut self, time: N64) {
        assert!(time >= 0.0, "time must be non-negative");
        self.time += time;
        assert!(self.time <= self.events.peek_time(), "time must not exceed time_until_next()");
        assert!(self.time < HIGH_TIME, "time must not exceed {}", HIGH_TIME);
    }
    
    /// Processes and returns the next `Collide` or `Separate` event,
    /// or returns `None` if there are no more events that occured at the given time
    /// (although an internal event might have been processed if `None` is returned).
    /// Will always return `None` if `self.time_until_next() > 0.0`.
    ///
    /// The returned value is a tuple, denoting the type of event (`Collide` or `Separate`)
    /// and the two `HitboxId`s involved, in increasing order.
    pub fn next(&mut self) -> Option<(Event, HitboxId, HitboxId)> {
        while let Some(event) = self.events.next(self.time, &mut self.hitboxes) {
            if let Some(event) = self.process_event(event) {
                return Some(event);
            }
        }
        None
    }
    
    fn process_event(&mut self, event: InternalEvent) -> Option<(Event, HitboxId, HitboxId)> {
        match event {
            InternalEvent::Collide(id_1, id_2) => {
                let mut hitbox_info_1 = self.hitboxes.remove(&id_1).unwrap();
                {
                    let hitbox_info_2 = self.hitboxes.get_mut(&id_2).unwrap();
                    assert!(hitbox_info_1.overlaps.insert(id_2), "illegal state");
                    assert!(hitbox_info_2.overlaps.insert(id_1), "illegal state");
                    let delay = hitbox_info_1.hitbox_at_time(self.time).separate_time(&hitbox_info_2.hitbox_at_time(self.time), self.padding);
                    self.events.add_pair_event(self.time + delay, InternalEvent::Separate(id_1, id_2),
                        &mut hitbox_info_1.event_keys, &mut hitbox_info_2.event_keys);
                }
                assert!(self.hitboxes.insert(id_1, hitbox_info_1).is_none(), "illegal state");
                Some(new_event(Event::Collide, id_1, id_2))
            },
            InternalEvent::Separate(id_1, id_2) => {
                let mut hitbox_info_1 = self.hitboxes.remove(&id_1).unwrap();
                {
                    let hitbox_info_2 = self.hitboxes.get_mut(&id_2).unwrap();
                    assert!(hitbox_info_1.overlaps.remove(&id_2), "illegal state");
                    assert!(hitbox_info_2.overlaps.remove(&id_1), "illegal state");
                    let delay = hitbox_info_1.hitbox_at_time(self.time).collide_time(&hitbox_info_2.hitbox_at_time(self.time));
                    self.events.add_pair_event(self.time + delay, InternalEvent::Collide(id_1, id_2),
                        &mut hitbox_info_1.event_keys, &mut hitbox_info_2.event_keys);
                }
                assert!(self.hitboxes.insert(id_1, hitbox_info_1).is_none(), "illegal state");
                Some(new_event(Event::Separate, id_1, id_2))
            },
            InternalEvent::Reiterate(id) => {
                self.internal_update_hitbox(id, None, None, Phase::Update);
                None
            },
            #[cfg(debug_assertions)]
            InternalEvent::PanicSmallHitbox(id) => {
                panic!("hitbox {} became too small", id)
            },
            #[cfg(debug_assertions)]
            InternalEvent::PanicDurationPassed(id) => {
                panic!("hitbox {} was not updated before duration passed", id)
            }
        }
    }
    
    /// Adds a new hitbox to the collider.
    /// The `id` may be used to track the hitbox over time (will panic if there is an id clash).
    /// `hitbox` is the initial state of the hitbox.
    /// `interactivity` determines which other hitboxes should be checked for `Collide`/`Separate` events.
    pub fn add_hitbox_with_interactivity(&mut self, id: HitboxId, hitbox: Hitbox, interactivity: I) {
        let hitbox_info = HitboxInfo::new(hitbox, interactivity, self.time);
        assert!(self.hitboxes.insert(id, hitbox_info).is_none(), "hitbox id {} already in use", id);
        self.internal_update_hitbox(id, None, None, Phase::Add);
    }
    
    /// Removes the hitbox with the given `id` from all tracking.
    /// No further events will be generated for this hitbox.
    pub fn remove_hitbox(&mut self, id: HitboxId) {
        self.internal_update_hitbox(id, None, None, Phase::Remove);
        self.hitboxes.remove(&id);
    }
    
    /// Returns the current state of the hitbox with the given `id`.
    pub fn get_hitbox(&self, id: HitboxId) -> Hitbox {
        self.hitboxes[&id].pub_hitbox_at_time(self.time)
    }
    
    /// Updates the hitbox with the given `id` to match the position, shape, and velocity
    /// provided by `hitbox`.
    ///
    /// If this hitbox had collided (and not separated) with another htibox and
    /// still overlaps after this update, then no new `Collide`/`Separate` events
    /// are generated immediately.
    pub fn update_hitbox(&mut self, id: HitboxId, hitbox: Hitbox) {
        self.internal_update_hitbox(id, Some(hitbox), None, Phase::Update);
    }
    
    /// Sets the interactivity of the hitbox with the given `id` to the new value `interactivity`.
    ///
    /// If this hitbox was currently overlapping with other hitboxes and the new `interactivity`
    /// does not care about such overlaps, then the overlaps will ceased to be tracked
    /// without generating a `Separation` event.
    pub fn update_interactivity(&mut self, id: HitboxId, interactivity: I) {
        self.internal_update_hitbox(id, None, Some(interactivity), Phase::Update);
    }
    
    /// Invokes the functionality of both `update_hitbox` and `update_interactivity`,
    /// but is more efficient than calling the two methods separately.
    pub fn update_hitbox_and_interactivity(&mut self, id: HitboxId, hitbox: Hitbox, interactivity: I) {
        self.internal_update_hitbox(id, Some(hitbox), Some(interactivity), Phase::Update);
    }
    
    fn internal_update_hitbox(&mut self, id: HitboxId, hitbox: Option<Hitbox>, interactivity: Option<I>, phase: Phase) {
        let mut hitbox_info = self.hitboxes.remove(&id).unwrap_or_else(|| panic!("hitbox id {} not found", id));
        let mut hitbox = hitbox.unwrap_or_else(|| hitbox_info.pub_hitbox_at_time(self.time));
        hitbox.validate(self.padding);
        self.events.clear_related_events(id, &mut hitbox_info.event_keys, &mut self.hitboxes);
        
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
        
        mem::swap(&mut hitbox, &mut hitbox_info.hitbox);
        let old_hitbox = hitbox;
        self.solitaire_event_check(id, &mut hitbox_info, new_group.is_some());
        
        let empty_group_array: &[Group] = &[];
        let interact_groups: &[Group] = if new_group.is_some() { hitbox_info.interactivity.interact_groups() } else { empty_group_array };
        let test_ids = self.grid.update_hitbox(
            id, (&old_hitbox, old_group), (&hitbox_info.hitbox, new_group), interact_groups
        );
        hitbox_info.start_time = self.time;
        
        if let Some(test_ids) = test_ids {
            for other_id in test_ids {
                if !hitbox_info.overlaps.contains(&other_id) {
                    let other_hitbox_info = self.hitboxes.get_mut(&other_id).unwrap();
                    if hitbox_info.interactivity.can_interact(&other_hitbox_info.interactivity) {
                        let delay = hitbox_info.hitbox.collide_time(&other_hitbox_info.hitbox_at_time(self.time));
                        self.events.add_pair_event(self.time + delay, InternalEvent::Collide(id, other_id),
                            &mut hitbox_info.event_keys, &mut other_hitbox_info.event_keys);
                    }
                }
            }
        }
        for &other_id in hitbox_info.overlaps.clone().iter() {
            let other_hitbox_info = self.hitboxes.get_mut(&other_id).unwrap();
            let delay = hitbox_info.hitbox.separate_time(&other_hitbox_info.hitbox_at_time(self.time), self.padding);
            self.events.add_pair_event(self.time + delay, InternalEvent::Separate(id, other_id),
                &mut hitbox_info.event_keys, &mut other_hitbox_info.event_keys);
        }
        
        assert!(self.hitboxes.insert(id, hitbox_info).is_none(), "illegal state");
    }
    
    fn recheck_overlap_interactivity(&mut self, id: HitboxId, hitbox_info: &mut HitboxInfo<I>) {
        for &other_id in hitbox_info.overlaps.clone().iter() {
            let other_hitbox_info = self.hitboxes.get_mut(&other_id).unwrap();
            if !hitbox_info.interactivity.can_interact(&other_hitbox_info.interactivity) {
                assert!(hitbox_info.overlaps.remove(&other_id), "illegal state");
                assert!(other_hitbox_info.overlaps.remove(&id), "illegal state");
            }
        }
    }
    
    fn clear_overlaps(&mut self, id: HitboxId, hitbox_info: &mut HitboxInfo<I>) {
        for &other_id in hitbox_info.overlaps.iter() {
            let other_hitbox_info = self.hitboxes.get_mut(&other_id).unwrap();
            assert!(other_hitbox_info.overlaps.remove(&id), "illegal state");
        }
        hitbox_info.overlaps.clear();
    }
    
    #[cfg(debug_assertions)] 
    fn solitaire_event_check(&mut self, id: HitboxId, hitbox_info: &mut HitboxInfo<I>, has_group: bool) {
        hitbox_info.pub_duration = hitbox_info.hitbox.duration;
        let mut result = (self.grid.cell_period(&hitbox_info.hitbox, has_group), InternalEvent::Reiterate(id));
        let delay = hitbox_info.hitbox.duration;
        if delay < result.0 { result = (delay, InternalEvent::PanicDurationPassed(id)); }
        let delay = hitbox_info.hitbox.time_until_too_small(self.padding);
        if delay < result.0 { result = (delay, InternalEvent::PanicSmallHitbox(id)); }
        hitbox_info.hitbox.duration = result.0;
        self.events.add_solitaire_event(self.time + result.0, result.1, &mut hitbox_info.event_keys);
    }
    
    #[cfg(not(debug_assertions))] 
    fn solitaire_event_check(&mut self, id: HitboxId, hitbox_info: &mut HitboxInfo<I>, has_group: bool) {
        hitbox_info.pub_duration = hitbox_info.hitbox.duration;
        let result = (self.grid.cell_period(&hitbox_info.hitbox, has_group), InternalEvent::Reiterate(id));
        hitbox_info.hitbox.duration = result.0;
        self.events.add_solitaire_event(self.time + result.0, result.1, &mut hitbox_info.event_keys);
    }
}

impl <I: Interactivity> EventKeysMap for HashMap<HitboxId, HitboxInfo<I>> {
    fn event_keys_mut(&mut self, id: HitboxId) -> &mut TightSet<EventKey> {
        &mut self.get_mut(&id).unwrap().event_keys
    }
}

impl <I: Interactivity + Default> Collider<I> {
    /// Shorthand for `self.add_hitbox_with_interactivity(id, hitbox, I::default());`.
    pub fn add_hitbox(&mut self, id: HitboxId, hitbox: Hitbox) {
        self.add_hitbox_with_interactivity(id, hitbox, I::default());
    }
}

enum Phase {
    Add, Remove, Update
}

struct HitboxInfo<I: Interactivity> {
    interactivity: I,
    hitbox: Hitbox,
    start_time: N64,
    pub_duration: N64,
    event_keys: TightSet<EventKey>,
    overlaps: TightSet<HitboxId>
}

impl <I: Interactivity> HitboxInfo<I> {
    fn new(hitbox: Hitbox, interactivity: I, start_time: N64) -> HitboxInfo<I> {
        HitboxInfo {
            interactivity: interactivity,
            pub_duration: hitbox.duration,
            hitbox: hitbox,
            start_time: start_time,
            event_keys: TightSet::new(),
            overlaps: TightSet::new()
        }
    }

    fn hitbox_at_time(&self, time: N64) -> Hitbox {
        let mut result = self.hitbox.clone();
        result.advance(self.start_time, time);
        result
    }
    
    fn pub_hitbox_at_time(&self, time: N64) -> Hitbox {
        let mut result = self.hitbox.clone();
        result.duration = self.pub_duration;
        result.advance(self.start_time, time);
        result
    }
}

/// An event type that may be returned from a `Collider` instance.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Event {
    /// Occurs when two hitboxes collide
    Collide,
    
    /// Occurs when two hitboxes separate.
    ///
    /// A second `Collide` betweent two hitboxes may not occur before a `Separate`.
    /// A `Separate` event must come after a `Collide` event.
    Separate
}

fn new_event(event: Event, mut id_1: HitboxId, mut id_2: HitboxId) -> (Event, HitboxId, HitboxId) {
    assert!(id_1 != id_2, "ids must be different: {} {}", id_1, id_2);
    if id_1 > id_2 { mem::swap(&mut id_1, &mut id_2); }
    (event, id_1, id_2)
}
