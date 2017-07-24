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

use geom::*;
use geom::shape::{PlacedBounds, Sector};
use float::n64;

// This module contains methods to solve for the normal vector
// between two PlacedShapes.

pub fn rect_rect_normal(dst: &PlacedShape, src: &PlacedShape) -> DirVec2 {
    let (card, overlap) = Card::values().iter().cloned()
        .map(|card| (card, dst.card_overlap(src, card)))
        .min_by_key(|&(_, overlap)| n64(overlap))
        .unwrap();
    DirVec2::new(card.into(), overlap)
}

pub fn circle_circle_normal(dst: &PlacedShape, src: &PlacedShape) -> DirVec2 {
    let mut dir = dst.pos - src.pos;
    let dist = dir.len();
    if dist == 0.0 { dir = v2(1.0, 0.0); }
    DirVec2::new(dir, (src.dims().x + dst.dims().x) * 0.5 - dist)
}

pub fn rect_circle_normal(dst: &PlacedShape, src: &PlacedShape) -> DirVec2 {
    let sector = dst.sector(src.pos);
    if sector.is_corner() {
        circle_circle_normal(&PlacedShape::new(dst.corner(sector), Shape::circle(0.0)), src)
    } else {
        rect_rect_normal(dst, src)
    }
}

pub fn masked_rect_rect_normal(dst: &PlacedShape, src: &PlacedShape, mask: CardMask) -> DirVec2 {
    let (card, overlap) = Card::values().iter().cloned()
        .filter(|&card| mask[card])
        .map(|card| (card, dst.card_overlap(src, card)))
        .min_by_key(|&(_, overlap)| n64(overlap))
        .unwrap_or_else(|| panic!("CardMask must be non-empty"));
    DirVec2::new(card.into(), overlap)
}

pub fn masked_circle_circle_normal(dst: &PlacedShape, src: &PlacedShape, mask: CardMask) -> DirVec2 {
    assert!(mask == CardMask::full(), "CardMask for circle-circle normal must be full");
    circle_circle_normal(dst, src)
}

pub fn masked_rect_circle_normal(dst: &PlacedShape, src: &PlacedShape, mask: CardMask) -> DirVec2 {
    let sector = dst.sector(src.pos);
    if mask_has_corner_sector(sector, mask.flip()) {
        circle_circle_normal(&PlacedShape::new(dst.corner(sector), Shape::circle(0.0)), src)
    } else {
        masked_rect_rect_normal(dst, src, mask)
    }
}

fn mask_has_corner_sector(sector: Sector, mask: CardMask) -> bool {
    if let Some((h_card, v_card)) = sector.corner_cards() {
        mask[h_card] && mask[v_card]
    } else {
        false
    }
}
