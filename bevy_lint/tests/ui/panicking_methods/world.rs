//! This tests the `panicking_query_methods` lint, specifically when triggered on the `World` type.

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::panicking_world_methods)]

use bevy::prelude::*;

#[derive(Component)]
struct Bob;

#[derive(Resource)]
struct Jeffrey;

// A non-send resource.
struct Patrick;

fn main() {
    let mut world = World::new();

    let bob = world.spawn(Bob).id();

    world.entity(bob);
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_entity(bob)`

    world.entity_mut(bob);
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_entity_mut(bob)`

    #[expect(deprecated, reason = "While this method is deprecated, we should still check for it while it exists.")]
    world.many_entities([bob]);
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_many_entities([bob])`

    #[expect(deprecated, reason = "While this method is deprecated, we should still check for it while it exists.")]
    world.many_entities_mut([bob]);
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_many_entities_mut([bob])`

    world.resource::<Jeffrey>();
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_resource::<Jeffrey>()`

    world.resource_mut::<Jeffrey>();
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_resource_mut::<Jeffrey>()`

    world.resource_ref::<Jeffrey>();
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_resource_ref::<Jeffrey>()`

    world.non_send_resource::<Patrick>();
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_non_send_resource::<Patrick>()`

    world.non_send_resource_mut::<Patrick>();
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.get_non_send_resource_mut::<Patrick>()`

    world.run_schedule(Update);
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.try_run_schedule(Update)`

    world.schedule_scope(Update, |_world, _schedule| {});
    //~^ ERROR: called a `World` method that can panic when a non-panicking alternative exists
    //~| HELP: use `world.try_schedule_scope(Update, |_world, _schedule| {})`
}
