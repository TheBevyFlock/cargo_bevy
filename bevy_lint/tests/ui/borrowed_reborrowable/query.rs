//! This tests the `borrowed_reborrowable` lint, specifically when triggered on the `Query` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::ecs::system::QueryLens;
use bevy::prelude::*;

// OK: Lint does not apply to immutable references
fn immutable_reference(_query: &Query<Entity>) {
    // ...
}

//~| HELP: use `Query` instead
//~v ERROR: parameter takes `&mut Query` instead of a re-borrowed `Query`
fn mutable_reference(query: &mut Query<Entity>) {
    query.iter_mut();
}

//~| HELP: use `Query` instead
//~v ERROR: parameter takes `&mut Query` instead of a re-borrowed `Query`
fn mutable_reference_return<'a>(_query: &'a mut Query<Entity>) -> usize {
    123
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return<'a>(query: &'a mut Query<Entity>) -> QueryLens<'a, Entity> {
    query.as_query_lens()
}

// OK: Lint does not apply when return type relies on reference lifetime
fn mutable_reference_bounded_return_complex<'a>(
    query: &'a mut Query<Entity>,
) -> Vec<(usize, QueryLens<'a, Entity>)> {
    vec![(1, query.as_query_lens())]
}

fn main() {
    fn some_system(mut query: Query<Entity>) {
        immutable_reference(&query);
        mutable_reference(&mut query);
        _ = mutable_reference_return(&mut query);
        _ = mutable_reference_bounded_return(&mut query);
        _ = mutable_reference_bounded_return_complex(&mut query);
    }

    App::new().add_systems(Update, some_system).run();
}
