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

use core::{Hitbox, self};
use geom::*;
use geom_ext::*;
use util;
use std::f64;

pub fn collide_time(a: &Hitbox, b: &Hitbox) -> f64 {
    let duration = a.duration.min(b.duration);
    if a.bounding_box_for(duration).overlaps(&b.bounding_box_for(duration)) {
        time_unpadded(a, b, true, duration)
    } else {
        f64::INFINITY
    }
}

pub fn separate_time(a: &Hitbox, b: &Hitbox, padding: f64) -> f64 {
    let (a, b) = match (a.shape.kind(), b.shape.kind()) {
        (ShapeKind::Rect, ShapeKind::Circle) => (b, a),
        _ => (a, b)
    };
    let mut a = a.clone();
    a.shape.shape = Shape::new(a.shape.kind(), a.shape.width() + 2.0*padding, a.shape.height() + 2.0*padding);
    time_unpadded(&a, b, false, a.duration.min(b.duration))
}

fn time_unpadded(a: &Hitbox, b: &Hitbox, for_collide: bool, duration: f64) -> f64 {
    let result = match (a.shape.kind(), b.shape.kind()) {
        (ShapeKind::Rect, ShapeKind::Rect) => rect_rect_time(a, b, for_collide),
        (ShapeKind::Circle, ShapeKind::Circle) => circle_circle_time(a, b, for_collide),
        (ShapeKind::Rect, ShapeKind::Circle) => rect_circle_time(a, b, for_collide, duration),
        (ShapeKind::Circle, ShapeKind::Rect) => rect_circle_time(b, a, for_collide, duration)
    };
    if result >= duration { f64::INFINITY } else { result }
}

fn rect_rect_time(a: &Hitbox, b: &Hitbox, for_collide: bool) -> f64 {
    let mut overlap_start = 0.0f64;
    let mut overlap_end = f64::INFINITY;
    for &card in Card::vals() {
        let overlap = a.shape.card_overlap(&b.shape, card);
        let overlap_vel = a.vel.card_overlap(&b.vel, card);
        if overlap < 0.0 {
            if !for_collide {
                return 0.0;
            } else if overlap_vel <= 0.0 {
                return f64::INFINITY;
            } else {
                overlap_start = overlap_start.max(-overlap/overlap_vel);
            }
        } else if overlap_vel < 0.0 {
            overlap_end = overlap_end.min(-overlap/overlap_vel);
        }
        if overlap_start >= overlap_end {
            return if for_collide { f64::INFINITY } else { 0.0 };
        }
    }
    if for_collide { overlap_start } else { overlap_end }
}

fn circle_circle_time(a: &Hitbox, b: &Hitbox, for_collide: bool) -> f64 {
    let sign = if for_collide { 1.0 } else { -1.0 };
    
    let net_rad = 0.5 * (a.shape.width() + b.shape.width());
    let dist = a.shape.pos - b.shape.pos;
    
    let coeff_c = sign * (net_rad * net_rad - dist.len_sq());
    if coeff_c > 0.0 { return 0.0; }
    
    let net_rad_vel = 0.5 * (a.vel.width() + b.vel.width());
    let dist_vel = a.vel.pos - b.vel.pos;
    
    let coeff_a = sign * (net_rad_vel * net_rad_vel - dist_vel.len_sq());
    let coeff_b = sign * 2.0 * (net_rad * net_rad_vel - dist * dist_vel);
    
    match util::quad_root_ascending(coeff_a, coeff_b, coeff_c) {
        Some(result) if result >= 0.0 => result,
        _ => f64::INFINITY
    }
}

fn rect_circle_time(rect: &Hitbox, circle: &Hitbox, for_collide: bool, duration: f64) -> f64 {
    if for_collide {
        rect_circle_collide_time(rect, circle, duration)
    } else {
        rect_circle_separate_time(rect, circle)
    }
}

fn rect_circle_collide_time(rect: &Hitbox, circle: &Hitbox, duration: f64) -> f64 {
    let base_time = rect_rect_time(rect, circle, true);
    if base_time >= duration {
        f64::INFINITY
    } else {
        base_time + rebased_rect_circle_collide_time(rect, circle)
    }
}

fn rect_circle_separate_time(rect: &Hitbox, circle: &Hitbox) -> f64 {
    let base_time = rect_rect_time(rect, circle, false);
    if base_time == 0.0 { return 0.0 }
    if base_time >= core::HIGH_TIME { return f64::INFINITY }
    
    let mut rect = rect.clone();
    rect.shape = rect.advanced_shape(base_time);
    rect.vel = -rect.vel;
    
    let mut circle = circle.clone();
    circle.shape = circle.advanced_shape(base_time);
    circle.vel = -circle.vel;
    
    (base_time - rebased_rect_circle_collide_time(&rect, &circle)).max(0.0)
}

fn rebased_rect_circle_collide_time(rect: &Hitbox, circle: &Hitbox) -> f64 {
    let sector = rect.shape.sector(circle.shape.pos);
    if sector.is_corner() {
        let mut corner = Hitbox::new(PlacedShape::new(rect.shape.corner(sector), Shape::new_circle(0.0)));
        corner.vel.pos = rect.vel.corner(sector);
        circle_circle_time(&corner, circle, true)
    } else {
        0.0
    }
}