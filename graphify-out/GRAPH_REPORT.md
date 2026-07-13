# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 8 files · ~16,051 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 314 nodes · 891 edges · 8 communities (7 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `56a1f08d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Canvas Rendering & Hit-Testing
- Canvas Input & Edit Sessions
- Window & Tool UI
- Document Model
- Text Editing & Styling
- Engine / PDF Loading
- App Entry Point

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 86 edges
2. `TextEdit` - 35 edges
3. `Document` - 22 edges
4. `WindowUi` - 21 edges
5. `build()` - 21 edges
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

## Communities (8 total, 1 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.08
Nodes (55): HashMap, ImageSurface, Instant, ModelerInputEventType, R, Page, a4_page(), ann_glyphs() (+47 more)

### Community 1 - "Canvas Input & Edit Sessions"
Cohesion: 0.06
Nodes (15): Cursor, DrawingArea, FnOnce, ScrolledWindow, Canvas, circle_cursor(), content_size(), cursor_from_draw() (+7 more)

### Community 2 - "Window & Tool UI"
Cohesion: 0.09
Nodes (54): Application, ApplicationWindow, Button, ColorDialogButton, IsA, Label, MenuItem, RGBA (+46 more)

### Community 3 - "Document Model"
Cohesion: 0.08
Nodes (32): Default, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), PageKind (+24 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.13
Nodes (12): Key, ModifierType, Propagation, ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), shift_selects_and_color_applies_to_selection_only() (+4 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.16
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas Input & Edit Sessions` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`?**
  _High betweenness centrality (0.443) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Canvas Input & Edit Sessions`, `Window & Tool UI`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.158) - this node is a cross-community bridge._
- **Why does `TextEdit` connect `Text Editing & Styling` to `Canvas Rendering & Hit-Testing`, `Canvas Input & Edit Sessions`, `Document Model`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.08173076923076923 - nodes in this community are weakly interconnected._
- **Should `Canvas Input & Edit Sessions` be split into smaller, more focused modules?**
  _Cohesion score 0.061754385964912284 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.0912568306010929 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.0824829931972789 - nodes in this community are weakly interconnected._