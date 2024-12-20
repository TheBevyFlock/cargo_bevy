//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `NonSendMut` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::prelude::*;

#[derive(Resource)]
struct MyResource;

// OK: Lint does not apply to immutable references
fn immutable_reference(_param: &NonSendMut<MyResource>) {
    // ...
}

//~| HELP: use `NonSendMut` instead
//~v ERROR: parameter takes `&mut NonSendMut` instead of a re-borrowed `NonSendMut`
fn mutable_reference(_param: &mut NonSendMut<MyResource>) {
    // ...
}

//~| HELP: use `NonSendMut` instead
//~v ERROR: parameter takes `&mut NonSendMut` instead of a re-borrowed `NonSendMut`
fn mutable_reference_return<'a>(_param: &'a mut NonSendMut<MyResource>) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(param: &'a mut NonSendMut<MyResource>) -> &'a MyResource {
    &*param
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    param: &'a mut NonSendMut<MyResource>,
) -> Vec<(usize, &'a MyResource)> {
    vec![(1, &*param)]
}

fn main() {
    fn some_system(mut param: NonSendMut<MyResource>) {
        immutable_reference(&param);
        mutable_reference(&mut param);
        _ = mutable_reference_return(&mut param);
        _ = mutable_reference_bounded_return(&mut param);
        _ = mutable_reference_bounded_return_complex(&mut param);
    }

    App::new().add_systems(Update, some_system).run();
}
