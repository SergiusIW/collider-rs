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

use std::collections::hash_map;
use fnv::{FnvHashMap, FnvHashSet};
use std::cmp;
use float::*;
use core::{HitboxId, Hitbox};
use core::inter::Group;
use geom_ext::PlacedShapeExt;
use util::TightSet;
use index_rect::IndexRect;

//TODO add unit tests for Grid

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
struct GridKey {
    coord: (i32, i32),
    group: Group
}

#[derive(Copy, Clone)]
struct GridArea {
    rect: IndexRect,
    group: Group
}

impl GridArea {
    fn contains(&self, key: GridKey) -> bool {
        self.group == key.group && self.rect.contains(key.coord)
    }
}

pub struct Grid {
    map: FnvHashMap<GridKey, TightSet<HitboxId>>,
    cell_width: R64
}

impl Grid {
    pub fn new(cell_width: R64) -> Grid {
        Grid { map : FnvHashMap::default(), cell_width: cell_width }
    }

    pub fn cell_period(&self, hitbox: &Hitbox, has_group: bool) -> N64 {
        if has_group {
            let speed = hitbox.vel.max_edge();
            if speed <= 0.0 {
                N64::infinity()
            } else {
                N64::from(self.cell_width) / N64::from(speed)
            }
        } else {
            N64::infinity()
        }
    }
    
    pub fn update_hitbox(&mut self, hitbox_id: HitboxId, old_hitbox: (&Hitbox, Option<Group>),
                         new_hitbox: (&Hitbox, Option<Group>), groups: &[Group]) -> Option<FnvHashSet<HitboxId>>
    {
        let (old_hitbox, old_group) = old_hitbox;
        let (new_hitbox, new_group) = new_hitbox;
        
        assert!(new_group.is_some() || groups.is_empty(), "illegal state");
        let old_area = self.index_bounds(old_hitbox, old_group);
        let new_area = self.index_bounds(new_hitbox, new_group);
        self.update_area(hitbox_id, old_area, new_area);
        new_area.map(|new_area| self.overlapping_ids(hitbox_id, new_area.rect, groups))
    }
    
    fn index_bounds(&self, hitbox: &Hitbox, group: Option<Group>) -> Option<GridArea> {
        group.map(|group| {
            let bounds = hitbox.bounding_box();
            let start_x = (bounds.left() / self.cell_width).floor().raw() as i32;
            let start_y = (bounds.bottom() / self.cell_width).floor().raw() as i32;
            let end_x = cmp::max((bounds.right() / self.cell_width).ceil().raw() as i32, start_x + 1);
            let end_y = cmp::max((bounds.top() / self.cell_width).ceil().raw() as i32, start_y + 1);
            GridArea { rect : IndexRect::new((start_x, start_y), (end_x, end_y)), group : group }
        })
    }

    fn overlapping_ids(&self, hitbox_id: HitboxId, rect: IndexRect, groups: &[Group]) -> FnvHashSet<HitboxId> {
        let mut result = FnvHashSet::default();
        for &group in groups {
            for coord in rect.iter() {
                let key = GridKey { coord : coord, group : group };
                if let Some(other_ids) = self.map.get(&key) {
                    for &other_id in other_ids.iter() {
                        if other_id != hitbox_id { result.insert(other_id); }
                    }
                }
            }
        }
        result
    }

    fn update_area(&mut self, hitbox_id: HitboxId, old_area: Option<GridArea>, new_area: Option<GridArea>) {
        if let Some(old_area) = old_area {
            for coord in old_area.rect.iter() {
                let key = GridKey { coord : coord, group : old_area.group };
                if new_area.map_or(true, |new_area| !new_area.contains(key)) {
                    if let hash_map::Entry::Occupied(mut entry) = self.map.entry(key) {
                        let success = entry.get_mut().remove(&hitbox_id);
                        assert!(success, "illegal state");
                        if entry.get().is_empty() { entry.remove(); }
                    } else {
                        unreachable!();
                    }
                }
            }
        }
        if let Some(new_area) = new_area {
            for coord in new_area.rect.iter() {
                let key = GridKey { coord : coord, group : new_area.group };
                if old_area.map_or(true, |old_area| !old_area.contains(key)) {
                   let other_ids = self.map.entry(key).or_insert_with(|| TightSet::new());
                   let success = other_ids.insert(hitbox_id);
                   assert!(success, "illegal state");
                }
            }
        }
    }
}
