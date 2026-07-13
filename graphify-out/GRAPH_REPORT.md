# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~25,080 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 447 nodes · 1289 edges · 17 communities (12 shown, 5 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 2 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `98a167c3`
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
- .page_hit
- Option
- .record_change
- .on_key
- .style_selection
- .finish_draw
- FnOnce

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 120 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 34 edges
4. `build()` - 29 edges
5. `Document` - 22 edges
6. `State` - 22 edges
7. `Color` - 21 edges
8. `draw_overlay()` - 17 edges
9. `AnnotationKind` - 16 edges
10. `Page` - 16 edges

## Surprising Connections (you probably didn't know these)
- `pattern_thumbnail()` --calls--> `draw_page_pattern()`  [INFERRED]
  src/ui/window.rs → src/ui/canvas.rs
- `build()` --calls--> `refresh()`  [INFERRED]
  src/ui/window.rs → src/ui/file_browser.rs
- `Draw` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `AppSettings` --references--> `Color`  [EXTRACTED]
  src/ui/settings.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (17 total, 5 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.05
Nodes (86): HeadingLevel, Instant, ModelerInputEventType, Page, a4_page(), ann_glyphs(), annotation_at(), annotation_at_also_hits_strokes_and_shapes() (+78 more)

### Community 1 - ".record_change"
Cohesion: 0.12
Nodes (4): ScrolledWindow, ShapeKind, Canvas, DrawingArea

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (63): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.08
Nodes (37): HashMap, ImageSurface, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index() (+29 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.15
Nodes (9): ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), shift_selects_and_color_applies_to_selection_only(), text_edit_insert_delete_and_navigate(), text_edit_vertical_keeps_column(), TextEdit (+1 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.13
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 9 - ".page_hit"
Cohesion: 0.17
Nodes (5): clamp_translate(), clamp_translate_keeps_box_on_page(), translate_annotation(), translate_annotation_shifts_every_kind(), union_bounds()

### Community 11 - "Option"
Cohesion: 0.12
Nodes (10): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, Uuid, stroke_halo(), text_cursor() (+2 more)

### Community 13 - ".on_key"
Cohesion: 0.22
Nodes (3): Key, ModifierType, Propagation

## Knowledge Gaps
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Engine / PDF Loading`, `.page_hit`, `Option`, `.record_change`, `.on_key`, `.style_selection`, `.finish_draw`, `FnOnce`?**
  _High betweenness centrality (0.469) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`?**
  _High betweenness centrality (0.143) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.096) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.05306930693069307 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07468354430379746 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.08069381598793364 - nodes in this community are weakly interconnected._