//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `FilteredEntityMut` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::world::{FilteredEntityMut, FilteredEntityRef};
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &FilteredEntityMut) {
    // ...
}

//~| HELP: use `FilteredEntityMut` instead
//~v ERROR: parameter takes `&mut FilteredEntityMut` instead of a re-borrowed `FilteredEntityMut`
fn mutable_reference(_param: &mut FilteredEntityMut) {
    // ...
}

//~| HELP: use `FilteredEntityMut` instead
//~v ERROR: parameter takes `&mut FilteredEntityMut` instead of a re-borrowed `FilteredEntityMut`
fn mutable_reference_return<'a>(_param: &'a mut FilteredEntityMut) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut FilteredEntityMut) -> FilteredEntityRef<'a> {
    param.as_readonly()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut FilteredEntityMut,
) -> Vec<(usize, FilteredEntityRef<'a>)> {
    vec![(1, param.as_readonly())]
}

fn main() {
    fn some_system(world: &mut World) {
        let mut param: FilteredEntityMut = world.spawn_empty().into();
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
