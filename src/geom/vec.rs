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

use std::ops::{Add, Sub, Mul, Neg, AddAssign, SubAssign, MulAssign};
use geom::card::Card;

/// A 2-D Cartesian vector using finite `f64` values.
#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub struct Vec2 {
    /// The x-coordinate.
    pub x: f64,
    /// The y-coordinate.
    pub y: f64
}

impl Vec2 {
    /// Constructs a vector with the given `x` and `y` coordinates.
    #[inline]
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }

    /// Constructs a (0, 0) vector.
    #[inline]
    pub fn zero() -> Vec2 {
        Vec2::default()
    }

    /// Computes the square of the Euclidean length of the vector.
    ///
    /// Due to underflow, this might be `0.0` even if `x` and `y` are non-zero but very small.
    pub fn len_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Computes the the Euclidean length of the vector.
    ///
    /// Due to underflow, this might be `0.0` even if `x` and `y` are non-zero but very small.
    pub fn len(&self) -> f64 {
        self.len_sq().sqrt()
    }

    /// Returns a vector in the same direction as `self` but with length (approximately) `1.0`,
    /// or `None` if `self.len() == 0.0`.
    pub fn normalize(&self) -> Option<Vec2> {
        let len = self.len();
        if len == 0.0 {
            None
        } else {
            Some(Vec2::new(self.x / len, self.y / len))
            //TODO return self if len is near 1.0? (can re-normalizing a normalized vector change its value slightly?)
        }
    }

    /// Computes the square of the Euclidean distance between two vectors.
    pub fn dist_sq(&self, other: &Vec2) -> f64 {
        (*self - *other).len_sq()
    }

    /// Computes the Euclidean distance between two vectors.
    pub fn dist(&self, other: &Vec2) -> f64 {
        (*self - *other).len()
    }

    /// Linearly interpolates between `self` and `other`.
    ///
    /// Using `ratio = 0.0` will return `self`, and using `ratio = 1.0` will return `other`.
    /// Can also extrapolate using `ratio > 1.0` or `ratio < 0.0`.
    pub fn lerp(&self, other: Vec2, ratio: f64) -> Vec2 {
        (1.0 - ratio) * *self + ratio * other
    }

    /// Rotates the vector by `angle` radians counter-clockwise (assuming +x is right and +y is up).
    pub fn rotate(&self, angle: f64) -> Vec2 {
        let sin = angle.sin();
        let cos = angle.cos();
        Vec2::new(cos * self.x - sin * self.y, sin * self.x + cos * self.y)
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self * rhs.x, self * rhs.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = f64;
    fn mul(self, rhs: Vec2) -> f64 {
        self.x*rhs.x + self.y*rhs.y
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl From<Card> for Vec2 {
    fn from(card: Card) -> Vec2 {
        match card {
            Card::MinusX => v2(-1.0, 0.0),
            Card::MinusY => v2(0.0, -1.0),
            Card::PlusX => v2(1.0, 0.0),
            Card::PlusY => v2(0.0, 1.0),
        }
    }
}

/// Shorthand for invoking the `Vec2` constructor.
#[inline]
pub fn v2(x: f64, y: f64) -> Vec2 {
    Vec2::new(x, y)
}

/// A 2-D vector that separates direction from length.
///
/// This may be used rather than `Vec2` if the length
/// may be at or near `0.0` but the direction is still important,
/// or to distinguish between a vector with a negative length
/// and a vector in the opposite direction of positive length.
/// Such distinctions are necessary when describing the
/// normal distance between `PlacedShape`s.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct DirVec2 {
    dir: Vec2,
    len: f64
}

impl DirVec2 {
    /// Constructs a vector with the given direction `dir` and length `len`.
    ///
    /// `dir` is normalized before being set.
    pub fn new(dir: Vec2, len: f64) -> DirVec2 {
        DirVec2 { dir: dir.normalize().unwrap(), len }
    }

    /// Returns the direction as a unit vector.
    #[inline]
    pub fn dir(&self) -> Vec2 {
        self.dir
    }

    /// Returns the length of the vector.  May be positive or negative.
    #[inline]
    pub fn len(&self) -> f64 {
        self.len
    }

    /// Returns a new vector with the same `len` but reversed `dir`.
    pub fn flip(&self) -> DirVec2 {
        DirVec2 { dir: -self.dir, len: self.len }
    }
}

impl From<DirVec2> for Vec2 {
    fn from(dir_vec: DirVec2) -> Vec2 {
        Vec2::new(dir_vec.dir().x * dir_vec.len(), dir_vec.dir().y * dir_vec.len())
    }
}
