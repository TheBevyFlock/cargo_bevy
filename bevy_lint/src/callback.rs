use rustc_driver::Callbacks;
use rustc_interface::interface::Config;

/// The `rustc` [`Callbacks`] that register Bevy's lints.
pub struct BevyLintCallback;

impl Callbacks for BevyLintCallback {
    fn config(&mut self, config: &mut Config) {
        // Load the lint configuration. Note that this should happen before lints are registered,
        // as they may access the config when constructed.
        crate::config::load_config(config);

        // We're overwriting `register_lints`, but we don't want to completely delete the original
        // function. Instead, we save it so we can call it ourselves inside its replacement.
        let previous = config.register_lints.take();

        config.register_lints = Some(Box::new(move |session, store| {
            // If there was a previous `register_lints`, call it first.
            if let Some(previous) = &previous {
                (previous)(session, store);
            }

            crate::lints::register_lints(store);
            crate::lints::register_passes(store);
            crate::groups::register_groups(store);
        }));
    }
}
