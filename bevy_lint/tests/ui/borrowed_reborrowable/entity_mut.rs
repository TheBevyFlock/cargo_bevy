//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `EntityMut` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::prelude::{EntityMut, EntityRef};
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &EntityMut) {
    // ...
}

//~| HELP: use `EntityMut` instead
//~v ERROR: parameter takes `&mut EntityMut` instead of a re-borrowed `EntityMut`
fn mutable_reference(_param: &mut EntityMut) {
    // ...
}

//~| HELP: use `EntityMut` instead
//~v ERROR: parameter takes `&mut EntityMut` instead of a re-borrowed `EntityMut`
fn mutable_reference_return<'a>(_param: &'a mut EntityMut) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut EntityMut) -> EntityRef<'a> {
    param.as_readonly()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut EntityMut,
) -> Vec<(usize, EntityRef<'a>)> {
    vec![(1, param.as_readonly())]
}

fn main() {
    fn some_system(world: &mut World) {
        let mut param: EntityMut = world.spawn_empty().into();
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
