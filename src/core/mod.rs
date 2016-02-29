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

mod solvers;

use geom::*;
use std::f64;

#[derive(PartialEq, Clone, Debug)]
pub struct Hitbox {
    pub shape: PlacedShape,
    pub vel: PlacedShape,
    pub group: Option<u32>,
    pub interactivity_change: bool,
    pub duration: f64
}

impl Hitbox {
    pub fn new(shape: PlacedShape) -> Hitbox {
        Hitbox {
            shape : shape,
            vel : PlacedShape::new(Vec2::new(0.0, 0.0), Shape::new(shape.kind(), 0.0, 0.0)),
            group : Some(0),
            interactivity_change : false,
            duration : f64::INFINITY
        }
    }
    
    fn advance(&mut self, orig_time: f64, new_time: f64) {
        assert!(orig_time <= new_time, "requires orig_time <= new_time");
        let delta = new_time - orig_time;
        if delta != 0.0 {
            self.shape = self.shape + self.vel*delta;
            let end_time = orig_time + self.duration;
            assert!(new_time <= end_time, "tried to advance Hitbox beyond its duration");
            self.duration = end_time - new_time;
        }
    }
}