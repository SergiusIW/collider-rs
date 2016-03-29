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

use std::ops::{Add, Sub, Mul, Neg};

#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub struct Vec2 {
    x: f64,
    y: f64
}

impl Vec2 {
    #[inline]
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x : x, y : y }
    }
    
    #[inline]
    pub fn zero() -> Vec2 {
        Vec2::default()
    }
    
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }
    
    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }
    
    pub fn len_sq(&self) -> f64 {
        self.x()*self.x() + self.y()*self.y()
    }
    
    pub fn len(&self) -> f64 {
        self.len_sq().sqrt()
    }
    
    pub fn normalize(&self) -> Vec2 {
        let len = self.len();
        assert!(len > 0.0, "can only normalize vector if length is non-zero");
        //TODO return self if len is near 1.0? (can re-normalizing a normalized vector change its value slightly?)
        Vec2::new(self.x()/len, self.y()/len)
    }
    
    pub fn dist_sq(&self, other: &Vec2) -> f64 {
        (*self - *other).len_sq()
    }
    
    pub fn dist(&self, other: &Vec2) -> f64 {
        (*self - *other).len()
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x()*rhs, self.y()*rhs)
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = f64;
    fn mul(self, rhs: Vec2) -> f64 {
        self.x()*rhs.x() + self.y()*rhs.y()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x(), -self.y())
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct DirVec2 {
    dir: Vec2,
    len: f64
}

impl DirVec2 {
    pub fn new(dir: Vec2, len: f64) -> DirVec2 {
        DirVec2 { dir : dir.normalize(), len : len }
    }
    
    #[inline]
    pub fn dir(&self) -> Vec2 {
        self.dir
    }
    
    #[inline]
    pub fn len(&self) -> f64 {
        self.len
    }
    
    pub fn flip(&self) -> DirVec2 {
        DirVec2 { dir : -self.dir, len : self.len }
    }
}

impl Into<Vec2> for DirVec2 {
    fn into(self) -> Vec2 {
        Vec2::new(self.dir().x()*self.len(), self.dir().y()*self.len())
    }
}