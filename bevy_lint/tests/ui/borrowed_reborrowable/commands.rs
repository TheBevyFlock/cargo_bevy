//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `Commands` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_commands: &Commands) {
    // ...
}

//~| HELP: use `Commands` instead
//~v ERROR: parameter takes `&mut Commands` instead of a re-borrowed `Commands`
fn mutable_reference(commands: &mut Commands) {
    commands.spawn_empty();
}

//~| HELP: use `Commands` instead
//~v ERROR: parameter takes `&mut Commands` instead of a re-borrowed `Commands`
fn mutable_reference_return<'a>(_commands: &'a mut Commands) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
    commands.spawn_empty()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    commands: &'a mut Commands,
) -> Vec<(usize, EntityCommands<'a>)> {
    vec![(1, commands.spawn_empty())]
}

fn main() {
    let mut world = World::new();
    let mut commands = world.commands();

    immutable_reference(&commands);
    mutable_reference(&mut commands);
    _ = mutable_reference_return(&mut commands);
    _ = mutable_reference_bounded_return(&mut commands);
    _ = mutable_reference_bounded_return_complex(&mut commands);
}
