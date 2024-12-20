use crate::{
    external_cli::{cargo, rustup, wasm_bindgen, CommandHelpers},
    run::select_run_binary,
};

pub use self::args::BuildArgs;

mod args;

pub fn build(args: &BuildArgs) -> anyhow::Result<()> {
    let cargo_args = args.cargo_args_builder();

    if args.is_web() {
        ensure_web_setup()?;

        let metadata = cargo::metadata::metadata_with_args(["--no-deps"])?;

        println!("Compiling to WebAssembly...");
        cargo::build::command().args(cargo_args).ensure_status()?;

        println!("Bundling JavaScript bindings...");
        let bin_target = select_run_binary(
            &metadata,
            args.cargo_args.package_args.package.as_deref(),
            args.cargo_args.target_args.bin.as_deref(),
            args.cargo_args.target_args.example.as_deref(),
            args.target().as_deref(),
            args.profile(),
        )?;
        wasm_bindgen::bundle(&bin_target)?;
    } else {
        cargo::build::command().args(cargo_args).ensure_status()?;
    }

    Ok(())
}

pub(crate) fn ensure_web_setup() -> anyhow::Result<()> {
    // `wasm32-unknown-unknown` compilation target
    rustup::install_target_if_needed("wasm32-unknown-unknown")?;
    // `wasm-bindgen-cli` for bundling
    cargo::install::if_needed(wasm_bindgen::PROGRAM, wasm_bindgen::PACKAGE, true, false)?;

    Ok(())
}
