use std::sync::Once;

use rustc_driver::Callbacks;
use rustc_interface::interface::Config;
use rustc_lint_defs::RegisteredTools;
use rustc_middle::ty::TyCtxt;
use rustc_span::symbol::Ident;

/// The original `registered_tools()` query.
///
/// Because we overwrite `registered_tools()` with our own version, we save the original in this
/// static so it can be called later.
static mut REGISTERED_TOOLS: fn(TyCtxt<'_>, ()) -> RegisteredTools = |_, _| {
    unreachable!("This function will be overwritten when `BevyLintCallback::config()` is run.")
};

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

        debug_assert!(
            config.override_queries.is_none(),
            "`override_queries()` already exists, but it would be replaced.",
        );

        config.override_queries = Some(|_session, providers| {
            static INIT: Once = Once::new();

            // SAFETY: `REGISTERED_TOOLS` is only written to here, and is not read by our custom
            // `registered_tools()` until later in the program.
            INIT.call_once(|| unsafe {
                // Save the original `registered_tools()` query so that our new version can still
                // call it.
                REGISTERED_TOOLS = providers.queries.registered_tools;
            });

            // Overwrite the `registered_tools()` query with our own version.
            providers.queries.registered_tools = registered_tools;
        });
    }
}

/// A version of the `registered_tools()` compiler query that also includes `bevy` by default.
fn registered_tools(tcx: TyCtxt<'_>, _: ()) -> RegisteredTools {
    // SAFETY: `REGISTERED_TOOLS` is not modified after it is first set in
    // `BevyLintCallback::config()`. Queries are not run until after that function finishes.
    let mut tools = unsafe { REGISTERED_TOOLS(tcx, ()) };

    tools.insert(Ident::from_str("bevy"));

    tools
}
