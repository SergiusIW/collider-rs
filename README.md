# collider-rs
Collider is a [Rust](https://www.rust-lang.org/) library for continuous 2D collision detection, for use with game developement.
[Continuous collision detection](https://en.wikipedia.org/wiki/Collision_detection#A_posteriori_.28discrete.29_versus_a_priori_.28continuous.29)
basically means that the time and location of the collision
are determined very precisely, as opposed to using a more traditional time-stepping and polling method.

This is a successor to a [Java library of the same name](https://github.com/SergiusIW/collider).
As my focus has shifted from Java to Rust, I ported the library.

### Documentation

Documentation for Collider can be found [here](https://docs.rs/collider/).

### Cargo

The Collider crate is hosted [here on crates.io](https://crates.io/crates/collider).
You can add a dependency in your `cargo.toml` file of a Rust project

```toml
[dependencies]
collider="*"
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
