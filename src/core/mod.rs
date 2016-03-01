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
use geom_ext::*;
use std::f64;

const HIGH_TIME: f64 = 1e50;

//TODO check Hitbox consistency when submitting to Collider for a change (e.g. make sure shape width/height is at least padding)

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
            vel : PlacedShape::zero(shape.kind()),
            group : Some(0),
            interactivity_change : false,
            duration : f64::INFINITY
        }
    }
    
    fn advance(&mut self, orig_time: f64, new_time: f64) {
        assert!(orig_time <= new_time, "requires orig_time <= new_time");
        let delta = new_time - orig_time;
        if delta != 0.0 {
            self.shape = self.advanced_shape(delta);
            let end_time = orig_time + self.duration;
            assert!(new_time <= end_time, "tried to advance Hitbox beyond its duration");
            self.duration = end_time - new_time;
        }
    }
    
    fn advanced_shape(&self, time: f64) -> PlacedShape {
        assert!(time <= HIGH_TIME, "requires time <= {}", HIGH_TIME);
        self.shape + self.vel*time
    }
    
    fn bounding_box(&self) -> PlacedShape {
        self.bounding_box_for(self.duration)
    }
    
    fn bounding_box_for(&self, duration: f64) -> PlacedShape {
        if self.vel.is_zero() {
            self.shape.as_rect()
        } else {
            let end_shape = self.advanced_shape(duration);
            self.shape.bounding_box(&end_shape)
        }
    }
    
    fn collide_time(&self, other: &Hitbox) -> f64 {
        solvers::collide_time(self, other)
    }
    
    fn separate_time(&self, other: &Hitbox, padding: f64) -> f64 {
        solvers::separate_time(self, other, padding)
    }
}
