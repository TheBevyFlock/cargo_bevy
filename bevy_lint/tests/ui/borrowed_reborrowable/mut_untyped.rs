//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `MutUntyped` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::change_detection::MutUntyped;
use bevy::ecs::ptr::PtrMut;
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &MutUntyped) {
    // ...
}

//~| HELP: use `MutUntyped` instead
//~v ERROR: parameter takes `&mut MutUntyped` instead of a re-borrowed `MutUntyped`
fn mutable_reference(_param: &mut MutUntyped) {
    // ...
}

//~| HELP: use `MutUntyped` instead
//~v ERROR: parameter takes `&mut MutUntyped` instead of a re-borrowed `MutUntyped`
fn mutable_reference_return<'a>(_param: &'a mut MutUntyped) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut MutUntyped) -> PtrMut<'a> {
    param.as_mut()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut MutUntyped,
) -> Vec<(usize, PtrMut<'a>)> {
    vec![(1, param.as_mut())]
}

fn main() {
    fn some_system(world: &mut World) {
        let mut param: MutUntyped = world
            .spawn(Name::new("test"))
            .into_mut::<Name>()
            .unwrap()
            .into();
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
