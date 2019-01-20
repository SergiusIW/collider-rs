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

mod collider;
mod dur_hitbox;
mod events;
mod grid;

pub use self::collider::*;

use std::f64;

use self::dur_hitbox::{DurHbVel, DurHitbox};
use geom::shape::PlacedBounds;
use geom::*;

const HIGH_TIME: f64 = 1e50;

/// Type used as a handle for referencing hitboxes in a `Collider` instance.
pub type HbId = u64;

/// Velocity information describing how a hitbox shape is changing over time.
#[derive(PartialEq, Clone, Debug)]
pub struct HbVel {
    /// The movement velocity of the hitbox.
    pub value: Vec2,

    /// A velocity describing how the hitbox dims are changing over time.
    ///
    /// Since the width and height of the shape is greater than `padding` at all
    /// times, if a resize velocity is set that decreases the dimensions of the
    /// shape over time, then the user is responsible for ensuring that the
    /// shape will not decrease below this threshold. Collider may panic if this
    /// is violated.
    pub resize: Vec2,

    /// An upper-bound on the time at which the hitbox will be updated by the
    /// user.
    ///
    /// This is an advanced feature for efficiency and does not impact the
    /// results. Infinity is used as the default, but using a lower value may
    /// improve performance
    ///
    /// Collider will panic if the end time is exceeded without update, at least
    /// in unoptimized builds.  It is ultimately the user's responsibility to
    /// ensure that end times are not exceeded.
    pub end_time: f64,
}

impl HbVel {
    /// Creates an `HbVel` with the given `value`.
    #[inline]
    pub fn moving(value: Vec2) -> HbVel {
        HbVel {
            value,
            resize: Vec2::zero(),
            end_time: f64::INFINITY,
        }
    }

    /// Creates an `HbVel` with the given `value` and `end_time`.
    #[inline]
    pub fn moving_until(value: Vec2, end_time: f64) -> HbVel {
        HbVel {
            value,
            resize: Vec2::zero(),
            end_time,
        }
    }

    /// Creates a stationary `HbVel`.
    #[inline]
    pub fn still() -> HbVel {
        HbVel {
            value: Vec2::zero(),
            resize: Vec2::zero(),
            end_time: f64::INFINITY,
        }
    }

    /// Creates a stationary `HbVel` with the given `end_time`.
    #[inline]
    pub fn still_until(end_time: f64) -> HbVel {
        HbVel {
            value: Vec2::zero(),
            resize: Vec2::zero(),
            end_time,
        }
    }
}

impl From<Vec2> for HbVel {
    fn from(value: Vec2) -> HbVel {
        HbVel::moving(value)
    }
}

impl PlacedBounds for HbVel {
    fn bounds_center(&self) -> &Vec2 {
        &self.value
    }
    fn bounds_dims(&self) -> &Vec2 {
        &self.resize
    }
}

/// Represents a moving shape for continuous collision testing.
#[derive(PartialEq, Clone, Debug)]
pub struct Hitbox {
    /// The placed shape at the given point in time.
    ///
    /// The width and height of the shape must be greater than `padding` at all
    /// times.
    pub value: PlacedShape,

    /// Velocity information describing how the hitbox shape is changing over
    /// time.
    pub vel: HbVel,
}

//TODO invoke hitbox.validate() in more places so that inconsistencies are still found in optimized builds, just found later

impl Hitbox {
    /// Constructs a new hitbox with the given `value` and `vel`.
    #[inline]
    pub fn new(value: PlacedShape, vel: HbVel) -> Hitbox {
        Hitbox { value, vel }
    }

    fn advanced_shape(&self, time: f64) -> PlacedShape {
        assert!(time < HIGH_TIME, "requires time < {}", HIGH_TIME);
        self.value.advance(self.vel.value, self.vel.resize, time)
    }

    fn validate(&self, min_size: f64, present_time: f64) {
        assert!(
            !self.vel.end_time.is_nan() && self.vel.end_time >= present_time,
            "end time must exceed present time"
        );
        if self.value.kind() == ShapeKind::Circle {
            assert_eq!(
                self.vel.resize.x, self.vel.resize.y,
                "circle resize velocity must maintain aspect ratio"
            );
        }
        assert!(
            self.value.dims().x >= min_size && self.value.dims().y >= min_size,
            "shape width/height must be at least {}",
            min_size
        );
    }

    fn time_until_too_small(&self, min_size: f64) -> f64 {
        let min_size = min_size * 0.9;
        assert!(self.value.dims().x > min_size && self.value.dims().y > min_size);
        let mut time = f64::INFINITY;
        if self.vel.resize.x < 0.0 {
            time = time.min((min_size - self.value.dims().x) / self.vel.value.x);
        }
        if self.vel.resize.y < 0.0 {
            time = time.min((min_size - self.value.dims().y) / self.vel.value.y);
        }
        time
    }

    fn to_dur_hitbox(&self, time: f64) -> DurHitbox {
        assert!(time <= self.vel.end_time);
        DurHitbox {
            value: self.value,
            vel: DurHbVel {
                value: self.vel.value,
                resize: self.vel.resize,
                duration: self.vel.end_time - time,
            },
        }
    }
}

/// A group id that may be used as a first measure to efficiently filter out
/// hitboxes that don't interact.
///
/// The total number of groups used should in general be very small. Often 1 is
/// enough, and 10 is excessive. As an example, in a
/// [danmaku](https://en.wikipedia.org/wiki/Shoot_%27em_up#Bullet_hell_and_niche_appeal)
/// game (which has many bullets on screen that do not interact with each
/// other), we may use one group for bullets and one group for everything else,
/// to avoid the quadratic cost of comparing all nearby bullets with each other.
pub type HbGroup = u32;

static DEFAULT_GROUPS: [HbGroup; 1] = [0];

/// A trait that holds metadata for describing a hitbox.
///
/// A user of `Collider` will need to implement an HbProfile that best suites
/// their needs in a game. The most basic HbProfile will just contain an integer
/// ID for the hitbox, but a user may define additional metadata for identfying
/// the hitbox and describing interactivity. An HbProfile must implement the
/// `Copy` trait and should not take up much memory.
pub trait HbProfile: Copy {
    /// A unique identifier for the hitbox.
    ///
    /// Trying to have multiple hitboxes with the same `id` to a `Collider`
    /// instance simultaneously will result in a panic.
    fn id(&self) -> HbId;

    /// Returns the group id associated with the hitbox. Default is `Some(0)`.
    ///
    /// If `None` is returned, then no collisions will be reported for this
    /// hitbox at all.
    fn group(&self) -> Option<HbGroup> {
        Some(0)
    }

    /// Returns a list of groups that this hitbox can interact with. Default is
    /// `[0]`.
    ///
    /// Using large lists of groups may be inefficient.
    fn interact_groups(&self) -> &'static [HbGroup] {
        &DEFAULT_GROUPS
    }

    /// Returns true if the pair of hitboxes should be checked for collisions.
    ///
    /// This method should be commutative. This method should be consistent with
    /// `group` and `interact_groups`, although possibly more restrictive.
    fn can_interact(&self, other: &Self) -> bool;
}
