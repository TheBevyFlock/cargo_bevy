use rustc_driver::Callbacks;
use rustc_interface::interface::Config;

/// The `rustc` [`Callbacks`] that register Bevy's lints.
pub struct BevyLintCallback;

impl Callbacks for BevyLintCallback {
    fn config(&mut self, config: &mut Config) {
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

        // Enable `rustc`'s internal lints when in debug mode.
        //
        // By setting `unstable_options = true`, we make `Session::enable_internal_lints()`
        // evaluate to true. This, combined with the `#[warn(rustc::internal)]` at the crate root,
        // enables `rustc`'s internal lints.
        if cfg!(debug_assertions) {
            config.opts.unstable_opts.unstable_options = true;
        }
    }
}
