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

use geom::*;
use geom_ext::*;
use util::n64;

pub fn rect_rect_normal(dst: &PlacedShape, src: &PlacedShape) -> DirVec2 {
    let (card, overlap) = Card::vals().iter()
        .map(|&card| (card, dst.card_overlap(src, card)))
        .min_by_key(|&(_, overlap)| n64(overlap))
        .unwrap();
    DirVec2::new(card.into(), overlap)
}

pub fn circle_circle_normal(dst: &PlacedShape, src: &PlacedShape) -> DirVec2 {
    let mut dir = dst.pos - src.pos;
    let dist = dir.len();
    if dist == 0.0 { dir = Vec2::new(1.0, 0.0); }
    DirVec2::new(dir, 0.5*(src.width() + dst.width()) - dist)
}

pub fn rect_circle_normal(dst: &PlacedShape, src: &PlacedShape) -> DirVec2 {
    let sector = dst.sector(src.pos);
    if sector.is_corner() {
        circle_circle_normal(&PlacedShape::new(dst.corner(sector), Shape::new_circle(0.0)), src)
    } else {
        rect_rect_normal(dst, src)
    }
}
