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

use std::cmp::Ordering;

#[derive(PartialEq, PartialOrd, Copy, Clone, Default)]
pub struct N64 {
    val: f64
}

pub fn n64(val: f64) -> N64 {
    N64::from(val)
}

impl From<f64> for N64 {
    fn from(val: f64) -> N64 {
        assert!(!val.is_nan(), "NaN encountered");
        N64 { val : val }
    }
}

impl Into<f64> for N64 {
    fn into(self) -> f64 {
        self.val
    }
}

impl Eq for N64 { }

impl Ord for N64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.val.partial_cmp(&other.val).unwrap()
    }
}

//TODO implement PartialOrd and Ord more efficientally knowing that val cannot be NaN