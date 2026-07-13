# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 14 files · ~32,303 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 536 nodes · 1535 edges · 19 communities (13 shown, 6 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 28 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `9448698b`
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

## Communities (19 total, 6 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.05
Nodes (93): Context, HashMap, HeadingLevel, a4_page(), ann_glyphs(), annotation_at(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside() (+85 more)

### Community 1 - ".record_change"
Cohesion: 0.11
Nodes (6): ScrolledWindow, Canvas, LassoShape, DrawingArea, Rc, RefCell

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (65): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+57 more)

### Community 3 - "Document Model"
Cohesion: 0.10
Nodes (25): Instant, ModelerInputEventType, Annotation, AnnotationKind, Page, ShapeAnnotation, ShapeKind, DragKind (+17 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.12
Nodes (14): Key, ModifierType, Propagation, ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), read_math_span_finds_inline_and_block_delimiters() (+6 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.24
Nodes (9): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+1 more)

### Community 6 - "Canvas"
Cohesion: 0.22
Nodes (3): Path, Result, Tool

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 11 - "Option"
Cohesion: 0.06
Nodes (43): BufReader, BufWriter, Child, ChildStdin, ChildStdout, Drop, H, page_size_or_a4() (+35 more)

### Community 20 - "Option"
Cohesion: 0.25
Nodes (8): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, stroke_halo(), text_cursor(), text_line_height()

### Community 21 - ".lift_annotation"
Cohesion: 0.06
Nodes (45): Color, default_font(), Document, insert_blank_page_adds_page_at_index(), LatexAnnotation, MarkdownAnnotation, PageKind, PagePattern (+37 more)

## Knowledge Gaps
- **2 isolated node(s):** `RenderRequest`, `ExportRequest`
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `Canvas`, `.update_layout`, `Option`, `.on_click`, `.attach_input`, `.update_layout`, `Option`, `.lift_annotation`?**
  _High betweenness centrality (0.403) - this node is a cross-community bridge._
- **Why does `Document` connect `.lift_annotation` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Engine / PDF Loading`, `Canvas`, `Option`?**
  _High betweenness centrality (0.162) - this node is a cross-community bridge._
- **Why does `PdfDocument` connect `Option` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.110) - this node is a cross-community bridge._
- **What connects `RenderRequest`, `ExportRequest` to the rest of the system?**
  _2 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.05009481668773704 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.10541310541310542 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.0711484593837535 - nodes in this community are weakly interconnected._