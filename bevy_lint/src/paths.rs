//! A collection of hardcoded types and functions that the linter uses.
//!
//! Since Bevy is a 3rd-party crate, we cannot easily add diagnostic items to it. In lieu of this,
//! we hardcode the paths to the items we need here, for easy referencing.
//!
//! Also see: [`match_type()`](clippy_utils::ty::match_type),
//! [`match_def_path()`](clippy_utils::match_def_path).

pub const APP: [&str; 3] = ["bevy_app", "app", "App"];
pub const COMMANDS: [&str; 4] = ["bevy_ecs", "system", "commands", "Commands"];
pub const COMPONENT: [&str; 3] = ["bevy_ecs", "component", "Component"];
pub const DEFERRED: [&str; 4] = ["bevy_ecs", "system", "system_param", "Deferred"];
pub const DEFERRED_WORLD: [&str; 4] = ["bevy_ecs", "world", "deferred_world", "DeferredWorld"];
pub const ENTITY_COMMANDS: [&str; 4] = ["bevy_ecs", "system", "commands", "EntityCommands"];
pub const ENTITY_MUT: [&str; 4] = ["bevy_ecs", "world", "entity_ref", "EntityMut"];
// Note that this moves to `bevy_ecs::event::base::Event` in 0.15.
pub const EVENT: [&str; 3] = ["bevy_ecs", "event", "Event"];
pub const EVENTS: [&str; 3] = ["bevy_ecs", "event", "Events"];
pub const FILTERED_ENTITY_MUT: [&str; 4] = ["bevy_ecs", "world", "entity_ref", "FilteredEntityMut"];
pub const MUT: [&str; 3] = ["bevy_ecs", "change_detection", "Mut"];
pub const MUT_UNTYPED: [&str; 3] = ["bevy_ecs", "change_detection", "MutUntyped"];
pub const NON_SEND_MUT: [&str; 3] = ["bevy_ecs", "change_detection", "NonSendMut"];
pub const PLUGIN: [&str; 3] = ["bevy_app", "plugin", "Plugin"];
pub const PTR_MUT: [&str; 2] = ["bevy_ptr", "PtrMut"];
pub const QUERY: [&str; 4] = ["bevy_ecs", "system", "query", "Query"];
pub const QUERY_STATE: [&str; 4] = ["bevy_ecs", "query", "state", "QueryState"];
pub const REFLECT: [&str; 3] = ["bevy_reflect", "reflect", "Reflect"];
pub const RES_MUT: [&str; 3] = ["bevy_ecs", "change_detection", "ResMut"];
pub const RESOURCE: [&str; 4] = ["bevy_ecs", "system", "system_param", "Resource"];
pub const WORLD: [&str; 3] = ["bevy_ecs", "world", "World"];
