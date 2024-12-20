//! This tests the `borrowed_reborrowable` lint, specifically when triggered on closure types.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::prelude::*;

fn main() {
    let mut world = World::new();
    let mut commands = world.commands();

    //~| HELP: use `Commands` instead
    //~v ERROR: parameter takes `&mut Commands` instead of a re-borrowed `Commands`
    let closure = |commands: &mut Commands| {
        commands.spawn_empty();
    };

    closure(&mut commands);
}
