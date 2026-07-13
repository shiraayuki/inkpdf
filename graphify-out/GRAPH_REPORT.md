# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 17 files · ~35,519 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 543 nodes · 1541 edges · 18 communities (14 shown, 4 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 28 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8577e64e`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Canvas Rendering & Hit-Testing
- .record_change
- Window & Tool UI
- Document Model
- Text Editing & Styling
- Engine / PDF Loading
- Canvas
- App Entry Point
- .record_change
- .update_layout
- Option
- Option
- .on_click
- .attach_input
- Context
- Option
- .lift_annotation

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 128 edges
2. `WindowUi` - 38 edges
3. `TextEdit` - 37 edges
4. `build()` - 31 edges
5. `Document` - 26 edges
6. `State` - 23 edges
7. `PdfDocument` - 21 edges
8. `Color` - 20 edges
9. `page_surface()` - 17 edges
10. `draw_overlay()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `pattern_thumbnail()` --calls--> `draw_page_pattern()`  [INFERRED]
  src/ui/window.rs → src/ui/canvas.rs
- `refresh()` --calls--> `add_secondary_click()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `refresh()` --calls--> `show_menu()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `build()` --calls--> `refresh()`  [INFERRED]
  src/ui/window.rs → src/ui/file_browser.rs
- `Draw` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (18 total, 4 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.05
Nodes (92): Context, HashMap, HeadingLevel, a4_page(), annotation_at(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), annotation_bounds() (+84 more)

### Community 1 - ".record_change"
Cohesion: 0.09
Nodes (7): ScrolledWindow, ShapeKind, Canvas, LassoShape, DrawingArea, Fn, FnOnce

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (65): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+57 more)

### Community 3 - "Document Model"
Cohesion: 0.11
Nodes (24): Instant, ModelerInputEventType, Annotation, Page, DragKind, DragState, Draw, eraser_hits() (+16 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.12
Nodes (14): Key, ModifierType, Propagation, ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), read_math_span_finds_inline_and_block_delimiters() (+6 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 6 - "Canvas"
Cohesion: 0.15
Nodes (3): content_size(), Path, Result

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 11 - "Option"
Cohesion: 0.05
Nodes (52): BufReader, BufWriter, Child, ChildStdin, ChildStdout, Drop, H, file_name() (+44 more)

### Community 13 - ".on_click"
Cohesion: 0.22
Nodes (3): ann_glyphs(), glyphs_from_plain(), Uuid

### Community 14 - ".attach_input"
Cohesion: 0.29
Nodes (6): Build, Dependencies, Features, inkpdf, Installation, Screenshots

### Community 20 - "Option"
Cohesion: 0.21
Nodes (9): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, stroke_halo(), text_cursor(), text_line_height() (+1 more)

### Community 21 - ".lift_annotation"
Cohesion: 0.10
Nodes (33): AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), LatexAnnotation, MarkdownAnnotation, PageKind (+25 more)

## Knowledge Gaps
- **6 isolated node(s):** `RenderRequest`, `ExportRequest`, `Features`, `Dependencies`, `Build` (+1 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Canvas`, `.update_layout`, `Option`, `.on_click`, `Option`, `.lift_annotation`?**
  _High betweenness centrality (0.392) - this node is a cross-community bridge._
- **Why does `Document` connect `.lift_annotation` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Canvas`, `Option`?**
  _High betweenness centrality (0.158) - this node is a cross-community bridge._
- **Why does `PdfDocument` connect `Option` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`?**
  _High betweenness centrality (0.107) - this node is a cross-community bridge._
- **What connects `RenderRequest`, `ExportRequest`, `Features` to the rest of the system?**
  _6 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.050353925353925355 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.08571428571428572 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.0711484593837535 - nodes in this community are weakly interconnected._