# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 8 files · ~16,825 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 330 nodes · 944 edges · 9 communities (8 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `1b1d4eac`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Canvas Rendering & Hit-Testing
- Canvas Input & Edit Sessions
- Window & Tool UI
- Document Model
- Text Editing & Styling
- Engine / PDF Loading
- Option
- App Entry Point

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 88 edges
2. `TextEdit` - 35 edges
3. `WindowUi` - 30 edges
4. `build()` - 24 edges
5. `Document` - 22 edges
6. `State` - 19 edges
7. `Color` - 16 edges
8. `Page` - 16 edges
9. `TextStyle` - 15 edges
10. `draw_overlay()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `Draw` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `State` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `color_from_rgba()` --references--> `Color`  [EXTRACTED]
  src/ui/window.rs → src/engine/document.rs
- `apply_glyph_font()` --references--> `TextStyle`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (9 total, 1 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.07
Nodes (62): HashMap, ImageSurface, Instant, ModelerInputEventType, Page, a4_page(), ann_glyphs(), annotation_at() (+54 more)

### Community 1 - "Canvas Input & Edit Sessions"
Cohesion: 0.07
Nodes (7): DrawingArea, ScrolledWindow, Canvas, content_size(), Relative, Uuid, Tool

### Community 2 - "Window & Tool UI"
Cohesion: 0.08
Nodes (60): Application, ApplicationWindow, Button, Cell, ColorDialogButton, IsA, Label, MenuItem (+52 more)

### Community 3 - "Document Model"
Cohesion: 0.08
Nodes (32): Default, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), PageKind (+24 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.17
Nodes (5): Key, ModifierType, Propagation, text_edit_insert_delete_and_navigate(), TextEdit

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.16
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Option"
Cohesion: 0.29
Nodes (8): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), FnOnce, Option, stroke_halo(), text_cursor()

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas Input & Edit Sessions` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Option`?**
  _High betweenness centrality (0.448) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Canvas Input & Edit Sessions`, `Window & Tool UI`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.131) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `Canvas Input & Edit Sessions`?**
  _High betweenness centrality (0.107) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.06772151898734177 - nodes in this community are weakly interconnected._
- **Should `Canvas Input & Edit Sessions` be split into smaller, more focused modules?**
  _Cohesion score 0.07380520266182698 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07603603603603604 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.0824829931972789 - nodes in this community are weakly interconnected._