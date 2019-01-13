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

use core::dur_hitbox::DurHitbox;
use core::events::{EventKey, EventKeysMap, EventManager, InternalEvent};
use core::grid::Grid;
use core::{HbGroup, HbId, HbProfile, HbVel, Hitbox, HIGH_TIME};
use fnv::FnvHashMap;
use geom::PlacedShape;
use std::mem;
use util::TightSet;

// TODO check that floating point values are within a good range when adding/updating hitboxes

/// A structure that tracks hitboxes and returns collide/separate events.
///
/// Collider manages events using a "simulation time" that the user updates as
/// necessary. This time starts at `0.0`.
pub struct Collider<P: HbProfile> {
    hitboxes: FnvHashMap<HbId, HitboxInfo<P>>,
    time: f64,
    grid: Grid,
    padding: f64,
    events: EventManager,
}

impl<P: HbProfile> Collider<P> {
    /// # Constructs a new `Collider` instance.
    ///
    /// To reduce the number of overlaps that are tested, hitboxes are placed in
    /// a sparse grid structure behind the scenes. `cell_width` is the width of
    /// the cells used in that grid. If your game has a similar grid concept,
    /// then it is usually a good choice to use the same cell width as that
    /// grid. Otherwise, a good choice is to use a width that is slightly larger
    /// than most of the hitboxes.
    ///
    /// Collider generates both `Collide` and `Separate` events. However, due to
    /// numerical error, it is important that two hitboxes be a certain small
    /// distance apart from each other after a collision before they are
    /// considered separated. Otherwise, false separation events may occur if,
    /// for example, a sprite runs into a wall and stops, still touching the
    /// wall. `padding` is used to describe what this minimum separation
    /// distance is. This should typically be something that is not visible to
    /// the user, perhaps a fraction of a "pixel."
    ///
    /// Another restriction introduced by `padding` is that hitboxes are not
    /// allowed to have a width or height smaller than `padding`.
    pub fn new(cell_width: f64, padding: f64) -> Collider<P> {
        assert!(cell_width > padding, "requires cell_width > padding");
        assert!(padding > 0.0, "requires padding > 0.0");
        Collider {
            hitboxes: FnvHashMap::default(),
            time: 0.0,
            grid: Grid::new(cell_width),
            padding,
            events: EventManager::new(),
        }
    }

    /// Returns the current simulation time.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Returns the time at which `self.next()` needs to be called again.
    ///
    /// Even if `self.next_time() == self.time()`, there is a chance that
    /// calling `self.next()` will return `None`, having processed an internal
    /// event. Regardless, after `self.next()` has been called repeatedly until
    /// it returns `None`, then `self.next_time()` will be greater than
    /// `self.time()` again.
    ///
    /// This is a fast constant-time operation.  The result may be infinity.
    pub fn next_time(&self) -> f64 {
        self.events.peek_time()
    }

    /// Advances the simulation time to the given value.
    ///
    /// The positions of all hitboxes will be updated based on the velocities of
    /// the hitboxes. Will panic if `time` exceeds `self.next_time()`. Will also
    /// panic if `time` is less than `self.time()` (i.e. cannot rewind time).
    ///
    /// The hitboxes are updated implicitly, and this is actually a
    /// fast constant-time operation.
    pub fn set_time(&mut self, time: f64) {
        assert!(time >= self.time, "cannot rewind time");
        assert!(time <= self.next_time(), "time must not exceed next_time()");
        assert!(time < HIGH_TIME, "time must not exceed {}", HIGH_TIME);
        self.time = time;
    }

    /// Processes and returns the next `Collide` or `Separate` event, or returns
    /// `None` if there are no more events that occured at the given time
    /// (although an internal event might have been processed if `None` is
    /// returned). Will always return `None` if `self.next_time() >
    /// self.time()`.
    ///
    /// The returned value is a tuple, denoting the type of event (`Collide` or
    /// `Separate`) and the two hitbox profiles involved, in increasing order by
    /// `HbId`.
    pub fn next(&mut self) -> Option<(HbEvent, P, P)> {
        while let Some(event) = self.events.next(self.time, &mut self.hitboxes) {
            if let Some((event, id_1, id_2)) = self.process_event(event) {
                return Some((
                    event,
                    self.hitboxes[&id_1].profile,
                    self.hitboxes[&id_2].profile,
                ));
            }
        }
        None
    }

    fn process_event(&mut self, event: InternalEvent) -> Option<(HbEvent, HbId, HbId)> {
        match event {
            InternalEvent::Collide(id_1, id_2) => {
                let mut hitbox_info_1 = self.hitboxes.remove(&id_1).unwrap();
                {
                    let hitbox_info_2 = self.hitboxes.get_mut(&id_2).unwrap();
                    Collider::process_collision(
                        id_1,
                        &mut hitbox_info_1,
                        id_2,
                        hitbox_info_2,
                        &mut self.events,
                        self.time,
                        self.padding,
                    );
                }
                assert!(self.hitboxes.insert(id_1, hitbox_info_1).is_none());
                Some(new_event(HbEvent::Collide, id_1, id_2))
            }
            InternalEvent::Separate(id_1, id_2) => {
                let mut hitbox_info_1 = self.hitboxes.remove(&id_1).unwrap();
                {
                    let hitbox_info_2 = self.hitboxes.get_mut(&id_2).unwrap();
                    assert!(hitbox_info_1.overlaps.remove(&id_2));
                    assert!(hitbox_info_2.overlaps.remove(&id_1));
                    let delay = hitbox_info_1
                        .hitbox_at_time(self.time)
                        .collide_time(&hitbox_info_2.hitbox_at_time(self.time));
                    self.events.add_pair_event(
                        self.time + delay,
                        InternalEvent::Collide(id_1, id_2),
                        &mut hitbox_info_1.event_keys,
                        &mut hitbox_info_2.event_keys,
                    );
                }
                assert!(self.hitboxes.insert(id_1, hitbox_info_1).is_none());
                Some(new_event(HbEvent::Separate, id_1, id_2))
            }
            InternalEvent::Reiterate(id) => {
                self.internal_update_hitbox(id, None);
                None
            }
            #[cfg(debug_assertions)]
            InternalEvent::PanicSmallHitbox(id) => panic!("hitbox {} became too small", id),
            #[cfg(debug_assertions)]
            InternalEvent::PanicDurationPassed(id) => {
                panic!("hitbox {} was not updated before duration passed", id)
            }
        }
    }

    fn process_collision(
        id_1: HbId,
        hb_1: &mut HitboxInfo<P>,
        id_2: HbId,
        hb_2: &mut HitboxInfo<P>,
        events: &mut EventManager,
        time: f64,
        padding: f64,
    ) {
        assert!(hb_1.overlaps.insert(id_2));
        assert!(hb_2.overlaps.insert(id_1));
        let delay = hb_1
            .hitbox_at_time(time)
            .separate_time(&hb_2.hitbox_at_time(time), padding);
        events.add_pair_event(
            time + delay,
            InternalEvent::Separate(id_1, id_2),
            &mut hb_1.event_keys,
            &mut hb_2.event_keys,
        );
    }

    /// Returns the current state of the hitbox with the given `id`.
    pub fn get_hitbox(&self, id: HbId) -> Hitbox {
        self.hitboxes[&id].pub_hitbox_at_time(self.time)
    }

    /// # Adds a new hitbox to the collider.
    ///
    /// The `profile` is used to track the hitbox over time; Collider will
    /// return this profile in certain methods, and the ID in this profile can
    /// be used to make updates to the hitbox. This method will panic if there
    /// is an ID clash. `hitbox` is the initial state of the hitbox.
    ///
    /// Returns a vector of all hitbox profiles that this new hitbox collided
    /// with as it was added. Note that separate collision events will not be
    /// generated for these collisions.
    pub fn add_hitbox(&mut self, profile: P, hitbox: Hitbox) -> Vec<P> {
        hitbox.validate(self.padding, self.time);
        let id = profile.id();
        let has_group = profile.group().is_some();
        let mut info = HitboxInfo::new(hitbox, profile, self.time);
        self.solitaire_event_check(id, &mut info, has_group);
        let dur_hitbox = info.hitbox.to_dur_hitbox(self.time);
        self.update_hitbox_tracking(id, info, None, dur_hitbox)
    }

    /// Updates the velocity information of the hitbox with the given `id`.
    pub fn set_hitbox_vel(&mut self, id: HbId, vel: HbVel) {
        if self.hitboxes[&id].hitbox.vel != vel {
            self.internal_update_hitbox(id, Some(vel));
        }
    }

    fn internal_update_hitbox(&mut self, id: HbId, vel: Option<HbVel>) {
        let mut info = self
            .hitboxes
            .remove(&id)
            .unwrap_or_else(|| panic!("hitbox id {} not found", id));
        let old_hitbox = info.hitbox.to_dur_hitbox(info.start_time);
        info.hitbox = info.pub_hitbox_at_time(self.time);
        if let Some(vel) = vel {
            info.hitbox.vel = vel;
            info.hitbox.validate(self.padding, self.time);
        }
        info.start_time = self.time;
        let has_group = info.profile.group().is_some();
        self.events
            .clear_related_events(id, &mut info.event_keys, &mut self.hitboxes);
        self.solitaire_event_check(id, &mut info, has_group);
        let new_hitbox = info.hitbox.to_dur_hitbox(self.time);
        let result = self.update_hitbox_tracking(id, info, Some(old_hitbox), new_hitbox);
        assert!(result.is_empty());
    }

    /// Removes the hitbox with the given `id` from all tracking.
    ///
    /// Returns a vector of all hitbox profiles that this hitbox separated from
    /// as it was removed. No further events will be generated for this hitbox.
    pub fn remove_hitbox(&mut self, id: HbId) -> Vec<P> {
        let mut info = self
            .hitboxes
            .remove(&id)
            .unwrap_or_else(|| panic!("hitbox id {} not found", id));
        self.events
            .clear_related_events(id, &mut info.event_keys, &mut self.hitboxes);
        if let Some(group) = info.profile.group() {
            let info_start_time = info.start_time;
            let empty_group_array: &[HbGroup] = &[];
            self.grid.update_hitbox(
                id,
                group,
                Some(&info.hitbox.to_dur_hitbox(info_start_time)),
                None,
                empty_group_array,
            );
        }
        self.clear_overlaps(id, &mut info)
    }

    /// Returns the profiles of all currently tracked overlaps on the hitbox
    /// with the given `id`.
    pub fn get_overlaps(&self, id: HbId) -> Vec<P> {
        let info = self
            .hitboxes
            .get(&id)
            .unwrap_or_else(|| panic!("hitbox id {} not found", id));
        info.overlaps
            .iter()
            .map(|other_id| self.hitboxes[other_id].profile)
            .collect()
    }

    /// Returns true if there is a currently tracked overlap between the
    /// hitboxes with `id_1` and `id_2`.
    pub fn is_overlapping(&self, id_1: HbId, id_2: HbId) -> bool {
        self.hitboxes
            .get(&id_1)
            .map(|info| info.overlaps.contains(&id_2))
            .unwrap_or(false)
    }

    /// Returns the profiles of all hitboxes that overlap the given `shape` and
    /// interact with the given `profile`.
    pub fn query_overlaps(&self, shape: &PlacedShape, profile: &P) -> Vec<P> {
        self.grid
            .shape_cellmates(shape, profile.interact_groups())
            .iter()
            .map(|id| &self.hitboxes[id])
            .filter(|info| info.profile.can_interact(profile))
            .filter(|info| info.pub_hitbox_at_time(self.time).value.overlaps(shape))
            .map(|info| info.profile)
            .collect()
    }

    fn update_hitbox_tracking(
        &mut self,
        id: HbId,
        mut info: HitboxInfo<P>,
        old_hitbox: Option<DurHitbox>,
        new_hitbox: DurHitbox,
    ) -> Vec<P> {
        let mut result = Vec::new();
        if let Some(group) = info.profile.group() {
            for &other_id in info.overlaps.clone().iter() {
                let other_info = self.hitboxes.get_mut(&other_id).unwrap();
                let delay =
                    new_hitbox.separate_time(&other_info.hitbox_at_time(self.time), self.padding);
                self.events.add_pair_event(
                    self.time + delay,
                    InternalEvent::Separate(id, other_id),
                    &mut info.event_keys,
                    &mut other_info.event_keys,
                );
            }
            let test_ids = self
                .grid
                .update_hitbox(
                    id,
                    group,
                    old_hitbox.as_ref(),
                    Some(&new_hitbox),
                    info.profile.interact_groups(),
                )
                .unwrap();
            for other_id in test_ids {
                if old_hitbox.is_none() || !info.overlaps.contains(&other_id) {
                    let other_info = self.hitboxes.get_mut(&other_id).unwrap();
                    if info.profile.can_interact(&other_info.profile) {
                        let delay = new_hitbox.collide_time(&other_info.hitbox_at_time(self.time));
                        if old_hitbox.is_none() && delay == 0.0 {
                            result.push(other_info.profile);
                            Collider::process_collision(
                                id,
                                &mut info,
                                other_id,
                                other_info,
                                &mut self.events,
                                self.time,
                                self.padding,
                            );
                        } else {
                            self.events.add_pair_event(
                                self.time + delay,
                                InternalEvent::Collide(id, other_id),
                                &mut info.event_keys,
                                &mut other_info.event_keys,
                            );
                        }
                    }
                }
            }
        }

        assert!(self.hitboxes.insert(id, info).is_none());
        result
    }

    fn clear_overlaps(&mut self, id: HbId, hitbox_info: &mut HitboxInfo<P>) -> Vec<P> {
        hitbox_info
            .overlaps
            .drain()
            .map(|other_id| {
                let other_hitbox_info = self.hitboxes.get_mut(&other_id).unwrap();
                assert!(other_hitbox_info.overlaps.remove(&id));
                other_hitbox_info.profile
            })
            .collect()
    }

    #[cfg(debug_assertions)]
    fn solitaire_event_check(
        &mut self,
        id: HbId,
        hitbox_info: &mut HitboxInfo<P>,
        has_group: bool,
    ) {
        hitbox_info.pub_end_time = hitbox_info.hitbox.vel.end_time;
        let mut result = (
            self.time + self.grid.cell_period(&hitbox_info.hitbox, has_group),
            InternalEvent::Reiterate(id),
        );
        let end_time = hitbox_info.hitbox.vel.end_time;
        if end_time < result.0 {
            result = (end_time, InternalEvent::PanicDurationPassed(id));
        }
        let end_time = self.time + hitbox_info.hitbox.time_until_too_small(self.padding);
        if end_time < result.0 {
            result = (end_time, InternalEvent::PanicSmallHitbox(id));
        }
        hitbox_info.hitbox.vel.end_time = result.0;
        self.events
            .add_solitaire_event(result.0, result.1, &mut hitbox_info.event_keys);
    }

    #[cfg(not(debug_assertions))]
    fn solitaire_event_check(
        &mut self,
        id: HbId,
        hitbox_info: &mut HitboxInfo<P>,
        has_group: bool,
    ) {
        hitbox_info.pub_end_time = hitbox_info.hitbox.vel.end_time;
        let mut result = (
            self.time + self.grid.cell_period(&hitbox_info.hitbox, has_group),
            true,
        );
        let end_time = hitbox_info.hitbox.vel.end_time;
        if end_time < result.0 {
            result = (end_time, false);
        }
        let end_time = self.time + hitbox_info.hitbox.time_until_too_small(self.padding);
        if end_time < result.0 {
            result = (end_time, false);
        }
        hitbox_info.hitbox.vel.end_time = result.0;
        if result.1 {
            self.events.add_solitaire_event(
                result.0,
                InternalEvent::Reiterate(id),
                &mut hitbox_info.event_keys,
            );
        }
    }
}

impl<P: HbProfile> EventKeysMap for FnvHashMap<HbId, HitboxInfo<P>> {
    fn event_keys_mut(&mut self, id: HbId) -> &mut TightSet<EventKey> {
        &mut self.get_mut(&id).unwrap().event_keys
    }
}

struct HitboxInfo<P: HbProfile> {
    profile: P,
    hitbox: Hitbox,
    start_time: f64,
    pub_end_time: f64,
    event_keys: TightSet<EventKey>,
    overlaps: TightSet<HbId>,
}

impl<P: HbProfile> HitboxInfo<P> {
    fn new(hitbox: Hitbox, profile: P, start_time: f64) -> HitboxInfo<P> {
        HitboxInfo {
            profile,
            pub_end_time: hitbox.vel.end_time,
            hitbox,
            start_time,
            event_keys: TightSet::new(),
            overlaps: TightSet::new(),
        }
    }

    fn hitbox_at_time(&self, time: f64) -> DurHitbox {
        assert!(
            time >= self.start_time && time <= self.hitbox.vel.end_time,
            "invalid time"
        );
        let mut result = self.hitbox.clone();
        result.value = result.advanced_shape(time - self.start_time);
        result.to_dur_hitbox(time)
    }

    fn pub_hitbox_at_time(&self, time: f64) -> Hitbox {
        assert!(
            time >= self.start_time && time <= self.pub_end_time,
            "invalid time"
        );
        let mut result = self.hitbox.clone();
        result.vel.end_time = self.pub_end_time;
        result.value = result.advanced_shape(time - self.start_time);
        result
    }
}

/// A hitbox event type that may be returned from a `Collider` instance.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum HbEvent {
    /// Occurs when two hitboxes collide
    Collide,

    /// Occurs when two hitboxes separate.
    ///
    /// A second `Collide` between two hitboxes may not occur before a
    /// `Separate`. A `Separate` event must come after a `Collide` event.
    Separate,
}

fn new_event(event: HbEvent, mut id_1: HbId, mut id_2: HbId) -> (HbEvent, HbId, HbId) {
    assert!(id_1 != id_2, "ids must be different: {} {}", id_1, id_2);
    if id_1 > id_2 {
        mem::swap(&mut id_1, &mut id_2);
    }
    (event, id_1, id_2)
}
