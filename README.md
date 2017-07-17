# collider-rs
Collider is a [Rust](https://www.rust-lang.org/) library for continuous 2D collision detection,
for use with game developement.

This is a successor to a [Java library of the same name](https://github.com/SergiusIW/collider).
As my focus has shifted from Java to Rust, I ported the library.

### Crate

The Rust crate for this library can be found [here](https://crates.io/crates/collider).

### Documentation

Documentation for Collider can be found [here](http://www.matthewmichelotti.com/projects/collider/rustdoc/collider/).

### Description

Most game engines follow the approach of periodically updating the
positions of all shapes and checking for collisions at a frozen snapshot in time.
[Continuous collision detection](https://en.wikipedia.org/wiki/Collision_detection#A_posteriori_.28discrete.29_versus_a_priori_.28continuous.29),
on the other hand, means that the time of collision is determined very precisely,
and the user is not restricted to a fixed time-stepping method.
There are currently two kinds of shapes supported by Collider: circles and rectangles.
The user specifies the positions and velocites of these shapes, which
they can update at any time, and Collider will solve for the precise times of
collision and separation.

There are certain advantages that continuous collision detection
holds over the traditional approach.
In a game engine, the position of a sprite may be updated to overlap a wall,
and in a traditional collision system there would need to be a post-correction
to make sure the sprite does not appear inside of the wall.
This is not needed with continuous collision detection, since
the precise time and location at which the sprite touches the wall is known.
Traditional collision detection may have an issue with "tunneling," in which a
fast small object runs into a narrow wall and collision detection misses it,
or two fast small objects fly right through each other and collision detection misses it.
This is also not a problem for contiuous collision detection.
It is also debatable that continuous collision detection may be
more efficient in certain circumstances,
since the hitboxes may be updated less frequently and still maintain a
smooth appearance over time.

### Example
```rust
use collider::{Collider, Event};
use collider::geom::{Shape, v2};

let mut collider: Collider = Collider::new(4.0, 0.01);

collider.add_hitbox(0, Shape::square(2.0).place(v2(-10.0, 0.0)).moving(v2(1.0, 0.0)));
collider.add_hitbox(1, Shape::square(2.0).place(v2(10.0, 0.0)).moving(v2(-1.0, 0.0)));

while collider.time() < 20.0 {
    let time = collider.next_time().min(20.0);
    collider.set_time(time);
    if let Some((event, id1, id2)) = collider.next() {
        println!("{:?} between hitbox {} and hitbox {} at time {}.",
                 event, id1, id2, collider.time());
        if event == Event::Collide {
            println!("Speed of collided hitboxes is halved.");
            for id in [id1, id2].iter().cloned() {
                let mut hitbox = collider.get_hitbox(id);
                hitbox.vel.value *= 0.5;
                collider.update_hitbox(id, hitbox);
            }
        }
    }
}

// the above loop prints the following events:
//   Collide between hitbox 0 and hitbox 1 at time 9.
//   Speed of collided hitboxes is halved.
//   Separate between hitbox 0 and hitbox 1 at time 13.01.
```

### Homepage

The homepage for Collider is on my personal website: http://www.matthewmichelotti.com/projects/collider/.

### License

Collider is licensed under the [Apache 2.0
License](http://www.apache.org/licenses/LICENSE-2.0.html).

### Looking forward

(Note: this section is intended for people who have already familiarized themselves with the library.)

There are a few new features that may be added in the more distant future, or if I receive high demand
* Extending the functionality of `PlacedShape.normal_from` so that the user
  may restrict which edges of a shape may induce a normal vector
  (e.g. for a platform in a game that may be jumped through from below but landed on from above,
  or for use in a wall made up of several rectangles lined up in a grid so that only normals
  that point away from the wall are generated).
* Adding right-triangles to the set of possible shapes.
  (Note: I do not intend to add general polygons)

The only breaking change I foresee in the future is possibly changing how `HitboxId`s work;
they are currently an integer used as a handle, but that may change.
`Interactivity` may also change with this.
