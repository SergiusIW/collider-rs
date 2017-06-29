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

//! Module containing geometry primitives.

mod shape;
mod vec;

pub use self::shape::*;
pub use self::vec::*;

#[derive(PartialEq, Eq, Copy, Clone)]
pub(crate) enum Card {
    Bottom,
    Left,
    Top,
    Right
}

impl Card {
    pub fn flip(self) -> Card {
        match self {
            Card::Bottom => Card::Top,
            Card::Top => Card::Bottom,
            Card::Left => Card::Right,
            Card::Right => Card::Left
        }
    }

    pub fn vals() -> &'static [Card; 4] {
        &CARD_VALS
    }
}

static CARD_VALS: [Card; 4] = [Card::Bottom, Card::Left, Card::Top, Card::Right];

impl Into<Vec2> for Card {
    fn into(self) -> Vec2 {
        match self {
            Card::Bottom => vec2(0.0, -1.0),
            Card::Left => vec2(-1.0, 0.0),
            Card::Top => vec2(0.0, 1.0),
            Card::Right => vec2(1.0, 0.0)
        }
    }
}
