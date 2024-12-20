use std::path::PathBuf;

use args::RunSubcommands;

use crate::{
    build::ensure_web_setup,
    external_cli::{
        cargo::{self, metadata::Metadata},
        wasm_bindgen, CommandHelpers,
    },
};

pub use self::args::RunArgs;

mod args;
mod serve;

pub fn run(args: &RunArgs) -> anyhow::Result<()> {
    let cargo_args = args.cargo_args_builder();

    if let Some(RunSubcommands::Web(web_args)) = &args.subcommand {
        ensure_web_setup()?;

        let metadata = cargo::metadata::metadata_with_args(["--no-deps"])?;

        // If targeting the web, run a web server with the WASM build
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

        let port = web_args.port;
        let url = format!("http://localhost:{port}");

        // Serving the app is blocking, so we open the page first
        if web_args.open {
            match webbrowser::open(&url) {
                Ok(()) => println!("Your app is running at <{url}>!"),
                Err(error) => {
                    println!("Failed to open the browser automatically, open the app at <{url}>. (Error: {error:?}")
                }
            }
        } else {
            println!("Open your app at <{url}>!");
        }

        serve::serve(bin_target, port)?;
    } else {
        // For native builds, wrap `cargo run`
        cargo::run::command().args(cargo_args).ensure_status()?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct BinTarget {
    /// The path to the directory in `target` which contains the binary.
    pub(crate) artifact_directory: PathBuf,
    /// The name of the binary (without any extensions).
    pub(crate) bin_name: String,
}

/// Determine which binary target should be run.
///
/// The `--package` arg narrows down the search space to the given package,
/// while the `--bin` and `--example` args determine the binary target within the selected packages.
///
/// If the search couldn't be narrowed down to a single binary,
/// the `default_run` option is taken into account.
///
/// The path to the compiled binary is determined via the compilation target and profile.
pub(crate) fn select_run_binary(
    metadata: &Metadata,
    package_name: Option<&str>,
    bin_name: Option<&str>,
    example_name: Option<&str>,
    compile_target: Option<&str>,
    compile_profile: &str,
) -> anyhow::Result<BinTarget> {
    // Determine which packages the binary could be in
    let packages = if let Some(package_name) = package_name {
        let package = metadata
            .packages
            .iter()
            .find(|package| package.name == *package_name)
            .ok_or_else(|| anyhow::anyhow!("Failed to find package {package_name}"))?;
        vec![package]
    } else {
        metadata.packages.iter().collect()
    };

    let mut is_example = false;

    let target = if let Some(bin_name) = bin_name {
        // The user specified a concrete binary
        let bins: Vec<_> = packages
            .iter()
            .flat_map(|package| {
                package
                    .bin_targets()
                    .filter(|target| target.name == *bin_name)
            })
            .collect();

        if bins.is_empty() {
            anyhow::bail!("No binary with name {bin_name} available!");
        } else if bins.len() > 1 {
            anyhow::bail!("Multiple binaries with name {bin_name} available!");
        }

        bins[0]
    } else if let Some(example_name) = example_name {
        // The user specified a concrete example
        let examples: Vec<_> = packages
            .iter()
            .flat_map(|package| {
                package
                    .example_targets()
                    .filter(|target| target.name == *example_name)
            })
            .collect();

        if examples.is_empty() {
            anyhow::bail!("No example with name {example_name} available!");
        } else if examples.len() > 1 {
            anyhow::bail!("Multiple examples with name {example_name} available!");
        }

        is_example = true;
        examples[0]
    } else {
        // Nothing concrete specified, try to pick one automatically

        // If there is only one binary, pick that one
        let bins: Vec<_> = packages
            .iter()
            .flat_map(|package| package.bin_targets())
            .collect();

        if bins.is_empty() {
            anyhow::bail!("No binaries available!");
        } else if bins.len() == 1 {
            bins[0]
        } else {
            // Otherwise, check if there is a default run target defined
            let default_runs: Vec<_> = packages
                .iter()
                .filter_map(|package| package.default_run.as_ref())
                .collect();

            if default_runs.is_empty() {
                anyhow::bail!("There are multiple binaries available, try specifying one with --bin or define `default_run` in the Cargo.toml");
            } else if default_runs.len() > 1 {
                anyhow::bail!(
                    "Found multiple `default_run` definitions, I don't know which one to pick!"
                );
            } else {
                let default_run = default_runs[0];
                bins.iter()
                    .find(|bin| bin.name == *default_run)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Didn't find `default_run` binary {default_run}")
                    })?
            }
        }
    };

    // Assemble the path where the binary will be put
    let mut artifact_directory = metadata.target_directory.clone();

    if let Some(target) = compile_target {
        artifact_directory.push(target);
    }

    artifact_directory.push(compile_profile);

    if is_example {
        artifact_directory.push("examples");
    }

    Ok(BinTarget {
        bin_name: target.name.clone(),
        artifact_directory,
    })
}
