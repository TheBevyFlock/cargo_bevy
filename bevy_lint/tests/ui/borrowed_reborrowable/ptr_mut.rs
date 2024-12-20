//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `PtrMut` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::ptr::{Ptr, PtrMut};
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &PtrMut) {
    // ...
}

//~| HELP: use `PtrMut` instead
//~v ERROR: parameter takes `&mut PtrMut` instead of a re-borrowed `PtrMut`
fn mutable_reference(_param: &mut PtrMut) {
    // ...
}

//~| HELP: use `PtrMut` instead
//~v ERROR: parameter takes `&mut PtrMut` instead of a re-borrowed `PtrMut`
fn mutable_reference_return<'a>(_param: &'a mut PtrMut) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut PtrMut) -> Ptr<'a> {
    param.as_ref()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(param: &'a mut PtrMut) -> Vec<(usize, Ptr<'a>)> {
    vec![(1, param.as_ref())]
}

fn main() {
    fn some_system(world: &mut World) {
        let mut param: PtrMut = world.into();
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
