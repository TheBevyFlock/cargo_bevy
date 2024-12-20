//! This test tracks the bug reported in [#174]. When this starts failing, the bug has been fixed.
//!
//! [#174]: https://github.com/TheBevyFlock/bevy_cli/issues/174

//@check-pass

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::prelude::*;

fn main() {
    let mut world = World::new();

    closure_wrapper(world.commands(), |commands| commands.spawn_empty().id());
    closure_wrapper2(world.commands(), |commands| commands.spawn_empty().id());
}

fn closure_wrapper(mut commands: Commands, f: impl FnOnce(&mut Commands) -> Entity) {
    f(&mut commands);
}

fn closure_wrapper2(mut commands: Commands, f: fn(&mut Commands) -> Entity) {
    f(&mut commands);
}
