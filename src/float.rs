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

use std::f64;
use std::cmp::Ordering;

// N64 wraps a non-NaN f64 value and implements Ord.

pub fn n64(val: f64) -> N64 { N64::new(val) }

#[derive(PartialEq, PartialOrd, Copy, Clone, Default)]
pub struct N64 {
    val: f64
}

impl N64 {
    fn new(val: f64) -> N64 {
        assert!(!val.is_nan(), "unexpected NaN");
        N64 { val }
    }
}

impl Eq for N64 { }

impl Ord for N64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.val.partial_cmp(&other.val).unwrap()
    }
}
