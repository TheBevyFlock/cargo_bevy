use args::RunSubcommands;

use crate::{
    external_cli::{cargo, wasm_bindgen},
    mainfest::package_name,
    web,
};

pub(crate) use self::args::RunArgs;

mod args;

pub(crate) fn run(args: &RunArgs) -> anyhow::Result<()> {
    if args.is_web() {
        web::ensure_setup()?;
    }

    let cargo_args = args.cargo_args();

    if let Some(RunSubcommands::Web(web_args)) = &args.subcommand {
        // If targeting the web, run a web server with the WASM build
        println!("Building for WASM...");
        cargo::build().args(cargo_args).status()?;

        println!("Bundling for the web...");
        wasm_bindgen::bundle(&package_name()?, args.is_release)?;

        let port = web_args.port;
        println!("Open your app at <http://127.0.0.1:{port}>");
        web::serve(port, args.is_release)?;
    } else {
        // For native builds, wrap `cargo run`
        cargo::run().args(cargo_args).status()?;
    }

    Ok(())
}
