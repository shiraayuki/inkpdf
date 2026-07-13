# Graph Report - inkpdf  (2026-07-13)

## Corpus Check
- 10 files · ~25,321 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 449 nodes · 1301 edges · 23 communities (17 shown, 6 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f681b682`
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
- Vec
- .is_empty
- Context
- .new
- .on_click

## God Nodes (most connected - your core abstractions)
1. `Canvas` - 120 edges
2. `TextEdit` - 37 edges
3. `WindowUi` - 36 edges
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
- `refresh()` --calls--> `add_secondary_click()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `refresh()` --calls--> `show_menu()`  [INFERRED]
  src/ui/file_browser.rs → src/ui/window.rs
- `build()` --calls--> `refresh()`  [INFERRED]
  src/ui/window.rs → src/ui/file_browser.rs
- `Draw` --references--> `Color`  [EXTRACTED]
  src/ui/canvas.rs → src/engine/document.rs

## Import Cycles
- None detected.

## Communities (23 total, 6 thin omitted)

### Community 0 - "Canvas Rendering & Hit-Testing"
Cohesion: 0.13
Nodes (22): annotation_at(), annotation_bounds(), annotation_bounds_covers_all_kinds(), bounds_of_points(), dist_point_segment(), eraser_hits(), hit_test(), hit_test_maps_click_to_page_local_point() (+14 more)

### Community 1 - ".record_change"
Cohesion: 0.12
Nodes (4): ScrolledWindow, ShapeKind, Canvas, DrawingArea

### Community 2 - "Window & Tool UI"
Cohesion: 0.08
Nodes (63): Application, ApplicationWindow, Cell, ColorDialogButton, IsA, MenuItem, RGBA, Separator (+55 more)

### Community 3 - "Document Model"
Cohesion: 0.07
Nodes (39): HashMap, ImageSurface, Annotation, AnnotationKind, Color, default_font(), Document, insert_blank_page_adds_page_at_index() (+31 more)

### Community 4 - "Text Editing & Styling"
Cohesion: 0.15
Nodes (9): ctrl_word_navigation_jumps_by_word(), empty_edit(), glyphs_of(), marker_and_bold_apply_to_selection(), shift_selects_and_color_applies_to_selection_only(), text_edit_insert_delete_and_navigate(), text_edit_vertical_keeps_column(), TextEdit (+1 more)

### Community 5 - "Engine / PDF Loading"
Cohesion: 0.13
Nodes (15): file_name(), OpenDocument, pdf_to_inkpdf_roundtrip_rebuilds_renderer(), Option, Path, Result, Self, String (+7 more)

### Community 6 - "Canvas"
Cohesion: 0.26
Nodes (12): AppSettings, load(), load_from(), path(), roundtrip_preserves_settings(), Default, PathBuf, Result (+4 more)

### Community 8 - ".record_change"
Cohesion: 0.18
Nodes (16): ListBox, FileBrowser, icon_button(), initial_dir(), list_dir_entries(), list_dir_entries_shows_dirs_and_pdf_inkpdf_only(), refresh(), Box (+8 more)

### Community 9 - ".page_hit"
Cohesion: 0.24
Nodes (6): clamp_translate(), clamp_translate_keeps_box_on_page(), cursor_at(), translate_annotation(), translate_annotation_shifts_every_kind(), union_bounds()

### Community 11 - "Option"
Cohesion: 0.27
Nodes (7): Cursor, circle_cursor(), cursor_from_draw(), plus_cursor(), Option, stroke_halo(), text_cursor()

### Community 13 - ".on_key"
Cohesion: 0.22
Nodes (3): Key, ModifierType, Propagation

### Community 17 - "Vec"
Cohesion: 0.15
Nodes (17): Instant, ModelerInputEventType, Page, ann_glyphs(), Draw, Glyph, glyphs_from_plain(), History (+9 more)

### Community 18 - ".is_empty"
Cohesion: 0.15
Nodes (16): HeadingLevel, heading_scale(), MathSplit, MdLine, MdPiece, MdRun, parse_markdown_lines(), parse_markdown_lines_list_items_get_prefixes() (+8 more)

### Community 19 - "Context"
Cohesion: 0.27
Nodes (18): apply_glyph_font(), draw_annotation(), draw_caret(), draw_glyphs(), draw_overlay(), draw_shape(), draw_stroke(), draw_text_run() (+10 more)

### Community 20 - ".new"
Cohesion: 0.22
Nodes (12): a4_page(), annotation_at_also_hits_strokes_and_shapes(), annotation_at_hits_inside_and_misses_outside(), draw_page_pattern(), ellipse_shape_does_not_touch_its_bounding_box_corner(), highlighted_text_renders_marker_pixels(), markdown_annotation_renders_pixels(), page_surface() (+4 more)

## Knowledge Gaps
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Canvas` connect `.record_change` to `Canvas Rendering & Hit-Testing`, `Window & Tool UI`, `Document Model`, `Engine / PDF Loading`, `.page_hit`, `Option`, `.record_change`, `.on_key`, `.style_selection`, `.finish_draw`, `FnOnce`, `Vec`, `.is_empty`, `.new`, `.on_click`, `.page_hit`?**
  _High betweenness centrality (0.463) - this node is a cross-community bridge._
- **Why does `WindowUi` connect `Window & Tool UI` to `.record_change`, `.record_change`, `.record_change`, `Canvas`?**
  _High betweenness centrality (0.125) - this node is a cross-community bridge._
- **Why does `Document` connect `Document Model` to `Canvas Rendering & Hit-Testing`, `Vec`, `Window & Tool UI`, `Engine / PDF Loading`?**
  _High betweenness centrality (0.099) - this node is a cross-community bridge._
- **Should `Canvas Rendering & Hit-Testing` be split into smaller, more focused modules?**
  _Cohesion score 0.12535612535612536 - nodes in this community are weakly interconnected._
- **Should `.record_change` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._
- **Should `Window & Tool UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07626582278481013 - nodes in this community are weakly interconnected._
- **Should `Document Model` be split into smaller, more focused modules?**
  _Cohesion score 0.07337662337662337 - nodes in this community are weakly interconnected._