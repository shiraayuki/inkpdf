# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~28,664 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 476 nodes · 1395 edges · 22 communities (18 shown, 4 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `defe874b`
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
- .style_selection
- annotation_bounds
- canvas.rs
- .style_selection

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 127 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 36 edges
4. `build()` - 29 edges
5. `State` - 23 edges
6. `Document` - 22 edges
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

## Communities (22 total, 4 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.21
Nodes (25): apply_glyph_font(), cursor_at(), draw_annotation(), draw_caret(), draw_glyphs(), draw_glyphs_with_math(), draw_one_glyph(), draw_overlay() (+17 more)

### Community 1 - ".record_change"
Cohesion: 0.11
Nodes (5): ScrolledWindow, Canvas, LassoShape, DrawingArea, FnOnce

### Community 2 - "Window & Tool UI"
Cohesion: 0.07
Nodes (62): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+54 more)

### Community 3 - "Document Model"
Cohesion: 0.06
Nodes (53): ImageSurface, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index(), LatexAnnotation (+45 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.11
Nodes (13): Key, ModifierType, Propagation, ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), read_math_span_rejects_unterminated_or_cross_line() (+5 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.13
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.29
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 11 - "Option"
Cohesion: 0.16
Nodes (12): Cursor, Instant, ModelerInputEventType, circle_cursor(), cursor_from_draw(), PenModel, plus_cursor(), Option (+4 more)

### Community 12 - "Option"
Cohesion: 0.27
Nodes (5): clamp_translate(), clamp_translate_keeps_box_on_page(), translate_annotation(), translate_annotation_shifts_every_kind(), union_bounds()

### Community 13 - ".on_click"
Cohesion: 0.22
Nodes (3): ann_glyphs(), glyphs_from_plain(), Uuid

### Community 15 - ".is_empty"
Cohesion: 0.13
Nodes (17): HeadingLevel, heading_scale(), MathSplit, MathToken, MdLine, MdPiece, MdRun, parse_markdown_lines() (+9 more)

### Community 16 - ".style_selection"
Cohesion: 0.23
Nodes (14): HashMap, a4_page(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), highlighted_text_renders_marker_pixels(), latex_annotation_renders_pixels(), markdown_annotation_renders_pixels(), page_surface() (+6 more)

### Community 17 - "annotation_bounds"
Cohesion: 0.25
Nodes (8): measure_glyphs(), measure_glyphs_with_math(), measure_grows_with_newlines(), measure_latex(), measure_markdown(), R, text_with_math_renders_narrower_than_its_raw_source(), with_scratch()

### Community 18 - "canvas.rs"
Cohesion: 0.11
Nodes (20): annotation_at(), annotation_bounds(), annotation_bounds_covers_all_kinds(), bounds_of_points(), dist_point_segment(), fraction_needs_more_height_than_plain_text(), hit_test(), hit_test_maps_click_to_page_local_point() (+12 more)

## Knowledge Gaps
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Window & Tool UI`, `Document Model`, `Text Editing & Styling`, `Engine / PDF Loading`, `.update_layout`, `Option`, `Option`, `.on_click`, `.attach_input`, `.is_empty`, `.style_selection`, `annotation_bounds`, `canvas.rs`, `.update_layout`, `.style_selection`, `.lift_annotation`?**
  _High betweenness centrality (0.463) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`?**
  _High betweenness centrality (0.119) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `canvas.rs`, `Window & Tool UI`, `Text Editing & Styling`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.096) - this node is a cross-community bridge._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.11333333333333333 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.075 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.0593607305936073 - nodes in this community are weakly interconnected._
- **Should `Text Editing & Styling` be split into smaller, more focused modules?**
  _Cohesion score 0.10631229235880399 - nodes in this community are weakly interconnected._