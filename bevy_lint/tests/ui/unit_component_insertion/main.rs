#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::unit_component_insertion)]
#![allow(unused_variables)]

use bevy::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new().add_systems(Startup, test);
}

fn test(mut commands: Commands) {
    commands.spawn((
        Name::new("Decal"),
        Transform::from_translation(Vec3::new(0.75, 0.0, 0.0)).rotate_z(PI / 4.0),
        //~^ ERROR: Expression returns `unit` and results in an empty component insertion
        example()
        //~^ ERROR: Expression returns `unit` and results in an empty component insertion
    ));
}

fn example() {
    let foo = 1;
}
