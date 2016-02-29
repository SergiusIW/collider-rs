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

use core::*;
use geom::*;
use geom_ext::*;
use util;
use std::f64;

pub const HIGH_TIME: f64 = 1e50;

pub fn rect_rect_time(a: &Hitbox, b: &Hitbox, for_collide: bool) -> f64 {
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

pub fn circle_circle_time(a: &Hitbox, b: &Hitbox, for_collide: bool) -> f64 {
    let sign = if for_collide { 1.0 } else { -1.0 };
    
    let net_rad = 0.5 * (a.shape.width() + b.shape.width());
    let dist_x = a.shape.pos.x() - b.shape.pos.x();
    let dist_y = a.shape.pos.y() - b.shape.pos.y();
    
    let coeff_c = sign * (net_rad * net_rad - dist_x * dist_x - dist_y * dist_y);
    if coeff_c > 0.0 { return 0.0; }
    
    let net_rad_vel = 0.5 * (a.vel.width() + b.vel.width());
    let dist_x_vel = a.vel.pos.x() - b.vel.pos.x();
    let dist_y_vel = a.vel.pos.y() - b.vel.pos.y();
    
    let coeff_a = sign * (net_rad_vel * net_rad_vel - dist_x_vel * dist_x_vel - dist_y_vel * dist_y_vel);
    let coeff_b = sign * 2.0 * (net_rad * net_rad_vel - dist_x * dist_x_vel - dist_y * dist_y_vel);
    
    match util::quad_root_ascending(coeff_a, coeff_b, coeff_c) {
        Some(result) if result >= 0.0 => result,
        _ => f64::INFINITY
    }
}

pub fn rect_circle_time(rect: &Hitbox, circle: &Hitbox, for_collide: bool) -> f64 {
    if for_collide {
        rect_circle_collide_time(rect, circle)
    } else {
        rect_circle_separate_time(rect, circle)
    }
}

fn rect_circle_collide_time(rect: &Hitbox, circle: &Hitbox) -> f64 {
    let base_time = rect_rect_time(rect, circle, true);
    if base_time >= rect.duration {
        f64::INFINITY
    } else {
        base_time + rebased_rect_circle_collide_time(rect, circle)
    }
}

fn rect_circle_separate_time(rect: &Hitbox, circle: &Hitbox) -> f64 {
    let base_time = rect_rect_time(rect, circle, false);
    if base_time == 0.0 { return 0.0 }
    if base_time >= HIGH_TIME { return f64::INFINITY }
    
    let mut rect = rect.clone();
    rect.duration = f64::INFINITY;
    rect.advance(0.0, base_time);
    rect.vel = -rect.vel;
    
    let mut circle = circle.clone();
    circle.duration = f64::INFINITY;
    circle.advance(0.0, base_time);
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