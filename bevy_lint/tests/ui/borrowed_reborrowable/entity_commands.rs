//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `EntityCommands` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &EntityCommands) {
    // ...
}

//~| HELP: use `EntityCommands` instead
//~v ERROR: parameter takes `&mut EntityCommands` instead of a re-borrowed `EntityCommands`
fn mutable_reference(_param: &mut EntityCommands) {
    // ...
}

//~| HELP: use `EntityCommands` instead
//~v ERROR: parameter takes `&mut EntityCommands` instead of a re-borrowed `EntityCommands`
fn mutable_reference_return<'a>(_param: &'a mut EntityCommands) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut EntityCommands) -> Commands<'a, 'a> {
    param.commands()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut EntityCommands,
) -> Vec<(usize, Commands<'a, 'a>)> {
    vec![(1, param.commands())]
}

fn main() {
    fn some_system(mut param: Commands) {
        let mut param = param.spawn_empty();
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
