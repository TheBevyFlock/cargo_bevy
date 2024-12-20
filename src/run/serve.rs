//! Serving the app locally for the browser.
use actix_web::{rt, web, App, HttpResponse, HttpServer, Responder};
use std::path::Path;

use super::BinTarget;

/// Serve a static HTML file with the given content.
async fn serve_static_html(content: &'static str) -> impl Responder {
    // Build the HTTP response with appropriate headers to serve the content as a file
    HttpResponse::Ok()
        .insert_header((
            actix_web::http::header::CONTENT_TYPE,
            "text/html; charset=utf-8",
        ))
        .body(content)
}

/// Create the default `index.html` if the user didn't provide one.
fn default_index(bin_target: &BinTarget) -> &'static str {
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/web/index.html"
    ));

    // Insert correct path to JS bindings
    let index = template.replace(
        "./build/bevy_app.js",
        format!("./build/{}.js", bin_target.bin_name).as_str(),
    );

    // Only static strings can be served in the web app,
    // so we leak the string memory to convert it to a static reference.
    // PERF: This is assumed to be used only once and is needed for the rest of the app running
    // time, making the memory leak acceptable.
    Box::leak(index.into_boxed_str())
}

/// Launch a web server running the Bevy app.
pub(crate) fn serve(bin_target: BinTarget, port: u16) -> anyhow::Result<()> {
    let index_html = default_index(&bin_target);

    rt::System::new().block_on(
        HttpServer::new(move || {
            let mut app = App::new();
            let bin_target = bin_target.clone();

            // Serve the build artifacts at the `/build/*` route
            // A custom `index.html` will have to call `/build/{bin_name}.js`
            app = app.service(
                actix_files::Files::new("/build", bin_target.artifact_directory.clone())
                    // This potentially includes artifacts which we will not need,
                    // but we can't add the bin name to the check due to lifetime requirements
                    .path_filter(move |path, _| {
                        path.file_stem().is_some_and(|stem| {
                            // Using `.starts_with` instead of equality, because of the `_bg` suffix
                            // of the WASM bindings
                            stem.to_string_lossy().starts_with(&bin_target.bin_name)
                        }) && (path.extension().is_some_and(|ext| ext == "js")
                            || path.extension().is_some_and(|ext| ext == "wasm"))
                    }),
            );

            // If the app has an assets folder, serve it under `/assets`
            if Path::new("assets").exists() {
                app = app.service(actix_files::Files::new("/assets", "./assets"))
            }

            if Path::new("web").exists() {
                // Serve the contents of the `web` folder under `/`, if it exists
                app = app.service(actix_files::Files::new("/", "./web").index_file("index.html"));
            } else {
                // If the user doesn't provide a custom web setup, serve a default `index.html`
                app = app.route("/", web::get().to(|| serve_static_html(index_html)))
            }

            app
        })
        .bind(("127.0.0.1", port))?
        .run(),
    )?;

    Ok(())
}
