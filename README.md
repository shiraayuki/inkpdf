# inkpdf

A PDF viewer and annotator for Linux. Open a PDF (or start from a blank page) and draw, type,
or sketch on top with a pen, shapes, text, Markdown, or LaTeX math boxes — annotations live in
their own layer over the rendered PDF, and export back into a real, flattened PDF file.

Built with Rust, GTK4, and libadwaita. Inspired by [Rnote](https://github.com/flxzt/rnote)'s
simplicity, with room to grow toward Xournal++-style depth.

## Features

- **Open PDFs or start blank** — view and annotate existing PDFs, or start from a blank page
  with a choice of plain, grid, dotted, or lined patterns.
- **Pen** with live stroke smoothing (via `ink-stroke-modeler`, the same smoothing Rnote uses),
  rendered as vector strokes, not raster ink.
- **Shapes** — rectangle, ellipse, and line, with adjustable color and stroke width.
- **Text** — multi-line text boxes with bold, italic, underline, strikethrough, custom color,
  and font selection.
- **Markdown** boxes — headings, bold/italic, lists, code, rules, and inline LaTeX math
  (`$...$` / `$$...$$`) rendered live.
- **Dedicated LaTeX boxes** — a box that's entirely math mode, no `$...$` needed; resize a
  formula that's gotten too small by right-click-dragging inside it.
- **Eraser** and a **lasso** tool for rectangular or freeform multi-selection, with bulk
  restyling (color, width) across a whole selection.
- **Undo/redo** across every tool.
- **Tabs** — multiple documents open at once.
- **Built-in file browser** — a slide-in sidebar to open files without a dialog.
- **Own file format** (`.inkpdf`) — gzip-compressed JSON that embeds the source PDF, so a saved
  file is fully self-contained.
- **PDF export** — flattens the background and every annotation into a real, standalone PDF.
- **Sandboxed PDF parsing** — all PDF parsing/rendering runs in a separate process, restricted
  via Landlock and seccomp, so a malicious or malformed PDF can't do anything with the app's
  own privileges.
- Light/dark theme, persisted tool defaults, and zoom controls.

## Installation

inkpdf currently only targets Linux. There's no published Flathub/AUR release yet, but you can
build a proper sandboxed Flatpak (or build from source) yourself.

### Flatpak

Requires `flatpak` and `flatpak-builder`, plus the Flathub remote:

```sh
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
```

Then, from a checkout of this repo:

```sh
flatpak install --user flathub org.gnome.Platform//49 org.gnome.Sdk//49 \
    org.freedesktop.Sdk.Extension.rust-stable//25.08
flatpak-builder --user --install --force-clean build-dir de.nikolas.inkpdf.json
flatpak run de.nikolas.inkpdf
```

This builds poppler from source and inkpdf offline against the vendored crates in
`cargo-sources.json`, so the first build takes a few minutes. All PDF parsing/rendering happens
in a Landlock+seccomp-sandboxed subprocess regardless of how you install inkpdf (see
"Sandboxed PDF parsing" above) — the Flatpak's own sandbox is on top of that, not instead of it.

### Build from source

#### Dependencies

- Rust (stable, edition 2024 support required — 1.85+)
- `gtk4`, `libadwaita`, `poppler-glib` (development packages)

On Arch Linux:

```sh
sudo pacman -S gtk4 libadwaita poppler-glib
```

#### Build

```sh
git clone https://github.com/shiraayuki/inkpdf
cd inkpdf
cargo build --release
./target/release/inkpdf
```

## Screenshots

<!-- TODO: add screenshots -->

![Main window](docs/screenshots/main-window.png)
![Annotating a PDF](docs/screenshots/annotating.png)
![LaTeX box](docs/screenshots/latex-box.png)
