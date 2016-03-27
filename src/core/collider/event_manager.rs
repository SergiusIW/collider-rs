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

pub struct EventManager {
    events: BTreeMap<EventKey, InternalEvent>,
    next_event_index: u64
}

impl EventManager {
    pub fn new() -> EventManager {
        EventManager { events : BTreeMap::new(), next_event_index : 0 }
    }

    pub fn add_solitaire_event(&mut self, delay: f64, event: InternalEvent, key_set: &mut TightSet<EventKey>) {
        if let Some(key) = self.new_event_key(delay) {
            assert!(self.events.insert(key, event).is_none(), "illegal state");
            assert!(key_set.insert(key).is_none(), "illegal state");
        }
    }
    
    pub fn add_pair_event(&mut self, delay: f64, event: InternalEvent, first_key_set: &mut TightSet<EventKey>,
        second_key_set: &mut TightSet<EventKey>)
    {
        if let Some(key) = self.new_event_key(delay) {
            assert!(self.events.insert(key, event).is_none(), "illegal state");
            assert!(first_key_set.insert(key).is_none(), "illegal state");
            assert!(second_key_set.insert(key).is_none(), "illegal state");
        }
    }
    
    pub fn clear_related_events<I: Interactivity>(&mut self, id: HitboxId, key_set: &mut TightSet<EventKey>,
        hitbox_map: &mut HashMap<HitboxId, HitboxInfo<I>>)
    {
        for key in key_set {
            let event = self.events.remove(key).unwrap();
            assert!(hitbox_map.get_mut(event.other_id(id)).unwrap().event_keys.remove(key), "illegal state");
        }
        key_set.clear();
    }
    
    fn new_event_key(&mut self, delay: f64) -> Option<EventKey> {
        assert!(delay >= 0.0, "illegal state");
        let time = self.time + delay;
        if time == f64::INFINITY {
            None
        } else {
            let result = EventKey { time: time, index: self.next_event_index };
            self.next_event_index += 1;
            result
        }
    }
    
    pub fn peek_time(&self) -> f64 {
        self.peek_key().unwrap_or(f64::INFINITY)
    }
    
    pub fn next<I: Interactivity>(&mut self, time: f64, hitbox_map: &mut HashMap<HitboxId, HitboxInfo<I>>)
        -> Option<InternalEvent>
    {
        self.peek_key().map(|key| {
            if key.time == time {
                let event = self.remove(key).unwrap();
                for id in event.involved_hitbox_ids().iter() {
                    assert!(hitbox_map.get_mut(id).unwrap().event_keys.remove(key), "illegal state");
                }
                Some(event)
            } else {
                None
            }
        })
    }
    
    fn peek_key(&self) -> Option<EventKey> {
        self.events.iter().next()
    }
    
    //fn remove_event(&mut self, key: EventKey) {
    //    assert!(key.time() != f64::INFINITY, "illegal state");
    //    let event = self.events.remove(key).unwrap();
    //    for id in event.involved_hitbox_ids().iter() {
    //        assert!(self.hitboxes.get_mut(id).unwrap().event_keys.remove(key), "illegal state");
    //    }
    //}
}

pub struct EventKey {
    time: n64,
    index: u64
}

impl PartialOrd for EventKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
    
    fn time(&self) -> f64 {
        self.time.into()
    }
}

impl Ord for EventKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.time == other.time {
            self.index.cmp(other.index)
        } else {
            self.time.cmp(other.time)
        }
    }
}
