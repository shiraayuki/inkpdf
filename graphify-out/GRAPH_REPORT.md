# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 9 files · ~18,997 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 374 nodes · 1065 edges · 10 communities (9 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c86d1651`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Canvas Rendering & Hit-Testing
- Window & Tool UI
- Document Model
- Text Editing & Styling
- Engine / PDF Loading
- Canvas
- App Entry Point
- settings.rs
- Option

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 105 edges
2. `TextEdit` - 35 edges
3. `WindowUi` - 31 edges
4. `build()` - 28 edges
5. `Document` - 22 edges
6. `Color` - 20 edges
7. `State` - 20 edges
8. `draw_overlay()` - 17 edges
9. `Page` - 16 edges
10. `TextStyle` - 15 edges

## Surprising Connections (you probably didn't know these)
- `pattern_thumbnail()` --calls--> `draw_page_pattern()`  [INFERRED]
  src/ui/window.rs → src/ui/canvas.rs
- `Draw` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `State` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `AppSettings` --references--> `Color`  [EXTRACTED]
  src/ui/settings.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (10 total, 1 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.07
Nodes (68): HashMap, ImageSurface, Instant, ModelerInputEventType, Page, a4_page(), ann_glyphs(), annotation_at() (+60 more)

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (62): Application, ApplicationWindow, Button, Cell, ColorDialogButton, IsA, Label, MenuItem (+54 more)

### Community 3 - "Document Model"
Cohesion: 0.09
Nodes (32): Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), PageKind, PagePattern (+24 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.11
Nodes (8): Key, ModifierType, Propagation, Fn, String, text_edit_insert_delete_and_navigate(), text_of(), TextEdit

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.14
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.06
Nodes (6): ScrolledWindow, Canvas, Relative, DrawingArea, FnOnce, Tool

### Community 9 - "settings.rs"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 11 - "Option"
Cohesion: 0.27
Nodes (9): Cursor, circle_cursor(), cursor_from_draw(), Overlay, plus_cursor(), Option, Uuid, stroke_halo() (+1 more)

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `Option`?**
  _High betweenness centrality (0.459) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Engine / PDF Loading`, `Canvas`?**
  _High betweenness centrality (0.109) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `Canvas`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.06621226874391431 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07425907425907426 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.08695652173913043 - nodes in this community are weakly interconnected._
- **Should `Text Editing & Styling` be split into smaller, more focused modules?**
  _Cohesion score 0.10810810810810811 - nodes in this community are weakly interconnected._