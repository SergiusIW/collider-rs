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

pub fn quad_root_ascending(a: f64, b: f64, c: f64) -> Option<f64> {
    let determinant = b*b - 4.0*a*c;
    if determinant <= 0.0 {
        None
    } else if b >= 0.0 {
        Some((2.0*c)/(-b - determinant.sqrt()))
    } else {
        Some((-b + determinant.sqrt())/(2.0*a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quad_root_ascending() {
        assert!((quad_root_ascending(1e-14, 2.0, -1.0).unwrap() - 0.5).abs() < 1e-7);
        assert!((quad_root_ascending(0.0, 2.0, -1.0).unwrap() - 0.5).abs() < 1e-7);
        assert!((quad_root_ascending(100.0, -1.0, -1e-16).unwrap() - 0.01).abs() < 1e-7);
        assert!(quad_root_ascending(0.0, -2.0, 1.0).unwrap().is_infinite());
        assert!(quad_root_ascending(-3.0, 0.0, -1.0).is_none());
        assert!(quad_root_ascending(1.0, 1.0, 1.0).is_none());
    }
}