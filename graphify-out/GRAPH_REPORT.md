# Graph Report - src  (2026-07-12)

## Corpus Check
- 2 files · ~7,915 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 229 nodes · 555 edges · 14 communities (7 shown, 7 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- UI Window & Widgets
- Canvas Rendering & Interaction
- Page Rendering & Document State
- Text Editing & Annotations
- Document Model (engine::document)
- PDF Engine & Loading
- App Entry Point
- Canvas Context Type
- Canvas Option Type
- Canvas Self Reference
- Canvas Uuid Type
- Window Path Type
- Window PathBuf Type

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 49 edges
2. `TextEdit` - 31 edges
3. `Document` - 17 edges
4. `build()` - 15 edges
5. `State` - 13 edges
6. `draw_overlay()` - 13 edges
7. `page_surface()` - 12 edges
8. `size_stepper()` - 11 edges
9. `measure_glyphs()` - 10 edges
10. `WindowUi` - 10 edges

## Surprising Connections (you probably didn't know these)
- `OpenDocument` --references--> `Document`  [EXTRACTED]
  engine/mod.rs → engine/document.rs
- `PdfDocument` --references--> `Document`  [EXTRACTED]
  engine/pdf.rs → engine/document.rs
- `build_details_panel()` --references--> `Canvas`  [EXTRACTED]
  ui/window.rs → ui/canvas.rs
- `build_tool_strip()` --references--> `Canvas`  [EXTRACTED]
  ui/window.rs → ui/canvas.rs
- `page_text()` --references--> `Canvas`  [EXTRACTED]
  ui/window.rs → ui/canvas.rs

## Import Cycles
- None detected.

## Communities (14 total, 7 thin omitted)

### Community 0 - "UI Window & Widgets"
Cohesion: 0.11
Nodes (42): Application, ApplicationWindow, Box, Button, ColorDialogButton, Fn, IsA, MenuItem (+34 more)

### Community 1 - "Canvas Rendering & Interaction"
Cohesion: 0.10
Nodes (9): DrawingArea, Option, ScrolledWindow, Canvas, hit_test(), hit_test_maps_click_to_page_local_point(), Relative, Tool (+1 more)

### Community 2 - "Page Rendering & Document State"
Cohesion: 0.12
Nodes (37): Context, Document, HashMap, ImageSurface, Page, PdfDocument, R, Rc (+29 more)

### Community 3 - "Text Editing & Annotations"
Cohesion: 0.11
Nodes (15): Annotation, Color, FnOnce, Key, ModifierType, Propagation, TextRun, empty_edit() (+7 more)

### Community 4 - "Document Model (engine::document)"
Cohesion: 0.14
Nodes (25): Default, Annotation, AnnotationKind, Color, Document, insert_blank_page_adds_page_at_index(), Page, PageKind (+17 more)

### Community 5 - "PDF Engine & Loading"
Cohesion: 0.16
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

## Knowledge Gaps
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas Rendering & Interaction` to `UI Window & Widgets`, `Page Rendering & Document State`, `Text Editing & Annotations`?**
  _High betweenness centrality (0.428) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model (engine::document)` to `PDF Engine & Loading`?**
  _High betweenness centrality (0.318) - this node is a cross-community bridge._
- **Why does `TextEdit` connect `Text Editing & Annotations` to `Canvas Rendering & Interaction`, `Page Rendering & Document State`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **Should `UI Window & Widgets` be split into smaller, more focused modules?**
  _Cohesion score 0.10707070707070707 - nodes in this community are weakly interconnected._
- **Should `Canvas Rendering & Interaction` be split into smaller, more focused modules?**
  _Cohesion score 0.09872241579558652 - nodes in this community are weakly interconnected._
- **Should `Page Rendering & Document State` be split into smaller, more focused modules?**
  _Cohesion score 0.12317073170731707 - nodes in this community are weakly interconnected._
- **Should `Text Editing & Annotations` be split into smaller, more focused modules?**
  _Cohesion score 0.1126984126984127 - nodes in this community are weakly interconnected._