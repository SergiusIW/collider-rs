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

pub use self::float::*;

#[cfg(feature = "noisy-floats")]
mod float
{
    pub use noisy_float::prelude::*;
    pub fn n64_cmp(val: N64) -> N64 { val }
    pub fn r64_cmp(val: R64) -> R64 { val }
}

#[cfg(not(feature = "noisy-floats"))]
mod float
{
    use std::f64;
    use std::cmp::Ordering;

    pub type N64 = f64;
    pub type R64 = f64;
    pub fn n64(val: f64) -> f64 { val }
    pub fn r64(val: f64) -> f64 { val }
    
    pub trait F64Ext {
        fn infinity() -> f64;
        fn raw(self) -> f64;
    }
    
    impl F64Ext for f64 {
        fn infinity() -> f64 { f64::INFINITY }
        fn raw(self) -> f64 { self }
    }
    
    pub fn n64_cmp(val: f64) -> Ordered64 { Ordered64::new(val) }
    pub fn r64_cmp(val: f64) -> Ordered64 { Ordered64::new(val) }
    
    #[derive(PartialEq, PartialOrd, Copy, Clone, Default)]
    pub struct Ordered64 {
        val: f64
    }
    
    impl Into<f64> for Ordered64 {
        fn into(self) -> f64 {
            self.val
        }
    }
    
    impl Ordered64 {
        fn new(val: f64) -> Ordered64 {
            assert!(!val.is_nan(), "unexpected NaN");
            Ordered64 { val : val }
        }
    }
    
    impl Eq for Ordered64 { }
    
    impl Ord for Ordered64 {
        fn cmp(&self, other: &Self) -> Ordering {
            self.val.partial_cmp(&other.val).unwrap()
        }
    }
}
