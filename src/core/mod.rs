// Copyright 2016-2017 Matthew D. Michelotti
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

mod grid;
mod collider;
mod events;
mod dur_hitbox;

pub use self::collider::*;

use std::f64;

use geom::*;
use self::dur_hitbox::DurHitbox;

const HIGH_TIME: f64 = 1e50;

/// Type used as a handle for referencing hitboxes in a `Collider` instance.
pub type HitboxId = u64;

/// Represents a moving shape for continuous collision testing.
#[derive(PartialEq, Clone, Debug)]
pub struct Hitbox {
    /// The placed shape `shape` at the given point in time.
    ///
    /// The width and height of the shape must be greater than `padding` (in the `Collider` constructor)
    /// at all times.
    pub shape: PlacedShape,

    /// A velocity that describes how the shape is changing over time.
    ///
    /// The `vel` may include the velocity of the width and height of the `shape` as
    /// well as the velocity of the position.
    ///
    /// Since the width and height of the shape is greater than `padding` at all times,
    /// if a shape velocity is set that decreases the dimensions of the shape over time,
    /// then the user is responsible for ensuring that the shape will not decrease below this threshold.
    /// Collider will catch such mistakes in unoptimized builds.
    pub vel: PlacedShape,

    /// An upper-bound on the time at which the hitbox will be updated by the user.
    ///
    /// This is an advanced feature for efficiency and does not impact the results.
    /// Infinity is used as the default, but using a lower value may improve performance
    ///
    /// Collider will panic if the end time is exceeded without update,
    /// at least in unoptimized builds.  It is ultimately the user's responsibility
    /// to ensure that end times are not exceeded.
    pub end_time: f64
}

//TODO invoke hitbox.validate() in more places so that inconsistencies are still found in optimized builds, just found later

impl Hitbox {
    /// Constructs a new hitbox with the given `shape` and a `vel` of zero and `duration` of infinity.
    pub fn new(shape: PlacedShape) -> Hitbox {
        Hitbox {
            shape : shape,
            vel : PlacedShape::new(Vec2::zero(), Shape::new(shape.kind(), Vec2::zero())),
            end_time : f64::INFINITY
        }
    }

    fn advanced_shape(&self, time: f64) -> PlacedShape {
        assert!(time < HIGH_TIME, "requires time < {}", HIGH_TIME);
        self.shape + self.vel * time
    }

    fn validate(&self, min_size: f64, present_time: f64) {
        assert!(!self.end_time.is_nan() && self.end_time >= present_time, "end time must exceed present time");
        assert!(self.shape.kind() == self.vel.kind(), "shape and vel have different kinds");
        assert!(self.shape.dims().x >= min_size && self.shape.dims().y >= min_size, "shape width/height must be at least {}", min_size);
    }

    fn time_until_too_small(&self, min_size: f64) -> f64 {
        let min_size = min_size * 0.9;
        assert!(self.shape.dims().x > min_size && self.shape.dims().y > min_size);
        let mut time = f64::INFINITY;
        if self.vel.dims().x < 0.0 { time = time.min(min_size - self.shape.dims().x / self.vel.dims().x); }
        if self.vel.dims().y < 0.0 { time = time.min(min_size - self.shape.dims().y / self.vel.dims().y); }
        time
    }

    fn to_dur_hitbox(&self, time: f64) -> DurHitbox {
        assert!(time <= self.end_time);
        DurHitbox {
            shape: self.shape,
            vel: self.vel,
            duration: self.end_time - time
        }
    }
}

/// Contains types that describe interactions between hitboxes.
pub mod inter {
    /// A group id that may be used as a first measure to efficiently filter out hitboxes that don't interact.
    ///
    /// The total number of groups used should in general be very small.
    /// Often 1 is enough, and 4 is excessive.
    /// As an example, in a [danmaku](https://en.wikipedia.org/wiki/Shoot_%27em_up#Bullet_hell_and_niche_appeal) game
    /// (which has many bullets on screen that do not interact with each other),
    /// we may use one group for bullets and one group for everything else,
    /// to avoid the quadratic cost of comparing all nearby bullets with each other.
    pub type Group = u32;

    static DEFAULT_GROUPS: [Group; 1] = [0];

    /// Used to determine which pairs of hitboxes should be checked for collisions
    /// and which pairs should be ignored.
    pub trait Interactivity {
        /// Returns the group id associated with the hitbox.
        /// Default is `Some(0)`.
        ///
        /// If `None` is returned, then no collisions will be reported
        /// for this hitbox at all.
        fn group(&self) -> Option<Group> { Some(0) }

        /// Returns a list of groups that this hitbox can interact with.
        /// Using large lists of groups may be inefficient.
        /// Default is `[0]`.
        fn interact_groups(&self) -> &'static [Group] { &DEFAULT_GROUPS }

        /// Returns true if the pair of hitboxes should be checked for collisions.
        /// This method should be commutative.
        /// This method should be consistent with `group` and `interact_groups`,
        /// although possibly more restrictive.
        fn can_interact(&self, other: &Self) -> bool;
    }

    /// The default implementation of `Interactivity`, in which
    /// every hitbox is allowed to interact with every other hitbox.
    #[derive(Default)]
    pub struct DefaultInteractivity;

    impl Interactivity for DefaultInteractivity {
        fn can_interact(&self, _other: &Self) -> bool { true }
    }
}
