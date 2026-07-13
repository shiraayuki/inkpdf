//! Subprocess entry points for all poppler/cairo interaction with untrusted
//! PDF bytes - both are re-invocations of the same `inkpdf` binary (see
//! `main.rs`'s argv dispatch), sandboxed via `engine::sandbox::apply` before
//! touching a single byte of PDF content:
//!
//! - **render worker**: long-lived, one per open PDF tab. Loads the PDF
//!   once, then serves repeated raster-render requests for on-screen
//!   display (`engine::pdf::PdfDocument` is the client side of this).
//! - **export worker**: short-lived, one per "Export as PDF" click. Given a
//!   whole (already-flattened) `Document` model, writes the real PDF file
//!   to a destination path fixed at spawn time, then exits.
//!
//! Splitting these two roles - rather than one do-everything worker - keeps
//! each one's sandbox tight: the render worker never gets any filesystem
//! write access at all, and the export worker only ever gets write access
//! to the one path it was told to write to.

use std::io::{Write, stdin, stdout};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use gtk::cairo;
use gtk::glib;

use crate::engine::document::Document;
use crate::engine::sandbox;
use crate::engine::worker_protocol::{
    ExportRequest, ExportResponse, RenderRequest, RenderResponse, read_message, write_message,
};

pub const RENDER_WORKER_ARG: &str = "--pdf-render-worker";
pub const EXPORT_WORKER_ARG: &str = "--pdf-export-worker";
/// Not a real worker role - just proves the sandbox blocks what it should;
/// see `sandbox::run_selftest`.
pub const SANDBOX_SELFTEST_ARG: &str = "--pdf-sandbox-selftest";

/// Path to the `inkpdf` binary to re-invoke as a worker. Uses the real
/// running executable in production; unit tests run inside a separate test
/// harness binary (`target/<profile>/deps/inkpdf-<hash>`) that has no
/// worker-dispatch logic of its own - `CARGO_BIN_EXE_inkpdf` isn't available
/// here (Cargo only sets it for integration tests/benches), so tests derive
/// the sibling `inkpdf` binary's path instead, which `cargo test` already
/// built alongside the test harness.
#[cfg(not(test))]
pub(crate) fn worker_exe_path() -> PathBuf {
    std::env::current_exe().expect("current executable path")
}

#[cfg(test)]
pub(crate) fn worker_exe_path() -> PathBuf {
    let mut exe = std::env::current_exe().expect("current test exe path");
    exe.pop(); // drop the test harness binary's own file name
    if exe.ends_with("deps") {
        exe.pop();
    }
    exe.push("inkpdf");
    exe
}

/// Spawns a fresh, sandboxed export worker, hands it `doc`, and waits for it
/// to write the flattened PDF to `path`.
pub fn spawn_export_worker(doc: &Document, path: &Path) -> Result<()> {
    let mut child = Command::new(worker_exe_path())
        .arg(EXPORT_WORKER_ARG)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("spawning sandboxed PDF export worker")?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let mut stdout = child.stdout.take().expect("piped stdout");

    let json = serde_json::to_vec(doc).context("serializing document for export")?;
    write_message(&mut stdin, &ExportRequest::Export, &json)?;
    drop(stdin); // EOF, so the worker's single read_message call unblocks

    let (resp, _): (ExportResponse, Vec<u8>) = read_message(&mut stdout)?;
    child.wait().ok();
    match resp {
        ExportResponse::Done => Ok(()),
        ExportResponse::Error(msg) => bail!("{msg}"),
    }
}

/// Entry point when re-invoked with [`RENDER_WORKER_ARG`]. Never returns
/// normally - exits once the parent closes its end of the pipe (the tab
/// closed, or the app quit).
pub fn run_render_worker() -> ! {
    sandbox::apply(&[]);

    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();
    let mut doc: Option<poppler::Document> = None;

    loop {
        let Ok((req, blob)) = read_message::<RenderRequest, _>(&mut stdin) else {
            std::process::exit(0);
        };
        match req {
            RenderRequest::Load => {
                let bytes = glib::Bytes::from_owned(blob);
                let resp = match poppler::Document::from_bytes(&bytes, None) {
                    Ok(d) => {
                        let n = d.n_pages().max(0) as usize;
                        let page_sizes =
                            (0..n).map(|i| d.page(i as i32).map(|p| p.size()).unwrap_or((595.0, 842.0))).collect();
                        doc = Some(d);
                        RenderResponse::Loaded { page_sizes }
                    }
                    Err(e) => RenderResponse::Error(format!("could not open PDF: {e}")),
                };
                let _ = write_message(&mut stdout, &resp, &[]);
            }
            RenderRequest::RenderPage { page_index, width, height, zoom } => {
                let resp = render_one_page(doc.as_ref(), page_index, width, height, zoom);
                let (header, blob) = match resp {
                    Ok((w, h, stride, data)) => (RenderResponse::Rendered { width: w, height: h, stride }, data),
                    Err(e) => (RenderResponse::Error(format!("{e:#}")), Vec::new()),
                };
                let _ = write_message(&mut stdout, &header, &blob);
            }
        }
    }
}

fn render_one_page(
    doc: Option<&poppler::Document>,
    page_index: usize,
    width: i32,
    height: i32,
    zoom: f64,
) -> Result<(i32, i32, i32, Vec<u8>)> {
    let doc = doc.ok_or_else(|| anyhow::anyhow!("no PDF loaded"))?;
    let page = doc.page(page_index as i32).ok_or_else(|| anyhow::anyhow!("page index out of range"))?;
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)?;
    {
        let c = cairo::Context::new(&surface)?;
        c.set_source_rgb(1.0, 1.0, 1.0);
        c.paint()?;
        c.scale(zoom, zoom);
        page.render(&c);
    }
    surface.flush();
    let stride = surface.stride();
    let data = surface.data()?.to_vec();
    Ok((width, height, stride, data))
}

/// Entry point when re-invoked with [`EXPORT_WORKER_ARG`]. Sandboxed with
/// write access scoped to `dest`'s parent directory only - reads exactly
/// one request, does the job, replies once, and exits.
pub fn run_export_worker(dest: PathBuf) -> ! {
    let write_dir = dest.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    sandbox::apply(&[&write_dir]);

    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();

    let result: Result<()> = (|| {
        let (_req, blob): (ExportRequest, Vec<u8>) = read_message(&mut stdin)?;
        let doc: Document = serde_json::from_slice(&blob).context("parsing document to export")?;
        // Reaches into `ui::canvas` for the actual drawing code (annotation
        // rendering, page patterns, math/markdown layout) rather than
        // duplicating ~thousands of lines of pure-Cairo logic that happens
        // to live there; it's `pub(crate)`-scoped and doesn't touch GTK
        // widgets, only `cairo::Context` and the document model.
        crate::ui::canvas::render_document_to_pdf_local(&doc, &dest)
    })();

    let resp = match result {
        Ok(()) => ExportResponse::Done,
        Err(e) => ExportResponse::Error(format!("{e:#}")),
    };
    let _ = write_message(&mut stdout, &resp, &[]);
    let _ = stdout.flush();
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Proves the sandbox actually blocks what it should - not just that
    /// applying it didn't print a warning. Runs `sandbox::run_selftest` in
    /// a real subprocess (applying Landlock/seccomp in this test process
    /// itself would permanently restrict every other test sharing it).
    #[test]
    fn sandbox_blocks_write_and_exec_but_not_read() {
        let output = Command::new(worker_exe_path())
            .arg(SANDBOX_SELFTEST_ARG)
            .output()
            .expect("spawning sandbox selftest");
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(stdout.contains("write: blocked"), "sandbox should block writing outside granted dirs: {stdout}");
        assert!(stdout.contains("exec: blocked"), "sandbox should block executing new processes: {stdout}");
        assert!(stdout.contains("read: allowed"), "sandbox should still allow plain reads: {stdout}");
    }
}
