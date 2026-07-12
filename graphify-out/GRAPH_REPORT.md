# Graph Report - src  (2026-07-12)

## Corpus Check
- Corpus is ~7,884 words - fits in a single context window. You may not need a graph.

## Summary
- 215 nodes · 555 edges · 8 communities (7 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- UI Window & Panels
- Canvas Rendering
- Canvas Interaction
- Document Model & Storage
- Text Editor Logic
- PDF Engine & Loading
- App Entry Point

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 49 edges
2. `TextEdit` - 31 edges
3. `Document` - 20 edges
4. `build()` - 15 edges
5. `State` - 13 edges
6. `draw_overlay()` - 13 edges
7. `PdfDocument` - 12 edges
8. `page_surface()` - 12 edges
9. `Page` - 11 edges
10. `size_stepper()` - 11 edges

## Surprising Connections (you probably didn't know these)
- `Glyph` --references--> `Color`  [EXTRACTED]
  ui/canvas.rs → engine/document.rs
- `State` --references--> `Color`  [EXTRACTED]
  ui/canvas.rs → engine/document.rs
- `ann_glyphs()` --references--> `TextAnnotation`  [EXTRACTED]
  ui/canvas.rs → engine/document.rs
- `DragState` --references--> `Annotation`  [EXTRACTED]
  ui/canvas.rs → engine/document.rs
- `TextEdit` --references--> `Annotation`  [EXTRACTED]
  ui/canvas.rs → engine/document.rs

## Import Cycles
- None detected.

## Communities (8 total, 1 thin omitted)

### Community 0 - "UI Window & Panels"
Cohesion: 0.11
Nodes (41): Application, ApplicationWindow, Box, Button, ColorDialogButton, Fn, IsA, MenuItem (+33 more)

### Community 1 - "Canvas Rendering"
Cohesion: 0.13
Nodes (37): Page, HashMap, ImageSurface, R, Rc, RefCell, a4_page(), ann_glyphs() (+29 more)

### Community 2 - "Canvas Interaction"
Cohesion: 0.10
Nodes (6): DrawingArea, ScrolledWindow, Canvas, content_size(), Relative, Tool

### Community 3 - "Document Model & Storage"
Cohesion: 0.12
Nodes (24): Default, Annotation, AnnotationKind, Color, Document, insert_blank_page_adds_page_at_index(), PageKind, PdfSource (+16 more)

### Community 4 - "Text Editor Logic"
Cohesion: 0.14
Nodes (12): FnOnce, Key, ModifierType, Propagation, empty_edit(), String, shift_selects_and_color_applies_to_selection_only(), text_edit_insert_delete_and_navigate() (+4 more)

### Community 5 - "PDF Engine & Loading"
Cohesion: 0.16
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas Interaction` to `UI Window & Panels`, `Canvas Rendering`, `Document Model & Storage`, `Text Editor Logic`?**
  _High betweenness centrality (0.367) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model & Storage` to `Canvas Rendering`, `Canvas Interaction`, `PDF Engine & Loading`?**
  _High betweenness centrality (0.143) - this node is a cross-community bridge._
- **Why does `OpenDocument` connect `PDF Engine & Loading` to `UI Window & Panels`, `Canvas Rendering`, `Canvas Interaction`, `Document Model & Storage`?**
  _High betweenness centrality (0.120) - this node is a cross-community bridge._
- **Should `UI Window & Panels` be split into smaller, more focused modules?**
  _Cohesion score 0.11406423034330011 - nodes in this community are weakly interconnected._
- **Should `Canvas Rendering` be split into smaller, more focused modules?**
  _Cohesion score 0.1254355400696864 - nodes in this community are weakly interconnected._
- **Should `Canvas Interaction` be split into smaller, more focused modules?**
  _Cohesion score 0.10121457489878542 - nodes in this community are weakly interconnected._
- **Should `Document Model & Storage` be split into smaller, more focused modules?**
  _Cohesion score 0.125 - nodes in this community are weakly interconnected._