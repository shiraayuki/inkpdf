# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 9 files · ~18,277 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 366 nodes · 1037 edges · 15 communities (11 shown, 4 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c511735d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Canvas Rendering & Hit-Testing
- Canvas Input & Edit Sessions
- Window & Tool UI
- Document Model
- Text Editing & Styling
- Engine / PDF Loading
- Canvas
- App Entry Point
- settings.rs
- Option
- .style_selection
- .on_click
- .page_hit

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 104 edges
2. `TextEdit` - 35 edges
3. `WindowUi` - 31 edges
4. `build()` - 28 edges
5. `Document` - 22 edges
6. `Color` - 20 edges
7. `State` - 20 edges
8. `Page` - 16 edges
9. `TextStyle` - 15 edges
10. `draw_overlay()` - 15 edges

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

## Communities (15 total, 4 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.07
Nodes (62): HashMap, ImageSurface, Instant, ModelerInputEventType, Page, a4_page(), ann_glyphs(), annotation_at() (+54 more)

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (62): Application, ApplicationWindow, Button, Cell, ColorDialogButton, IsA, Label, MenuItem (+54 more)

### Community 3 - "Document Model"
Cohesion: 0.08
Nodes (32): Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), PageKind, PagePattern (+24 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.17
Nodes (5): Key, ModifierType, Propagation, text_edit_insert_delete_and_navigate(), TextEdit

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.14
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.14
Nodes (3): ScrolledWindow, Canvas, DrawingArea

### Community 9 - "settings.rs"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 11 - "Option"
Cohesion: 0.27
Nodes (7): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, stroke_halo(), text_cursor()

### Community 12 - ".style_selection"
Cohesion: 0.22
Nodes (3): Fn, String, text_of()

## Knowledge Gaps
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas` to `Canvas Rendering & Hit-Testing`, `Canvas Input & Edit Sessions`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `.commit_editing`, `Option`, `.style_selection`, `.on_click`, `.page_hit`?**
  _High betweenness centrality (0.463) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `.commit_editing`, `Window & Tool UI`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.110) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `Canvas`?**
  _High betweenness centrality (0.099) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.07219548315438726 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07425907425907426 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.07686274509803921 - nodes in this community are weakly interconnected._
- **Should `Engine / PDF Loading` be split into smaller, more focused modules?**
  _Cohesion score 0.14153846153846153 - nodes in this community are weakly interconnected._