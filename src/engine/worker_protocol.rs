//! Wire format shared by the main process and the sandboxed PDF workers
//! (`engine::pdf_worker`): a small JSON "header" for the message shape plus
//! an optional raw binary blob (PDF bytes, a rendered pixel buffer, or a
//! serialized `Document`) - large payloads go through the blob rather than
//! JSON, which would bloat them ~3-4x as a number-per-byte array.

use std::io::{Read, Write};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Requests understood by the long-lived render worker (one per open PDF
/// tab): load the document once, then repeatedly rasterize pages for
/// on-screen display.
#[derive(Serialize, Deserialize)]
pub enum RenderRequest {
    /// The PDF bytes to parse follow as the message's raw blob.
    Load,
    /// Renders one page at the given device-pixel size (the caller controls
    /// resolution via `width`/`height`/`zoom`, matching what the on-screen
    /// cache needs).
    RenderPage { page_index: usize, width: i32, height: i32, zoom: f64 },
}

#[derive(Serialize, Deserialize)]
pub enum RenderResponse {
    Loaded { page_sizes: Vec<(f64, f64)> },
    /// Premultiplied ARGB32 pixels follow as the raw blob, `stride` bytes
    /// per row (matches `cairo::Format::ARgb32`/`ImageSurface::stride`).
    Rendered { width: i32, height: i32, stride: i32 },
    Error(String),
}

/// Request understood by the short-lived export worker: flatten a whole
/// document (background pages + every annotation) into a real PDF file at
/// the destination path given on its command line.
#[derive(Serialize, Deserialize)]
pub enum ExportRequest {
    /// The `Document` to flatten (JSON-serialized) follows as the raw blob.
    Export,
}

#[derive(Serialize, Deserialize)]
pub enum ExportResponse {
    Done,
    Error(String),
}

/// Refuses to read a message whose header or blob claims to be larger than
/// this - mirrors `storage`'s gzip-bomb cap: a worker (or, in principle, a
/// compromised one) shouldn't be able to make its counterpart allocate an
/// unbounded amount of memory just by claiming a huge length.
const MAX_MESSAGE_BYTES: u64 = 1024 * 1024 * 1024;

/// Writes `header` (JSON) then `blob` (raw bytes), each length-prefixed.
pub fn write_message<H: Serialize, W: Write>(w: &mut W, header: &H, blob: &[u8]) -> Result<()> {
    let json = serde_json::to_vec(header).context("serializing worker message header")?;
    w.write_all(&(json.len() as u32).to_le_bytes())?;
    w.write_all(&json)?;
    w.write_all(&(blob.len() as u64).to_le_bytes())?;
    w.write_all(blob)?;
    w.flush()?;
    Ok(())
}

/// Reads back a `(header, blob)` pair written by `write_message`.
pub fn read_message<H: for<'de> Deserialize<'de>, R: Read>(r: &mut R) -> Result<(H, Vec<u8>)> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf).context("reading worker message header length")?;
    let header_len = u32::from_le_bytes(len_buf) as u64;
    if header_len > MAX_MESSAGE_BYTES {
        bail!("worker message header implausibly large ({header_len} bytes)");
    }
    let mut header_buf = vec![0u8; header_len as usize];
    r.read_exact(&mut header_buf).context("reading worker message header")?;
    let header = serde_json::from_slice(&header_buf).context("parsing worker message header")?;

    let mut blob_len_buf = [0u8; 8];
    r.read_exact(&mut blob_len_buf).context("reading worker message blob length")?;
    let blob_len = u64::from_le_bytes(blob_len_buf);
    if blob_len > MAX_MESSAGE_BYTES {
        bail!("worker message blob exceeds the size cap ({blob_len} bytes)");
    }
    let mut blob = vec![0u8; blob_len as usize];
    r.read_exact(&mut blob).context("reading worker message blob")?;
    Ok((header, blob))
}
