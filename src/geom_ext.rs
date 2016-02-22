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
use geom::*;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Card {
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
    
    pub fn vals() -> &'static [Card] {
        &CARD_VALS
    }
}

static CARD_VALS: [Card; 4] = [Card::Bottom, Card::Left, Card::Top, Card::Right];

impl Into<Vec2> for Card {
    fn into(self) -> Vec2 {
        match self {
            Card::Bottom => Vec2::new(0.0, -1.0),
            Card::Left => Vec2::new(-1.0, 0.0),
            Card::Top => Vec2::new(0.0, 1.0),
            Card::Right => Vec2::new(1.0, 0.0)
        }
    }
}

pub trait PlacedShapeExt {
    fn sector(&self, point: Vec2) -> Sector;
    fn corner(&self, sector: Sector) -> Vec2;
    fn card_overlap(&self, src: &PlacedShape, card: Card) -> f64;
}

impl PlacedShapeExt for PlacedShape {
    fn sector(&self, point: Vec2) -> Sector {
        let x = interval_sector(self.left(), self.right(), point.x());
        let y = interval_sector(self.bottom(), self.top(), point.y());
        Sector::new(x, y)
    }
    
    fn corner(&self, sector: Sector) -> Vec2 {
        let x = match sector.x {
            Ordering::Less => self.left(),
            Ordering::Greater => self.right(),
            Ordering::Equal => panic!("expected corner sector")
        };
        let y = match sector.y {
            Ordering::Less => self.bottom(),
            Ordering::Greater => self.top(),
            Ordering::Equal => panic!("expected corner sector")
        };
        Vec2::new(x, y)
    }
    
    fn card_overlap(&self, src: &PlacedShape, card: Card) -> f64 {
        edge(src, card) + edge(self, card.flip())
    }
}

fn edge(shape: &PlacedShape, card: Card) -> f64 {
    match card {
        Card::Bottom => -shape.bottom(),
        Card::Left => -shape.left(),
        Card::Top => shape.top(),
        Card::Right => shape.right()
    }
}

fn interval_sector(left: f64, right: f64, val: f64) -> Ordering {
    if val < left {
        Ordering::Less
    } else if val > right {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Sector {
    x: Ordering,
    y: Ordering
}

impl Sector {
    pub fn new(x: Ordering, y: Ordering) -> Sector {
        Sector { x : x, y : y }
    }

    pub fn is_corner(&self) -> bool {
        self.x != Ordering::Equal && self.y != Ordering::Equal
    }
}