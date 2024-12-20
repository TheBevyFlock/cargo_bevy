//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `Mut` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::prelude::Mut;
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &Mut<Name>) {
    // ...
}

//~| HELP: use `Mut` instead
//~v ERROR: parameter takes `&mut Mut` instead of a re-borrowed `Mut`
fn mutable_reference(_param: &mut Mut<Name>) {
    // ...
}

//~| HELP: use `Mut` instead
//~v ERROR: parameter takes `&mut Mut` instead of a re-borrowed `Mut`
fn mutable_reference_return<'a>(_param: &'a mut Mut<Name>) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut Mut<Name>) -> &'a mut Name {
    param.as_mut()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut Mut<Name>,
) -> Vec<(usize, &'a mut Name)> {
    vec![(1, param.as_mut())]
}

fn main() {
    fn some_system(world: &mut World) {
        let mut param = world.spawn(Name::new("test")).into_mut::<Name>().unwrap();
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
