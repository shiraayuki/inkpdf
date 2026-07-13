# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 9 files · ~20,872 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 391 nodes · 1123 edges · 11 communities (10 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `5ad9c46d`
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
- settings.rs
- Option

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 116 edges
2. `TextEdit` - 35 edges
3. `WindowUi` - 31 edges
4. `build()` - 28 edges
5. `Document` - 22 edges
6. `Color` - 21 edges
7. `State` - 21 edges
8. `draw_overlay()` - 17 edges
9. `Page` - 16 edges
10. `TextStyle` - 15 edges

## Surprising Connections (you probably didn't know these)
- `pattern_thumbnail()` --calls--> `draw_page_pattern()`  [INFERRED]
  src/ui/window.rs → src/ui/canvas.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `AppSettings` --references--> `Color`  [EXTRACTED]
  src/ui/settings.rs → src/engine/document.rs
- `color_from_rgba()` --references--> `Color`  [EXTRACTED]
  src/ui/window.rs → src/engine/document.rs
- `apply_glyph_font()` --references--> `TextStyle`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (11 total, 1 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.08
Nodes (51): HashMap, a4_page(), ann_glyphs(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), annotation_bounds(), annotation_bounds_covers_all_kinds(), apply_glyph_font() (+43 more)

### Community 1 - ".record_change"
Cohesion: 0.06
Nodes (11): ScrolledWindow, PagePattern, Canvas, content_size(), rects_intersect(), Relative, DrawingArea, Fn (+3 more)

### Community 2 - "Window & Tool UI"
Cohesion: 0.08
Nodes (63): Application, ApplicationWindow, Button, Cell, ColorDialogButton, IsA, Label, MenuItem (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.08
Nodes (37): ImageSurface, Instant, ModelerInputEventType, Annotation, AnnotationKind, Color, default_font(), Page (+29 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.13
Nodes (4): Key, ModifierType, Propagation, TextEdit

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.13
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.26
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 9 - "settings.rs"
Cohesion: 0.24
Nodes (12): Document, insert_blank_page_adds_page_at_index(), Self, load(), load_rejects_garbage(), roundtrip_preserves_document(), Path, PathBuf (+4 more)

### Community 11 - "Option"
Cohesion: 0.13
Nodes (12): Cursor, annotation_at(), circle_cursor(), cursor_from_draw(), hit_test(), hit_test_maps_click_to_page_local_point(), plus_cursor(), Option (+4 more)

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `Option`?**
  _High betweenness centrality (0.476) - this node is a cross-community bridge._
- **Why does `Document` connect `settings.rs` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.104) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `Canvas`?**
  _High betweenness centrality (0.095) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.07505827505827506 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.05516475379489078 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07792207792207792 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.07767722473604827 - nodes in this community are weakly interconnected._