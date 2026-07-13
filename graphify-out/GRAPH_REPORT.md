# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~29,160 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 482 nodes · 1420 edges · 19 communities (12 shown, 7 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ac8ec8b5`
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
- .update_layout
- Option
- Option
- .on_click
- .attach_input
- .is_empty
- canvas.rs
- .update_layout

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 128 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 37 edges
4. `build()` - 30 edges
5. `Document` - 23 edges
6. `State` - 23 edges
7. `Color` - 20 edges
8. `page_surface()` - 17 edges
9. `draw_overlay()` - 17 edges
10. `AnnotationKind` - 16 edges

## Surprising Connections (you probably didn't know these)
- `refresh()` --calls--> `add_secondary_click()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `refresh()` --calls--> `show_menu()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `build()` --calls--> `refresh()`  [INFERRED]
  src/ui/window.rs → src/ui/file_browser.rs
- `draw_stroke()` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs
- `AppSettings` --references--> `Color`  [EXTRACTED]
  src/ui/settings.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (19 total, 7 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.07
Nodes (71): HeadingLevel, ann_glyphs(), annotation_at(), annotation_bounds(), annotation_bounds_covers_all_kinds(), apply_glyph_font(), bounds_of_points(), cursor_at() (+63 more)

### Community 1 - ".record_change"
Cohesion: 0.10
Nodes (5): ScrolledWindow, Canvas, LassoShape, DrawingArea, Fn

### Community 2 - "Window & Tool UI"
Cohesion: 0.08
Nodes (63): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.06
Nodes (52): HashMap, ImageSurface, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index() (+44 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.12
Nodes (14): Key, ModifierType, Propagation, ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), read_math_span_rejects_unterminated_or_cross_line() (+6 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.14
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.26
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 11 - "Option"
Cohesion: 0.09
Nodes (28): Cursor, Instant, ModelerInputEventType, a4_page(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), circle_cursor(), cursor_from_draw() (+20 more)

### Community 12 - "Option"
Cohesion: 0.27
Nodes (5): clamp_translate(), clamp_translate_keeps_box_on_page(), translate_annotation(), translate_annotation_shifts_every_kind(), union_bounds()

## Knowledge Gaps
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `.update_layout`, `Option`, `Option`, `.on_click`, `.attach_input`, `.is_empty`, `canvas.rs`, `.update_layout`, `.lift_annotation`?**
  _High betweenness centrality (0.460) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`, `Canvas`, `.update_layout`?**
  _High betweenness centrality (0.119) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Engine / PDF Loading`, `Option`, `.update_layout`?**
  _High betweenness centrality (0.097) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.06835443037974684 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.0989247311827957 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07721518987341772 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.06252587991718427 - nodes in this community are weakly interconnected._