//! All lints offered by `bevy_lint`.
//!
//! Click on each module to learn more about individual lints. Within each module is a static that
//! documents a lint's name, group, and short description, such as
//! [`missing_reflect::MISSING_REFLECT`].

use crate::lint::BevyLint;
use rustc_lint::{Lint, LintStore};

pub mod duplicate_bevy_dependencies;
pub mod insert_event_resource;
pub mod main_return_without_appexit;
pub mod missing_reflect;
pub mod panicking_methods;
pub mod plugin_not_ending_in_plugin;
pub mod zst_query;

pub(crate) static LINTS: &[&BevyLint] = &[
    insert_event_resource::INSERT_EVENT_RESOURCE,
    main_return_without_appexit::MAIN_RETURN_WITHOUT_APPEXIT,
    panicking_methods::PANICKING_QUERY_METHODS,
    missing_reflect::MISSING_REFLECT,
    panicking_methods::PANICKING_WORLD_METHODS,
    plugin_not_ending_in_plugin::PLUGIN_NOT_ENDING_IN_PLUGIN,
    zst_query::ZST_QUERY,
    duplicate_bevy_dependencies::DUPLICATE_BEVY_DEPENDENCIES,
];

pub(crate) fn register_lints(store: &mut LintStore) {
    let lints: Vec<&Lint> = LINTS.iter().map(|x| x.lint).collect();
    store.register_lints(&lints);
}

pub(crate) fn register_passes(store: &mut LintStore) {
    store.register_late_pass(|_| Box::new(insert_event_resource::InsertEventResource));
    store.register_late_pass(|_| Box::new(main_return_without_appexit::MainReturnWithoutAppExit));
    store.register_late_pass(|_| Box::new(missing_reflect::MissingReflect));
    store.register_late_pass(|_| Box::new(panicking_methods::PanickingMethods));
    store.register_late_pass(|_| Box::new(plugin_not_ending_in_plugin::PluginNotEndingInPlugin));
    store.register_late_pass(|_| Box::new(zst_query::ZstQuery));
    store.register_late_pass(|_| Box::new(duplicate_bevy_dependencies::DuplicateBevyDependencies));
}
