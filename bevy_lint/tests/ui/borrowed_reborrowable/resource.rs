//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `ResMut` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::prelude::*;

#[derive(Resource)]
struct Data(String);

// OK: Lint does not apply to immutable references
fn immutable_reference(_res: &ResMut<Data>) {
    // ...
}

//~| HELP: use `ResMut` instead
//~v ERROR: parameter takes `&mut ResMut` instead of a re-borrowed `ResMut`
fn mutable_reference(_res: &mut ResMut<Data>) {
    // ...
}

//~| HELP: use `ResMut` instead
//~v ERROR: parameter takes `&mut ResMut` instead of a re-borrowed `ResMut`
fn mutable_reference_return<'a>(_res: &'a mut ResMut<Data>) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(res: &'a mut ResMut<Data>) -> &'a mut String {
    &mut res.0
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    res: &'a mut ResMut<Data>,
) -> Vec<(usize, &'a mut String)> {
    vec![(1, &mut res.0)]
}

fn main() {
    fn some_system(mut res: ResMut<Data>) {
        immutable_reference(&res);
        mutable_reference(&mut res);
        _ = mutable_reference_return(&mut res);
        _ = mutable_reference_bounded_return(&mut res);
        _ = mutable_reference_bounded_return_complex(&mut res);
    }

    App::new().add_systems(Update, some_system).run();
}
