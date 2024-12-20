//! This tests the `borrowed_reborrowable` lint, specifically when triggered on a `self` parameter.
//! 
//! The lint should _not_ match against a `self` parameter as it may be impossible to change the
//! method's signature (i.e. for trait methods).

//@check-pass

#![feature(register_tool)]
#![register_tool(bevy)]
#![deny(bevy::borrowed_reborrowable)]

use bevy::prelude::*;

#[allow(dead_code)]
trait MyTrait {
    fn do_thing(&mut self);
}

impl<'w, 's> MyTrait for Commands<'w, 's> {
    fn do_thing(&mut self) {}
}
