# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 8 files · ~11,184 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 242 nodes · 638 edges · 8 communities (7 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8fab60df`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- UI Window & Widgets
- Canvas Rendering & Interaction
- Page Rendering & Document State
- Text Editing & Annotations
- Document Model (engine::document)
- PDF Engine & Loading
- App Entry Point

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 58 edges
2. `TextEdit` - 35 edges
3. `Document` - 20 edges
4. `TextStyle` - 15 edges
5. `build()` - 15 edges
6. `State` - 13 edges
7. `page_surface()` - 13 edges
8. `draw_overlay()` - 13 edges
9. `PdfDocument` - 12 edges
10. `Page` - 11 edges

## Surprising Connections (you probably didn't know these)
- `color_from_rgba()` --references--> `Color`  [EXTRACTED]
  src/ui/window.rs → src/engine/document.rs
- `apply_glyph_font()` --references--> `TextStyle`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `Glyph` --references--> `TextStyle`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `State` --references--> `TextStyle`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `ann_glyphs()` --references--> `TextAnnotation`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (8 total, 1 thin omitted)

### Community 0 - "UI Window & Widgets"
Cohesion: 0.11
Nodes (45): Application, ApplicationWindow, Box, Button, ColorDialogButton, IsA, MenuItem, RGBA (+37 more)

### Community 1 - "Canvas Rendering & Interaction"
Cohesion: 0.07
Nodes (13): DrawingArea, ScrolledWindow, Canvas, content_size(), hit_test(), hit_test_maps_click_to_page_local_point(), Relative, Fn (+5 more)

### Community 2 - "Page Rendering & Document State"
Cohesion: 0.12
Nodes (41): HashMap, ImageSurface, R, Rc, RefCell, a4_page(), ann_glyphs(), annotation_at() (+33 more)

### Community 3 - "Text Editing & Annotations"
Cohesion: 0.16
Nodes (6): FnOnce, Key, ModifierType, Propagation, text_edit_insert_delete_and_navigate(), TextEdit

### Community 4 - "Document Model (engine::document)"
Cohesion: 0.12
Nodes (27): Default, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), Page (+19 more)

### Community 5 - "PDF Engine & Loading"
Cohesion: 0.16
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas Rendering & Interaction` to `UI Window & Widgets`, `Page Rendering & Document State`, `Text Editing & Annotations`?**
  _High betweenness centrality (0.396) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model (engine::document)` to `Canvas Rendering & Interaction`, `Page Rendering & Document State`, `PDF Engine & Loading`?**
  _High betweenness centrality (0.120) - this node is a cross-community bridge._
- **Why does `TextEdit` connect `Text Editing & Annotations` to `Canvas Rendering & Interaction`, `Page Rendering & Document State`, `Document Model (engine::document)`?**
  _High betweenness centrality (0.112) - this node is a cross-community bridge._
- **Should `UI Window & Widgets` be split into smaller, more focused modules?**
  _Cohesion score 0.10638297872340426 - nodes in this community are weakly interconnected._
- **Should `Canvas Rendering & Interaction` be split into smaller, more focused modules?**
  _Cohesion score 0.06766917293233082 - nodes in this community are weakly interconnected._
- **Should `Page Rendering & Document State` be split into smaller, more focused modules?**
  _Cohesion score 0.1191919191919192 - nodes in this community are weakly interconnected._
- **Should `Document Model (engine::document)` be split into smaller, more focused modules?**
  _Cohesion score 0.12380952380952381 - nodes in this community are weakly interconnected._