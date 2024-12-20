//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `Deferred` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &Deferred<CommandQueue>) {
    // ...
}

//~| HELP: use `Deferred` instead
//~v ERROR: parameter takes `&mut Deferred` instead of a re-borrowed `Deferred`
fn mutable_reference(_param: &mut Deferred<CommandQueue>) {
    // ...
}

//~| HELP: use `Deferred` instead
//~v ERROR: parameter takes `&mut Deferred` instead of a re-borrowed `Deferred`
fn mutable_reference_return<'a>(_param: &'a mut Deferred<CommandQueue>) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut Deferred<CommandQueue>) -> &'a CommandQueue {
    &*param
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut Deferred<CommandQueue>,
) -> Vec<(usize, &'a CommandQueue)> {
    vec![(1, &*param)]
}

fn main() {
    fn some_system(mut param: Deferred<CommandQueue>) {
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
