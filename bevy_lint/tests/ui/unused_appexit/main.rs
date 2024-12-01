#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::unused_appexit)]

use bevy::prelude::*;

fn main() {
    //~v ERROR: called `App::run()` without handling the returned `AppExit`
    App::new().run();
}