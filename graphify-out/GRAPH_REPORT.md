# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 9 files · ~20,799 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 390 nodes · 1122 edges · 15 communities (11 shown, 4 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `43553374`
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
- .page_hit
- settings.rs
- Option
- .commit_editing
- .finish_draw

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
- `Draw` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `State` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `color_from_rgba()` --references--> `Color`  [EXTRACTED]
  src/ui/window.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (15 total, 4 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.07
Nodes (66): HashMap, ImageSurface, Instant, ModelerInputEventType, Page, a4_page(), ann_glyphs(), annotation_at() (+58 more)

### Community 2 - "Window & Tool UI"
Cohesion: 0.08
Nodes (63): Application, ApplicationWindow, Button, Cell, ColorDialogButton, IsA, Label, MenuItem (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.06
Nodes (34): Annotation, AnnotationKind, Color, default_font(), PageKind, PagePattern, PdfSource, Default (+26 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.17
Nodes (5): Key, ModifierType, Propagation, text_edit_insert_delete_and_navigate(), TextEdit

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.14
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.12
Nodes (3): ScrolledWindow, Canvas, DrawingArea

### Community 8 - ".page_hit"
Cohesion: 0.17
Nodes (5): clamp_translate(), clamp_translate_keeps_box_on_page(), translate_annotation(), translate_annotation_shifts_every_kind(), union_bounds()

### Community 9 - "settings.rs"
Cohesion: 0.24
Nodes (12): Document, insert_blank_page_adds_page_at_index(), Self, load(), load_rejects_garbage(), roundtrip_preserves_document(), Path, PathBuf (+4 more)

### Community 11 - "Option"
Cohesion: 0.29
Nodes (8): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, Uuid, stroke_halo(), text_cursor()

## Knowledge Gaps
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `Canvas` to `Canvas Rendering & Hit-Testing`, `.record_change`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `.page_hit`, `Option`, `.commit_editing`, `.clamped_local`, `.finish_draw`?**
  _High betweenness centrality (0.477) - this node is a cross-community bridge._
- **Why does `Document` connect `settings.rs` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Engine / PDF Loading`, `.commit_editing`?**
  _High betweenness centrality (0.104) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `Canvas`?**
  _High betweenness centrality (0.095) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.07006151742993848 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07659007659007659 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.06253652834599649 - nodes in this community are weakly interconnected._
- **Should `Engine / PDF Loading` be split into smaller, more focused modules?**
  _Cohesion score 0.14153846153846153 - nodes in this community are weakly interconnected._