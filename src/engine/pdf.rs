use std::cell::RefCell;
use std::io::{BufReader, BufWriter};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use gtk::cairo;

use crate::engine::pdf_worker::{RENDER_WORKER_ARG, worker_exe_path};
use crate::engine::worker_protocol::{RenderRequest, RenderResponse, read_message, write_message};

/// A loaded PDF, rendered out-of-process by a sandboxed worker (see
/// `engine::pdf_worker::run_render_worker`) so a malicious/malformed PDF
/// exploiting a bug in poppler's C++ parser can't do anything with the
/// app's own privileges - the worker never gets filesystem write/execute
/// access at all. One worker per open PDF, spawned on `from_bytes` and
/// killed on `Drop`.
pub struct PdfDocument {
    child: Child,
    io: RefCell<(BufWriter<ChildStdin>, BufReader<ChildStdout>)>,
    page_sizes: Vec<(f64, f64)>,
}

impl PdfDocument {
    /// Takes `Arc<[u8]>` (not `Vec<u8>`) so the caller can share the same
    /// buffer with `PdfSource.bytes` instead of duplicating it.
    pub fn from_bytes(data: Arc<[u8]>) -> Result<Self> {
        let mut child = Command::new(worker_exe_path())
            .arg(RENDER_WORKER_ARG)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("spawning sandboxed PDF render worker")?;
        let mut stdin = BufWriter::new(child.stdin.take().expect("piped stdin"));
        let mut stdout = BufReader::new(child.stdout.take().expect("piped stdout"));

        write_message(&mut stdin, &RenderRequest::Load, &data)?;
        let (resp, _): (RenderResponse, Vec<u8>) = read_message(&mut stdout)?;
        let page_sizes = match resp {
            RenderResponse::Loaded { page_sizes } => page_sizes,
            RenderResponse::Error(msg) => bail!("could not open PDF: {msg}"),
            RenderResponse::Rendered { .. } => bail!("unexpected worker response to Load"),
        };

        Ok(Self { child, io: RefCell::new((stdin, stdout)), page_sizes })
    }

    pub fn n_pages(&self) -> usize {
        self.page_sizes.len()
    }

    /// Falls back to A4 for an out-of-range index rather than panicking.
    pub fn page_size(&self, index: usize) -> (f64, f64) {
        page_size_or_a4(&self.page_sizes, index)
    }

    /// Renders page `index` at `width x height` device pixels (the caller
    /// controls resolution via `zoom`) and returns it ready to blit via
    /// `Context::set_source_surface`.
    pub fn render_page_argb(&self, index: usize, width: i32, height: i32, zoom: f64) -> Result<cairo::ImageSurface> {
        let mut io = self.io.borrow_mut();
        let (stdin, stdout) = &mut *io;
        write_message(stdin, &RenderRequest::RenderPage { page_index: index, width, height, zoom }, &[])?;
        let (resp, data): (RenderResponse, Vec<u8>) = read_message(stdout)?;
        match resp {
            RenderResponse::Rendered { width, height, stride } => {
                cairo::ImageSurface::create_for_data(data, cairo::Format::ARgb32, width, height, stride)
                    .map_err(|e| anyhow::anyhow!("building surface from worker output: {e}"))
            }
            RenderResponse::Error(msg) => bail!("render failed: {msg}"),
            RenderResponse::Loaded { .. } => bail!("unexpected worker response to RenderPage"),
        }
    }
}

impl Drop for PdfDocument {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn page_size_or_a4(sizes: &[(f64, f64)], index: usize) -> (f64, f64) {
    sizes.get(index).copied().unwrap_or((595.0, 842.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_page_with_content() {
        let Ok(path) = std::env::var("INKPDF_TEST_PDF") else {
            eprintln!("INKPDF_TEST_PDF not set - skipping");
            return;
        };

        let data = std::fs::read(&path).expect("read pdf");
        let pdf = PdfDocument::from_bytes(data.into()).expect("PDF should load");
        assert!(pdf.n_pages() > 0);

        let (w, h) = pdf.page_size(0);
        let mut surface = pdf.render_page_argb(0, w.ceil() as i32, h.ceil() as i32, 1.0).expect("render should succeed");
        surface.flush();

        let data = surface.data().expect("surface data readable");
        let non_white = data.iter().filter(|&&b| b != 0xFF).count();
        assert!(non_white > 0, "rendered page is blank");
    }

    #[test]
    fn page_size_falls_back_instead_of_panicking_on_out_of_range_index() {
        assert_eq!(page_size_or_a4(&[(100.0, 200.0)], 5), (595.0, 842.0));
        assert_eq!(page_size_or_a4(&[(100.0, 200.0)], 0), (100.0, 200.0));
    }
}
