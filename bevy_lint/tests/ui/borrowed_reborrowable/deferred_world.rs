//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `DeferredWorld` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::prelude::*;
use bevy::ecs::world::DeferredWorld;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &DeferredWorld) {
    // ...
}

//~| HELP: use `DeferredWorld` instead
//~v ERROR: parameter takes `&mut DeferredWorld` instead of a re-borrowed `DeferredWorld`
fn mutable_reference(_param: &mut DeferredWorld) {
    // ...
}

//~| HELP: use `DeferredWorld` instead
//~v ERROR: parameter takes `&mut DeferredWorld` instead of a re-borrowed `DeferredWorld`
fn mutable_reference_return<'a>(_param: &'a mut DeferredWorld) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut DeferredWorld) -> Commands<'a, 'a> {
    param.commands()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut DeferredWorld,
) -> Vec<(usize, Commands<'a, 'a>)> {
    vec![(1, param.commands())]
}

fn main() {
    fn some_system(mut param: DeferredWorld) {
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
