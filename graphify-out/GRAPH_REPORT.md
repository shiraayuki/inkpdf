# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 14 files · ~32,128 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 535 nodes · 1532 edges · 20 communities (15 shown, 5 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 28 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `045de3c3`
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
- .is_empty
- Context
- Option
- .lift_annotation

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 128 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 37 edges
4. `build()` - 30 edges
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
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (20 total, 5 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.05
Nodes (97): Context, HeadingLevel, a4_page(), ann_glyphs(), annotation_at(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), annotation_bounds() (+89 more)

### Community 1 - ".record_change"
Cohesion: 0.11
Nodes (6): ScrolledWindow, Canvas, LassoShape, DrawingArea, Rc, RefCell

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (65): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+57 more)

### Community 3 - "Document Model"
Cohesion: 0.07
Nodes (42): HashMap, Instant, ModelerInputEventType, Annotation, AnnotationKind, Color, default_font(), LatexAnnotation (+34 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.12
Nodes (7): Key, ModifierType, Propagation, Path, Result, text_edit_insert_delete_and_navigate(), TextEdit

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.24
Nodes (9): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+1 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 9 - ".update_layout"
Cohesion: 0.13
Nodes (3): content_size(), Relative, FnOnce

### Community 11 - "Option"
Cohesion: 0.06
Nodes (43): BufReader, BufWriter, Child, ChildStdin, ChildStdout, Drop, H, page_size_or_a4() (+35 more)

### Community 15 - ".is_empty"
Cohesion: 0.18
Nodes (6): clamp_translate(), clamp_translate_keeps_box_on_page(), DragKind, translate_annotation(), translate_annotation_shifts_every_kind(), union_bounds()

### Community 20 - "Option"
Cohesion: 0.23
Nodes (8): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, stroke_halo(), text_cursor(), Tool

### Community 21 - ".lift_annotation"
Cohesion: 0.24
Nodes (15): Document, insert_blank_page_adds_page_at_index(), Self, load(), load_capped(), load_capped_accepts_documents_within_the_cap(), load_capped_rejects_a_gzip_bomb(), load_rejects_garbage() (+7 more)

## Knowledge Gaps
- **2 isolated node(s):** `RenderRequest`, `ExportRequest`
  These have ≤1 connection - possible missing edges or undocumented components.
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `.update_layout`, `Option`, `.on_click`, `.attach_input`, `.is_empty`, `.update_layout`, `Option`?**
  _High betweenness centrality (0.402) - this node is a cross-community bridge._
- **Why does `Document` connect `.lift_annotation` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `Option`, `.on_click`?**
  _High betweenness centrality (0.162) - this node is a cross-community bridge._
- **Why does `PdfDocument` connect `Option` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.110) - this node is a cross-community bridge._
- **What connects `RenderRequest`, `ExportRequest` to the rest of the system?**
  _2 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.05321100917431193 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.11 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07200229489386116 - nodes in this community are weakly interconnected._